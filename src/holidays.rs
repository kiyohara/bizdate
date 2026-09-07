//! UTF-8 のローカル祝日 CSV を読む。取得・保存と CLI の exit code は扱わない。

use std::{
    collections::BTreeSet,
    env,
    error::Error,
    ffi::OsString,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};

const HEADER: [&str; 2] = ["国民の祝日・休日月日", "国民の祝日・休日名称"];

#[derive(Debug)]
pub enum HolidayError {
    InvalidDataHome,
    InvalidHome,
    MissingData,
    Io(io::Error),
    InvalidEncoding(std::string::FromUtf8Error),
    InvalidMetadata(&'static str),
    UnsupportedSchema(String),
    Csv(csv::Error),
    InvalidHeader,
    InvalidDate(String),
    EmptyData,
    Expired,
    YearOutOfRange { year: i16, first: i16, last: i16 },
}

impl fmt::Display for HolidayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDataHome => write!(f, "XDG_DATA_HOME must be a nonempty absolute path"),
            Self::InvalidHome => write!(
                f,
                "HOME must be an absolute path when XDG_DATA_HOME is unset"
            ),
            Self::MissingData => write!(f, "local holiday data is missing"),
            Self::Io(error) => write!(f, "cannot read holiday data: {error}"),
            Self::InvalidEncoding(error) => write!(f, "holiday data must be UTF-8: {error}"),
            Self::InvalidMetadata(key) => write!(f, "missing or invalid holiday metadata: {key}"),
            Self::UnsupportedSchema(schema) => write!(f, "unsupported holiday schema: {schema:?}"),
            Self::Csv(error) => write!(f, "invalid holiday CSV: {error}"),
            Self::InvalidHeader => write!(f, "invalid holiday CSV header"),
            Self::InvalidDate(input) => write!(f, "invalid holiday date: {input:?}"),
            Self::EmptyData => write!(f, "holiday CSV contains no dates"),
            Self::Expired => write!(f, "local holiday data has expired"),
            Self::YearOutOfRange { year, first, last } => {
                write!(
                    f,
                    "holiday data does not cover {year} (covered: {first}..={last})"
                )
            }
        }
    }
}

impl Error for HolidayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidEncoding(error) => Some(error),
            Self::Csv(error) => Some(error),
            _ => None,
        }
    }
}

/// 取得・保存側とも共有する保存先。パスの解決だけを行い、ディレクトリは作らない。
pub fn data_path() -> Result<PathBuf, HolidayError> {
    data_path_with(|key| env::var_os(key))
}

fn data_path_with(
    mut read_env: impl FnMut(&str) -> Option<OsString>,
) -> Result<PathBuf, HolidayError> {
    let base = match read_env("XDG_DATA_HOME") {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(HolidayError::InvalidDataHome);
            }
            path
        }
        None => {
            let home = read_env("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute());
            home.ok_or(HolidayError::InvalidHome)?.join(".local/share")
        }
    };
    Ok(base.join("bizdate/holidays/holidays.csv"))
}

/// 一度読み取った祝日集合。照会ごとに期限と対象年を検証する。
#[derive(Debug)]
pub struct HolidayData {
    dates: BTreeSet<Date>,
    expires_at: Timestamp,
    first_year: i16,
    last_year: i16,
}

impl HolidayData {
    /// ローカルファイルだけを読む。欠落や不正データから空の集合へ fallback しない。
    pub fn load(path: &Path, now: Timestamp) -> Result<Self, HolidayError> {
        let bytes = fs::read(path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                HolidayError::MissingData
            } else {
                HolidayError::Io(error)
            }
        })?;
        let input = String::from_utf8(bytes).map_err(HolidayError::InvalidEncoding)?;
        Self::parse(&input, now)
    }

    /// 取得側も保存前の検証に使う。読めない本文で既存データを置き換えないため。
    pub(crate) fn parse(input: &str, now: Timestamp) -> Result<Self, HolidayError> {
        let (metadata, body) = Metadata::parse(input)?;
        let expires_at = metadata.expiration()?;
        if now >= expires_at {
            return Err(HolidayError::Expired);
        }

        let mut reader = csv::Reader::from_reader(body.as_bytes());
        if reader
            .headers()
            .map_err(HolidayError::Csv)?
            .iter()
            .ne(HEADER)
        {
            return Err(HolidayError::InvalidHeader);
        }
        let mut dates = BTreeSet::new();
        for record in reader.records() {
            let record = record.map_err(HolidayError::Csv)?;
            dates.insert(parse_csv_date(&record[0])?);
        }
        let (Some(first), Some(last)) = (dates.first(), dates.last()) else {
            return Err(HolidayError::EmptyData);
        };
        let (first_year, last_year) = (first.year(), last.year());
        Ok(Self {
            dates,
            expires_at,
            first_year,
            last_year,
        })
    }

    /// 土日等による短絡判定の前にも呼び、データの有効性を確認する。
    pub fn validate_for(&self, date: Date, now: Timestamp) -> Result<(), HolidayError> {
        if now >= self.expires_at {
            return Err(HolidayError::Expired);
        }
        let year = date.year();
        if !(self.first_year..=self.last_year).contains(&year) {
            return Err(HolidayError::YearOutOfRange {
                year,
                first: self.first_year,
                last: self.last_year,
            });
        }
        Ok(())
    }

    /// 年カバー外や期限切れを「祝日ではない」として返さない。
    pub fn is_holiday(&self, date: Date, now: Timestamp) -> Result<bool, HolidayError> {
        self.validate_for(date, now)?;
        Ok(self.dates.contains(&date))
    }
}

#[derive(Default)]
struct Metadata<'a> {
    schema: Option<&'a str>,
    fetched_at: Option<&'a str>,
    source_url: Option<&'a str>,
    expires_at: Option<&'a str>,
}

impl<'a> Metadata<'a> {
    fn parse(mut body: &'a str) -> Result<(Self, &'a str), HolidayError> {
        let mut metadata = Self::default();
        // 本文中の # は CSV の一部。先頭の連続するメタ行だけを取り除く。
        while body.starts_with('#') {
            let (line, rest) = body.split_once('\n').unwrap_or((body, ""));
            body = rest;
            let line = line[1..].trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let field = match key {
                "bizdate-meta schema" => &mut metadata.schema,
                "fetched_at" => &mut metadata.fetched_at,
                "source_url" => &mut metadata.source_url,
                "expires_at" => &mut metadata.expires_at,
                _ => continue,
            };
            if field.replace(value).is_some() {
                return Err(HolidayError::InvalidMetadata("duplicate field"));
            }
        }
        Ok((metadata, body))
    }

    fn expiration(&self) -> Result<Timestamp, HolidayError> {
        match self.schema {
            Some("1") => {}
            Some(schema) => return Err(HolidayError::UnsupportedSchema(schema.to_owned())),
            None => return Err(HolidayError::InvalidMetadata("schema")),
        }
        if self.source_url.is_none_or(str::is_empty) {
            return Err(HolidayError::InvalidMetadata("source_url"));
        }
        let fetched = parse_timestamp(self.fetched_at, "fetched_at")?;
        if let Some(expiration) = self.expires_at {
            return parse_timestamp(Some(expiration), "expires_at");
        }
        // UTC の暦年を加算する。2/29 は翌年の 2 月末に丸める。
        fetched
            .to_zoned(TimeZone::UTC)
            .checked_add(1.year())
            .map(|value| value.timestamp())
            .map_err(|_| HolidayError::InvalidMetadata("fetched_at + 1 year"))
    }
}

fn parse_timestamp(input: Option<&str>, key: &'static str) -> Result<Timestamp, HolidayError> {
    let invalid = || HolidayError::InvalidMetadata(key);
    let input = input.ok_or_else(invalid)?;
    // Jiff は RFC 3339 より広い構文を受け付けるため、UTC RFC 3339 に絞る。
    let utc = input
        .strip_suffix('Z')
        .or_else(|| input.strip_suffix('z'))
        .or_else(|| input.strip_suffix("+00:00"))
        .ok_or_else(invalid)?;
    let bytes = utc.as_bytes();
    if bytes.len() < 19
        || !bytes.iter().take(19).enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            10 => matches!(b, b'T' | b't'),
            13 | 16 => *b == b':',
            _ => b.is_ascii_digit(),
        })
        || (bytes.len() > 19
            && (bytes[19] != b'.'
                || bytes.len() == 20
                || !bytes[20..].iter().all(u8::is_ascii_digit)))
    {
        return Err(invalid());
    }
    input.parse().map_err(|_| invalid())
}

fn parse_csv_date(input: &str) -> Result<Date, HolidayError> {
    let invalid = || HolidayError::InvalidDate(input.to_owned());
    let mut parts = input.split('/');
    let year = parts.next().ok_or_else(invalid)?;
    let month = parts.next().ok_or_else(invalid)?;
    let day = parts.next().ok_or_else(invalid)?;
    if parts.next().is_some()
        || year.len() != 4
        || !(1..=2).contains(&month.len())
        || !(1..=2).contains(&day.len())
        || ![year, month, day]
            .iter()
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(invalid());
    }
    Date::new(
        year.parse().map_err(|_| invalid())?,
        month.parse().map_err(|_| invalid())?,
        day.parse().map_err(|_| invalid())?,
    )
    .map_err(|_| invalid())
}

#[cfg(test)]
mod tests;
