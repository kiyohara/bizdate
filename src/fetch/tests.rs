use super::*;
use crate::test_support::{TempDir, now};

/// 取得元が返す本文。メタ行は保存側が付ける。既定の取得先と同じく CRLF で改行する。
const SOURCE_BODY: &str =
    "国民の祝日・休日月日,国民の祝日・休日名称\r\n2026/1/1,元日\r\n2026/9/22,休日\r\n";

fn payload(charset: Option<&str>, body: Vec<u8>) -> Payload {
    Payload {
        charset: charset.map(str::to_owned),
        body,
    }
}

/// 既定の取得先と同じ条件。charset を宣言せず CP932 の本文を返す。
fn cp932() -> Payload {
    payload(None, SHIFT_JIS.encode(SOURCE_BODY).0.into_owned())
}

/// 差し替える取得の実体。失敗の種類ごとに 1 つ用意する。
type Download = Box<dyn Fn(&str) -> Result<Payload, FetchError>>;

fn refused(_: &str) -> Result<Payload, FetchError> {
    Err(FetchError::Request(Box::new(io::Error::from(
        io::ErrorKind::ConnectionRefused,
    ))))
}

/// 保存先とその親を含む一時ディレクトリ。親は保存側に作らせる。
fn destination(dir: &TempDir) -> PathBuf {
    dir.0.join("bizdate/holidays/holidays.csv")
}

/// 保存先の解決も closure で渡す。`decide` と同じく利用者入力の検証を先に通すため。
fn at(path: &Path) -> impl FnOnce() -> Result<PathBuf, HolidayError> + use<> {
    let path = path.to_owned();
    move || Ok(path)
}

/// 一時ファイルを残していないことも含めて、ディレクトリの中身を見る。
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn source_must_be_an_http_url() {
    for source in ["http://example.com/h.csv", "HTTPS://example.com/h.csv"] {
        assert!(validate_source(source).is_ok(), "{source}");
    }
    for source in [
        // ローカル path や他 scheme を取得元にしない。
        "file:///etc/passwd",
        "/tmp/holidays.csv",
        "example.com/h.csv",
        "ftp://example.com/h.csv",
        "",
        // メタ行を壊す改行や空白を先に弾く。
        "https://example.com/h.csv\n# source_url=https://evil.example",
        "https://example.com/a b.csv",
    ] {
        assert!(
            matches!(validate_source(source), Err(FetchError::InvalidSource(_))),
            "{source:?}"
        );
    }
}

#[test]
fn undeclared_charset_falls_back_to_shift_jis_only_when_utf8_fails() {
    // 既定の取得先は charset を宣言せず CP932 を返す。
    assert_eq!(decode(&cp932()).unwrap(), SOURCE_BODY);
    // UTF-8 として読めるなら変換しない。
    let utf8 = payload(None, SOURCE_BODY.as_bytes().to_vec());
    assert_eq!(decode(&utf8).unwrap(), SOURCE_BODY);
}

#[test]
fn declared_charset_is_used_and_a_bom_overrides_it() {
    // 宣言があれば推定しない。UTF-8 として読める本文でも Shift_JIS として解釈する。
    let mistaken = payload(Some("shift_jis"), "国".as_bytes().to_vec());
    assert_ne!(decode(&mistaken).unwrap(), "国");
    // label の大小は問わない。quote の除去は Content-Type の解析側で済ませる。
    let declared = payload(Some("Shift_JIS"), cp932().body);
    assert_eq!(decode(&declared).unwrap(), SOURCE_BODY);

    // BOM が付いていれば宣言より優先し、BOM 自体は残さない。
    let mut body = BOM.to_vec();
    body.extend_from_slice(SOURCE_BODY.as_bytes());
    assert_eq!(
        decode(&payload(Some("shift_jis"), body)).unwrap(),
        SOURCE_BODY
    );
}

#[test]
fn unusable_encodings_are_errors_rather_than_replacement_characters() {
    let unknown = payload(Some("no-such-charset"), cp932().body);
    assert!(matches!(
        decode(&unknown),
        Err(FetchError::UnknownCharset(label)) if label == "no-such-charset"
    ));
    // 宣言どおりに読めない本文を、置換文字で埋めた本文として保存しない。
    let mislabelled = payload(Some("utf-8"), cp932().body);
    let error = decode(&mislabelled).unwrap_err();
    assert!(
        matches!(
            error,
            FetchError::Undecodable {
                encoding: "UTF-8",
                declared: true
            }
        ),
        "{error}"
    );
    assert_eq!(error.to_string(), "source is not valid UTF-8");

    // 推定で選んだ文字コードは、取得元が名乗ったものと同じ文言にしない。
    let guessed = payload(None, vec![0x00, 0xFF, 0x81, 0x20]);
    let error = decode(&guessed).unwrap_err();
    assert!(
        matches!(
            error,
            FetchError::Undecodable {
                encoding: "Shift_JIS",
                declared: false
            }
        ),
        "{error}"
    );
    assert_eq!(
        error.to_string(),
        "source declares no charset and is not valid Shift_JIS"
    );
}

#[test]
fn content_type_charset_is_read_only_from_the_parameters() {
    for (header, expected) in [
        ("text/csv", None),
        ("text/csv; charset=Shift_JIS", Some("Shift_JIS")),
        ("text/csv;charset=\"utf-8\"", Some("utf-8")),
        ("text/csv; foo=bar; charset = utf-8", Some("utf-8")),
        // media type 側の charset らしき文字列は parameter ではない。
        ("charset/csv", None),
        // 空値は何も伝えていない。宣言が無い場合と同じく推定へ落とす。
        ("text/csv; charset=", None),
        ("text/csv; charset=\"\"", None),
        ("text/csv; charset= ", None),
    ] {
        assert_eq!(charset_of(header).as_deref(), expected, "{header}");
    }
}

#[test]
fn saved_data_carries_metadata_and_is_readable_by_the_judgment_side() {
    let dir = TempDir::new();
    let path = destination(&dir);
    // 親ディレクトリが無い状態から保存できる。
    assert!(!path.parent().unwrap().exists());
    let saved_to = save("https://example.com/h.csv", at(&path), now(), |_| {
        Ok(cp932())
    })
    .unwrap();
    // 保存できた path を返す。CLI はこれを stdout へ出す。
    assert_eq!(saved_to, path);

    let saved = fs::read_to_string(&path).unwrap();
    assert_eq!(
        saved,
        format!(
            "# bizdate-meta schema=1\n\
             # fetched_at=2026-09-06T02:24:33Z\n\
             # source_url=https://example.com/h.csv\n\
             {SOURCE_BODY}"
        )
    );
    // 保存直後は expires_at を書かず、読み取り側の fetched_at + 1 year に委ねる。
    assert!(!saved.contains("expires_at"));

    // 判定側と往復する。CP932 の祝日名も含めて読める。
    let data = HolidayData::load(&path, now()).unwrap();
    assert!(
        data.is_holiday("2026-09-22".parse().unwrap(), now())
            .unwrap()
    );
    assert!(
        !data
            .is_holiday("2026-09-23".parse().unwrap(), now())
            .unwrap()
    );
    // 一時ファイルを残さない。
    assert_eq!(entries(path.parent().unwrap()), ["holidays.csv"]);
}

#[test]
fn a_failed_run_leaves_the_previous_data_intact() {
    let dir = TempDir::new();
    let path = destination(&dir);
    save("https://example.com/h.csv", at(&path), now(), |_| {
        Ok(cp932())
    })
    .unwrap();
    let saved = fs::read_to_string(&path).unwrap();

    // 取得失敗と、ローカル形式として読めない本文。どちらも既存データを置き換えない。
    let rejected = |body: &str| payload(None, body.as_bytes().to_vec());
    let attempts: [(&str, Download); 3] = [
        ("取得失敗", Box::new(refused)),
        (
            "別スキーマ",
            Box::new(move |_: &str| Ok(rejected("date,name\r\n2026/1/1,元日\r\n"))),
        ),
        (
            "エラーページ",
            Box::new(move |_: &str| Ok(rejected("<html>404</html>"))),
        ),
    ];
    for (label, download) in attempts {
        let error = save("https://example.com/h.csv", at(&path), now(), download).unwrap_err();
        assert!(!matches!(error, FetchError::Io(_)), "{label}: {error}");
        assert_eq!(fs::read_to_string(&path).unwrap(), saved, "{label}");
        assert_eq!(entries(path.parent().unwrap()), ["holidays.csv"], "{label}");
    }

    // 用法誤りの取得元でも、取得そのものを試みない。
    let unreachable = save("file:///etc/passwd", at(&path), now(), |_| {
        panic!("must not download from a rejected source")
    });
    assert!(matches!(unreachable, Err(FetchError::InvalidSource(_))));
    assert_eq!(fs::read_to_string(&path).unwrap(), saved);
}

#[test]
fn user_input_is_checked_before_the_destination_and_the_network() {
    // `--source` の誤りと保存先の不正が重なっても、先に返るのは利用者入力の方である。
    let error = save(
        "file:///etc/passwd",
        || Err(HolidayError::InvalidHome),
        now(),
        |_| panic!("must not download"),
    )
    .unwrap_err();
    assert!(matches!(error, FetchError::InvalidSource(_)), "{error}");

    // 保存先を解決できないまま取得へ進まない。診断は読み取り側の文言をそのまま出す。
    let error = save(
        "https://example.com/h.csv",
        || Err(HolidayError::InvalidDataHome),
        now(),
        |_| panic!("must not download"),
    )
    .unwrap_err();
    assert!(matches!(error, FetchError::Destination(_)), "{error}");
    assert_eq!(error.to_string(), HolidayError::InvalidDataHome.to_string());
}

#[test]
fn the_agent_bounds_how_long_a_fetch_can_take() {
    // ureq の既定は await_100 以外すべて timeout 無しであり、応答しない取得元で終了しない。
    let timeouts = agent().config().timeouts();
    assert_eq!(timeouts.global, Some(TIMEOUT_GLOBAL));
    assert_eq!(timeouts.connect, Some(TIMEOUT_CONNECT));
    // 接続の上限は全体の上限を超えない。
    assert!(TIMEOUT_CONNECT <= TIMEOUT_GLOBAL);
}

#[test]
fn saving_reports_the_reason_when_the_destination_is_unusable() {
    let dir = TempDir::new();
    // ファイルを親ディレクトリにできない。書き込み失敗として返す。
    let blocked = dir.0.join("blocked");
    fs::write(&blocked, "not a directory").unwrap();
    let error = save(
        "https://example.com/h.csv",
        at(&blocked.join("holidays.csv")),
        now(),
        |_| Ok(cp932()),
    )
    .unwrap_err();
    assert!(matches!(error, FetchError::Io(_)), "{error}");
    assert_eq!(fs::read_to_string(&blocked).unwrap(), "not a directory");
}
