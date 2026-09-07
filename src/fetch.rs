//! 祝日データの取得と保存。判定サブコマンドはローカルだけを読むため、ここが唯一のネット経路になる。

use std::{
    error::Error,
    fmt, fs, io,
    io::Write,
    path::{Path, PathBuf},
    process,
    str::from_utf8,
    time::Duration,
};

use encoding_rs::{Encoding, SHIFT_JIS, UTF_8};
use jiff::{Timestamp, tz::TimeZone};

use crate::holidays::{HolidayData, HolidayError};

/// `--source` 未指定時の取得先 (doc/design/decision-log/0009-holiday-csv-local-format.md)。
pub const DEFAULT_SOURCE: &str = "https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv";

/// メタ形式の版。読み取り側が受け付ける値と揃える。
const SCHEMA: &str = "1";

/// UTF-8 の BOM。取得元が付けてきた場合に取り除く。
const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// 取得全体の上限。応答しない取得元で、cron やスクリプトから回した実行が張り付かないようにする。
const TIMEOUT_GLOBAL: Duration = Duration::from_secs(30);

/// 接続確立の上限。到達できない取得元を全体上限より早く諦める。
const TIMEOUT_CONNECT: Duration = Duration::from_secs(10);

/// 取得元が返した本文と、宣言された文字コード。HTTP 層と変換層を分けるための受け渡し。
#[derive(Debug)]
pub struct Payload {
    /// Content-Type の charset parameter。既定の取得先のように宣言が無ければ `None`。
    pub charset: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub enum FetchError {
    InvalidSource(String),
    Request(Box<dyn Error + Send + Sync>),
    UnknownCharset(String),
    Undecodable {
        encoding: &'static str,
        /// 取得元が宣言した文字コードか、宣言が無く本文から決めたか。
        declared: bool,
    },
    Destination(HolidayError),
    Rejected(HolidayError),
    Io(io::Error),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource(source) => write!(f, "source must be an http(s) URL: {source:?}"),
            Self::Request(error) => write!(f, "cannot fetch holiday data: {error}"),
            Self::UnknownCharset(label) => write!(f, "unknown source charset: {label:?}"),
            Self::Undecodable {
                encoding,
                declared: true,
            } => write!(f, "source is not valid {encoding}"),
            Self::Undecodable {
                encoding,
                declared: false,
            } => write!(f, "source declares no charset and is not valid {encoding}"),
            Self::Destination(error) => error.fmt(f),
            Self::Rejected(error) => write!(f, "fetched data is unusable: {error}"),
            Self::Io(error) => write!(f, "cannot save holiday data: {error}"),
        }
    }
}

impl Error for FetchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) => Some(error.as_ref()),
            Self::Destination(error) => Some(error),
            Self::Rejected(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// 取得から保存までの全体。保存先の解決と取得の実体は `decide` と同じく closure で受け取り、
/// 利用者入力の検証を先に済ませる。保存できた path を返す。
pub fn save(
    source: &str,
    data_path: impl FnOnce() -> Result<PathBuf, HolidayError>,
    now: Timestamp,
    download: impl FnOnce(&str) -> Result<Payload, FetchError>,
) -> Result<PathBuf, FetchError> {
    validate_source(source)?;
    // 保存先を解決できないまま取得へ進まない。ネットワークより先に環境を確かめる。
    let path = data_path().map_err(FetchError::Destination)?;
    let payload = download(source)?;
    let document = compose(source, &decode(&payload)?, now);
    // 読めない本文で既存データを置き換えない。取得は成功しても保存はしない。
    HolidayData::parse(&document, now).map_err(FetchError::Rejected)?;
    replace(&path, &document).map_err(FetchError::Io)?;
    Ok(path)
}

/// 実際の HTTP GET。redirect と TLS は ureq に委ねる。
pub fn download(source: &str) -> Result<Payload, FetchError> {
    let request = |error: ureq::Error| FetchError::Request(Box::new(error));
    let mut response = agent().get(source).call().map_err(request)?;
    let charset = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(charset_of);
    let body = response.body_mut().read_to_vec().map_err(request)?;
    Ok(Payload { charset, body })
}

/// 取得に上限を入れた agent。ureq の既定は `await_100` 以外すべて timeout 無しであり、
/// 応答しない取得元に当たると exit code へ写像される前に実行が終わらなくなる。
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT_GLOBAL))
        .timeout_connect(Some(TIMEOUT_CONNECT))
        .build()
        .new_agent()
}

/// `--source` は http(s) だけを受ける (doc/design/cli-interface.md)。
/// ローカル path を取得元にせず、メタ行を壊す改行や空白も先に弾く。
fn validate_source(source: &str) -> Result<(), FetchError> {
    let scheme = source.split_once("://").map_or("", |(scheme, _)| scheme);
    let supported = scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https");
    if !supported || source.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(FetchError::InvalidSource(source.to_owned()));
    }
    Ok(())
}

/// Content-Type から charset parameter を取り出す。media type 側は見ない。
/// 空値は文字コードを何も伝えていないため、宣言が無い場合と同じく推定へ落とす。
fn charset_of(content_type: &str) -> Option<String> {
    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.split_once('=')?;
        name.trim()
            .eq_ignore_ascii_case("charset")
            .then(|| value.trim().trim_matches('"').to_owned())
            .filter(|value| !value.is_empty())
    })
}

/// 保存は UTF-8 に揃える (doc/design/business-day.md)。既定の取得先は charset を宣言せず
/// CP932 を返すため、宣言が無い場合だけ本文から推定する。置換文字で埋めた本文は保存しない。
fn decode(payload: &Payload) -> Result<String, FetchError> {
    // 宣言と推定は診断で区別する。推定した文字コードの名前だけを出すと、
    // 取得元がそれを名乗っていない場合に読み手が原因を追えない。
    let (encoding, declared, body) = if let Some(body) = payload.body.strip_prefix(BOM) {
        (UTF_8, true, body)
    } else if let Some(label) = payload.charset.as_deref() {
        let encoding = Encoding::for_label(label.as_bytes())
            .ok_or_else(|| FetchError::UnknownCharset(label.to_owned()))?;
        (encoding, true, payload.body.as_slice())
    } else if from_utf8(&payload.body).is_ok() {
        (UTF_8, false, payload.body.as_slice())
    } else {
        (SHIFT_JIS, false, payload.body.as_slice())
    };
    encoding
        .decode_without_bom_handling_and_without_replacement(body)
        .map(|decoded| decoded.into_owned())
        .ok_or(FetchError::Undecodable {
            encoding: encoding.name(),
            declared,
        })
}

/// メタ行を先頭に付ける。`expires_at` は書かず、読み取り側の `fetched_at + 1 year` に委ねる。
/// 本文は取得元のまま置き、列構成にも改行コードにも手を入れない。
fn compose(source: &str, body: &str, now: Timestamp) -> String {
    // 読み取り側は秒の小数部も受けるが、メタ行は秒までの UTC RFC 3339 に揃える。
    let fetched_at = now.to_zoned(TimeZone::UTC).strftime("%Y-%m-%dT%H:%M:%SZ");
    format!(
        "# bizdate-meta schema={SCHEMA}\n# fetched_at={fetched_at}\n# source_url={source}\n{body}"
    )
}

/// 同じディレクトリの一時ファイルへ書き切ってから rename する。
/// 途中で失敗した実行が、それまで有効だったローカルデータを壊した状態で残さない。
fn replace(path: &Path, document: &str) -> io::Result<()> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "holiday data path must name a file inside a directory",
        ));
    };
    fs::create_dir_all(parent)?;
    let temporary = Temporary(Some(parent.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        process::id()
    ))));
    let mut file = fs::File::create(temporary.path())?;
    file.write_all(document.as_bytes())?;
    // 内容が届いていないファイルで既存データを置き換えない。
    file.sync_all()?;
    drop(file);
    temporary.persist(path)
}

/// rename に成功するまで一時ファイルを掃除する。失敗した実行が残骸を残さない。
struct Temporary(Option<PathBuf>);

impl Temporary {
    fn path(&self) -> &Path {
        self.0
            .as_deref()
            .expect("the path is taken only by persist")
    }

    fn persist(mut self, path: &Path) -> io::Result<()> {
        let temporary = self.0.take().expect("the path is taken only here");
        // 同一ディレクトリ内の rename であり、置き換えは分割されない。
        fs::rename(&temporary, path).inspect_err(|_| {
            let _ = fs::remove_file(&temporary);
        })
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests;
