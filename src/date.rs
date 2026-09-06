//! 日付入力とタイムゾーンを解決する。CLI の option / exit code は扱わない。

use std::{env, error::Error, fmt};

use jiff::{Timestamp, civil::Date, tz::TimeZone};

#[derive(Debug)]
pub enum DateError {
    InvalidFormat(String),
    InvalidDate(jiff::Error),
    UnknownTimeZone(String),
    NonUnicodeTimeZone,
    LocalTimeZone(jiff::Error),
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(input) => {
                write!(f, "date must have the format YYYY-MM-DD: {input:?}")
            }
            Self::InvalidDate(error) => write!(f, "invalid calendar date: {error}"),
            Self::UnknownTimeZone(name) => write!(f, "unknown IANA time zone: {name:?}"),
            Self::NonUnicodeTimeZone => write!(f, "BIZDATE_TZ must be a Unicode IANA name"),
            Self::LocalTimeZone(error) => write!(f, "cannot resolve local time zone: {error}"),
        }
    }
}

impl Error for DateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDate(error) | Self::LocalTimeZone(error) => Some(error),
            Self::InvalidFormat(_) | Self::UnknownTimeZone(_) | Self::NonUnicodeTimeZone => None,
        }
    }
}

/// ASCII の `YYYY-MM-DD` だけを受け付け、実在する暦日か検証する。
pub fn parse_date(input: &str) -> Result<Date, DateError> {
    let bytes = input.as_bytes();
    if bytes.len() != 10
        || !bytes.iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
    {
        return Err(DateError::InvalidFormat(input.to_owned()));
    }
    input.parse().map_err(DateError::InvalidDate)
}

/// CLI 指定、`BIZDATE_TZ`、local timezone の順で解決する。
///
/// `TZ` はここで専用入力として読まない。local timezone の解釈は
/// Jiff の通常規則に委ねるため、その段階では `TZ` が効くことがある。
pub fn resolve_timezone(cli_timezone: Option<&str>) -> Result<TimeZone, DateError> {
    resolve_timezone_with(cli_timezone, |key| env::var(key), TimeZone::try_system)
}

fn resolve_timezone_with(
    cli_timezone: Option<&str>,
    read_env: impl FnOnce(&str) -> Result<String, env::VarError>,
    local_timezone: impl FnOnce() -> Result<TimeZone, jiff::Error>,
) -> Result<TimeZone, DateError> {
    if let Some(name) = cli_timezone {
        return parse_timezone(name);
    }
    match read_env("BIZDATE_TZ") {
        Ok(name) => parse_timezone(&name),
        Err(env::VarError::NotPresent) => local_timezone().map_err(DateError::LocalTimeZone),
        Err(env::VarError::NotUnicode(_)) => Err(DateError::NonUnicodeTimeZone),
    }
}

fn parse_timezone(name: &str) -> Result<TimeZone, DateError> {
    let timezone = TimeZone::get(name).map_err(|_| DateError::UnknownTimeZone(name.to_owned()))?;
    // Jiff の Etc/Unknown は UTC と同様に動く sentinel であり、IANA 名ではない。
    if timezone.is_unknown() {
        return Err(DateError::UnknownTimeZone(name.to_owned()));
    }
    Ok(timezone)
}

/// 採用タイムゾーンの「今日」。呼び出し元で一度取得した現在時刻を渡す。
/// `--day-off` の年省略も、判定対象日の年ではなくこの暦年を使う。
pub fn today(timezone: &TimeZone, now: Timestamp) -> Date {
    timezone.to_datetime(now).date()
}

/// 指定日を暦日として解釈する。省略時だけ採用タイムゾーンの「今日」を使う。
pub fn resolve_date(
    input: Option<&str>,
    timezone: &TimeZone,
    now: Timestamp,
) -> Result<Date, DateError> {
    match input {
        Some(input) => parse_date(input),
        None => Ok(today(timezone, now)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant() -> Timestamp {
        "2026-12-31T15:00:00Z".parse().unwrap()
    }

    fn no_local() -> Result<TimeZone, jiff::Error> {
        panic!("local timezone must not be consulted")
    }

    #[test]
    fn cli_takes_priority_over_environment() {
        let timezone = resolve_timezone_with(
            Some("Asia/Tokyo"),
            |_| Ok("America/New_York".into()),
            no_local,
        )
        .unwrap();
        assert_eq!(today(&timezone, instant()), Date::new(2027, 1, 1).unwrap());
    }

    #[test]
    fn environment_takes_priority_over_local() {
        let timezone = resolve_timezone_with(
            None,
            |key| {
                assert_eq!(key, "BIZDATE_TZ");
                Ok("Asia/Tokyo".into())
            },
            no_local,
        )
        .unwrap();
        assert_eq!(today(&timezone, instant()), Date::new(2027, 1, 1).unwrap());
    }

    #[test]
    fn absent_overrides_use_local_without_reading_tz_as_an_override() {
        let timezone = resolve_timezone_with(
            None,
            |key| match key {
                "BIZDATE_TZ" => Err(env::VarError::NotPresent),
                "TZ" => Ok("Asia/Tokyo".into()),
                _ => panic!("unexpected environment lookup"),
            },
            || Ok(TimeZone::UTC),
        )
        .unwrap();
        assert_eq!(
            today(&timezone, instant()),
            Date::new(2026, 12, 31).unwrap()
        );
    }

    #[test]
    fn cli_does_not_read_lower_priority_environment() {
        resolve_timezone_with(
            Some("UTC"),
            |_| panic!("must not read environment"),
            no_local,
        )
        .unwrap();
    }

    #[test]
    fn invalid_overrides_do_not_fall_back() {
        for name in [
            "Unknown/Zone",
            "Etc/Unknown",
            "",
            "+09:00",
            " UTC ",
            "/etc/localtime",
            "EST5EDT,M3.2.0,M11.1.0",
        ] {
            assert!(matches!(
                resolve_timezone_with(Some(name), |_| panic!("must not fall back"), no_local),
                Err(DateError::UnknownTimeZone(_))
            ));
            assert!(matches!(
                resolve_timezone_with(None, |_| Ok(name.into()), no_local),
                Err(DateError::UnknownTimeZone(_))
            ));
        }
    }

    #[test]
    fn iana_names_are_ascii_case_insensitive() {
        for name in ["asia/tokyo", "ASIA/TOKYO"] {
            let timezone =
                resolve_timezone_with(Some(name), |_| panic!("unused"), no_local).unwrap();
            assert_eq!(today(&timezone, instant()), Date::new(2027, 1, 1).unwrap());
        }
        let timezone = resolve_timezone_with(None, |_| Ok("utc".into()), no_local).unwrap();
        assert_eq!(
            today(&timezone, instant()),
            Date::new(2026, 12, 31).unwrap()
        );
    }

    #[test]
    fn environment_and_local_errors_are_typed() {
        assert!(matches!(
            resolve_timezone_with(
                None,
                |_| Err(env::VarError::NotUnicode(Default::default())),
                no_local,
            ),
            Err(DateError::NonUnicodeTimeZone)
        ));
        let error = resolve_timezone_with(
            None,
            |_| Err(env::VarError::NotPresent),
            || TimeZone::get("Unknown/Zone"),
        )
        .unwrap_err();
        assert!(matches!(error, DateError::LocalTimeZone(_)));
        assert!(error.source().is_some());
        assert!(error.to_string().contains("local time zone"));
    }

    #[test]
    fn today_crosses_day_month_and_year_boundaries() {
        let tokyo = TimeZone::get("Asia/Tokyo").unwrap();
        for (now, utc, japan) in [
            ("2026-01-31T14:59:59Z", "2026-01-31", "2026-01-31"),
            ("2026-01-31T15:00:00Z", "2026-01-31", "2026-02-01"),
            ("2026-12-31T15:00:00Z", "2026-12-31", "2027-01-01"),
        ] {
            let now = now.parse().unwrap();
            assert_eq!(
                resolve_date(None, &TimeZone::UTC, now).unwrap(),
                parse_date(utc).unwrap()
            );
            assert_eq!(
                resolve_date(None, &tokyo, now).unwrap(),
                parse_date(japan).unwrap()
            );
        }
    }

    #[test]
    fn explicit_date_is_independent_of_today_and_timezone() {
        for timezone in [TimeZone::UTC, TimeZone::get("Asia/Tokyo").unwrap()] {
            let date = resolve_date(Some("2028-02-29"), &timezone, instant()).unwrap();
            assert_eq!(date, Date::new(2028, 2, 29).unwrap());
            assert_ne!(date.year(), today(&timezone, instant()).year());
            assert!(resolve_date(Some(""), &timezone, instant()).is_err());
        }
    }

    #[test]
    fn calendar_month_is_the_specified_dates_month() {
        for (input, first, last) in [
            ("2026-01-31", "2026-01-01", "2026-01-31"),
            ("2026-02-01", "2026-02-01", "2026-02-28"),
            ("2028-02-29", "2028-02-01", "2028-02-29"),
            ("2026-12-31", "2026-12-01", "2026-12-31"),
        ] {
            let date = parse_date(input).unwrap();
            assert_eq!(date.first_of_month(), parse_date(first).unwrap());
            assert_eq!(date.last_of_month(), parse_date(last).unwrap());
        }
    }

    #[test]
    fn rejects_nonexistent_dates() {
        for input in [
            "2026-02-30",
            "2027-02-29",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-04-31",
        ] {
            assert!(
                matches!(parse_date(input), Err(DateError::InvalidDate(_))),
                "{input}"
            );
        }
    }

    #[test]
    fn rejects_other_date_formats() {
        for input in [
            "2026-1-1",
            "20260101",
            "",
            " 2026-01-01",
            "2026-01-01\n",
            "2026/01/01",
            "2026-01-01T00:00:00Z",
            "+002026-01-01",
            "２０２６-01-01",
            "2026-0é-1",
        ] {
            let error = parse_date(input).unwrap_err();
            assert!(matches!(&error, DateError::InvalidFormat(value) if value == input));
            assert_eq!(
                error.to_string(),
                format!("date must have the format YYYY-MM-DD: {input:?}")
            );
        }
    }

    // 子プロセスに環境を渡し、並列テスト中のプロセス環境変更を避ける。
    #[test]
    fn process_tz_is_used_only_by_local_timezone_resolution() {
        const CHILD: &str = "BIZDATE_DATE_TEST_CHILD";
        if env::var_os(CHILD).is_some() {
            let timezone = resolve_timezone(None).unwrap();
            let expected = env::var("BIZDATE_DATE_TEST_EXPECTED").unwrap();
            assert_eq!(today(&timezone, instant()), parse_date(&expected).unwrap());
            let cli = resolve_timezone(Some("UTC")).unwrap();
            assert_eq!(today(&cli, instant()), parse_date("2026-12-31").unwrap());
            return;
        }
        for (bizdate_tz, expected) in [(None, "2027-01-01"), (Some("UTC"), "2026-12-31")] {
            let mut child = std::process::Command::new(env::current_exe().unwrap());
            child
                .args([
                    "--exact",
                    "date::tests::process_tz_is_used_only_by_local_timezone_resolution",
                ])
                .env(CHILD, "1")
                .env("BIZDATE_DATE_TEST_EXPECTED", expected)
                .env("TZ", "Asia/Tokyo")
                .env_remove("BIZDATE_TZ");
            if let Some(name) = bizdate_tz {
                child.env("BIZDATE_TZ", name);
            }
            assert!(child.status().unwrap().success());
        }
    }
}
