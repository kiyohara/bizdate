use super::*;
use crate::test_support::{FIXTURE, TempDir, now};

fn date(input: &str) -> Date {
    input.parse().unwrap()
}

fn with_expiration(expiration: &str) -> String {
    format!("# expires_at={expiration}\n{FIXTURE}")
}

#[test]
fn reads_fixture_and_uses_only_listed_dates() {
    let data = HolidayData::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/holidays.csv"),
        now(),
    )
    .unwrap();
    assert_eq!(data.dates.len(), 3);
    // 振替休日・国民の休日を名前に依存せず取り込む。
    for input in ["2026-01-01", "2026-05-06", "2026-09-22"] {
        assert!(data.is_holiday(date(input), now()).unwrap());
    }
    // 実際の祝日でも fixture に無い日付を再計算して追加しない。
    assert!(!data.is_holiday(date("2026-02-11"), now()).unwrap());
    assert!(!data.is_holiday(date("2026-01-02"), now()).unwrap());
}

#[test]
fn leading_metadata_is_skipped_but_csv_quotes_and_hashes_are_preserved() {
    let input =
        format!("# comment\n# future_key=value\n{FIXTURE}2026/12/1,\"# name, with\n# newline\"\n");
    for input in [input.clone(), input.replace('\n', "\r\n")] {
        let data = HolidayData::parse(&input, now()).unwrap();
        assert_eq!(data.dates.len(), 4);
        assert!(data.is_holiday(date("2026-12-01"), now()).unwrap());
    }
}

#[test]
fn metadata_prefix_identifies_only_the_schema_field() {
    let unmarked = FIXTURE.replace("# bizdate-meta schema=", "# schema=");
    assert!(matches!(
        HolidayData::parse(&unmarked, now()),
        Err(HolidayError::InvalidMetadata("schema"))
    ));
    for key in ["fetched_at", "source_url"] {
        let prefixed = FIXTURE.replace(&format!("# {key}="), &format!("# bizdate-meta {key}="));
        assert!(matches!(
            HolidayData::parse(&prefixed, now()),
            Err(HolidayError::InvalidMetadata(field)) if field == key
        ));
    }
    // 無関係な schema コメントや未知の prefix 付きフィールドを採用しない。
    let input = format!("# schema=2\n# bizdate-meta expires_at=invalid\n{FIXTURE}");
    let data = HolidayData::parse(&input, now()).unwrap();
    assert_eq!(
        data.expires_at,
        "2027-09-06T02:24:33Z".parse::<Timestamp>().unwrap()
    );
}

#[test]
fn expiration_boundary_is_exclusive_with_or_without_explicit_expiration() {
    for input in [FIXTURE.to_owned(), with_expiration("2027-09-06T02:24:33Z")] {
        for (instant, expired) in [
            ("2027-09-06T02:24:32.999999999Z", false),
            ("2027-09-06T02:24:33Z", true),
            ("2027-09-06T02:24:33.000000001Z", true),
        ] {
            let result = HolidayData::parse(&input, instant.parse().unwrap());
            if expired {
                assert!(matches!(result, Err(HolidayError::Expired)));
            } else {
                assert!(result.is_ok());
            }
        }
    }
}

#[test]
fn explicit_expiration_takes_precedence_over_anniversary() {
    for expiration in ["2026-10-01T00:00:00Z", "2028-10-01T00:00:00Z"] {
        let input = with_expiration(expiration);
        let expiration: Timestamp = expiration.parse().unwrap();
        assert!(
            HolidayData::parse(&input, expiration.checked_sub(1.nanosecond()).unwrap()).is_ok()
        );
        assert!(matches!(
            HolidayData::parse(&input, expiration),
            Err(HolidayError::Expired)
        ));
    }
}

#[test]
fn calendar_year_preserves_time_and_clamps_leap_day() {
    for (fetched, expiration) in [
        ("2024-02-29T12:34:56.123Z", "2025-02-28T12:34:56.123Z"),
        ("2023-03-01T00:00:00Z", "2024-03-01T00:00:00Z"),
    ] {
        let input = FIXTURE.replace("2026-09-06T02:24:33Z", fetched);
        let data = HolidayData::parse(&input, fetched.parse().unwrap()).unwrap();
        assert_eq!(data.expires_at, expiration.parse::<Timestamp>().unwrap());
        assert!(matches!(
            HolidayData::parse(&input, data.expires_at),
            Err(HolidayError::Expired)
        ));
    }
}

#[test]
fn already_loaded_data_cannot_be_queried_after_expiration_even_on_weekends() {
    let data = HolidayData::parse(FIXTURE, now()).unwrap();
    for input in ["2026-01-01", "2026-09-05", "2026-09-06"] {
        assert!(matches!(
            data.is_holiday(date(input), data.expires_at),
            Err(HolidayError::Expired)
        ));
    }
}

#[test]
fn rejects_years_outside_coverage_including_weekends() {
    let data = HolidayData::parse(FIXTURE, now()).unwrap();
    for input in ["2025-12-31", "2027-01-01", "2027-01-02"] {
        let error = data.is_holiday(date(input), now()).unwrap_err();
        assert!(matches!(
            error,
            HolidayError::YearOutOfRange {
                first: 2026,
                last: 2026,
                ..
            }
        ));
    }
}

#[test]
fn coverage_is_inclusive_minimum_and_maximum_year_regardless_of_row_order() {
    let input = format!("{FIXTURE}2028/1/1,late\n2024/1/1,early\n2026/1/1,duplicate\n");
    let data = HolidayData::parse(&input, now()).unwrap();
    assert_eq!(data.dates.len(), 5);
    for input in ["2024-12-31", "2025-01-01", "2028-12-31"] {
        assert!(!data.is_holiday(date(input), now()).unwrap());
    }
    assert!(data.is_holiday(date("2029-01-01"), now()).is_err());
}

#[test]
fn required_metadata_cannot_be_missing_empty_or_duplicated() {
    for key in ["schema", "fetched_at", "source_url"] {
        let prefix = if key == "schema" {
            "# bizdate-meta schema=".to_owned()
        } else {
            format!("# {key}=")
        };
        let original = FIXTURE
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap();
        for input in [
            FIXTURE.replace(&format!("{original}\n"), ""),
            FIXTURE.replace(original, &prefix),
            format!("{original}\n{FIXTURE}"),
        ] {
            assert!(HolidayData::parse(&input, now()).is_err(), "key={key}");
        }
    }
    assert!(matches!(
        HolidayData::parse(&FIXTURE.replace("schema=1", "schema=2"), now()),
        Err(HolidayError::UnsupportedSchema(_))
    ));
    assert!(matches!(
        HolidayData::parse(
            &format!(
                "# expires_at=2028-01-01T00:00:00Z\n{}",
                with_expiration("2029-01-01T00:00:00Z")
            ),
            now()
        ),
        Err(HolidayError::InvalidMetadata(_))
    ));
}

#[test]
fn timestamps_must_be_utc_rfc3339_even_when_expiration_is_explicit() {
    for value in [
        "",
        "invalid",
        "2026-09-06",
        "2026-09-06 02:24:33Z",
        "2026-09-06T11:24:33+09:00",
        "2026-09-06T02:24:33+0000",
        "2026-09-06T02:24:33Z[UTC]",
        "2026-09-06T02:24Z",
        "2026-02-30T02:24:33Z",
    ] {
        let input = with_expiration("2027-09-06T02:24:33Z").replace(
            "fetched_at=2026-09-06T02:24:33Z",
            &format!("fetched_at={value}"),
        );
        assert!(
            matches!(
                HolidayData::parse(&input, now()),
                Err(HolidayError::InvalidMetadata("fetched_at"))
            ),
            "{value}"
        );
        assert!(
            matches!(
                HolidayData::parse(&with_expiration(value), now()),
                Err(HolidayError::InvalidMetadata("expires_at"))
            ),
            "{value}"
        );
    }
    for value in [
        "2026-09-06T02:24:33+00:00",
        "2026-09-06t02:24:33z",
        "2026-09-06T02:24:33.123Z",
    ] {
        assert!(HolidayData::parse(&FIXTURE.replace("2026-09-06T02:24:33Z", value), now()).is_ok());
    }
}

#[test]
fn invalid_csv_and_dates_are_errors_not_partial_holiday_sets() {
    assert!(matches!(
        HolidayData::parse(&FIXTURE.replace(HEADER[0], "date"), now()),
        Err(HolidayError::InvalidHeader)
    ));
    let empty = FIXTURE.split("2026/1/1").next().unwrap();
    assert!(matches!(
        HolidayData::parse(empty, now()),
        Err(HolidayError::EmptyData)
    ));
    for row in ["2026/1/2", "2026/1/2,name,extra"] {
        assert!(matches!(
            HolidayData::parse(&format!("{FIXTURE}{row}\n"), now()),
            Err(HolidayError::Csv(_))
        ));
    }
    for input in [
        "2026/2/30",
        "2027/2/29",
        "2026-01-01",
        "2026/13/1",
        "2026/1/0",
        "26/1/1",
        "2026/001/1",
        "2026/1/1/2",
        "２０２６/1/1",
        " 2026/1/1",
    ] {
        assert!(
            matches!(
                HolidayData::parse(&format!("{FIXTURE}{input},bad\n"), now()),
                Err(HolidayError::InvalidDate(_))
            ),
            "{input}"
        );
    }
    assert_eq!(parse_csv_date("2028/02/29").unwrap(), date("2028-02-29"));
}

#[test]
fn filesystem_missing_invalid_utf8_and_expired_data_are_errors() {
    let dir = TempDir::new();
    let path = dir.0.join("holidays.csv");
    assert!(matches!(
        HolidayData::load(&path, now()),
        Err(HolidayError::MissingData)
    ));
    fs::write(&path, [0xff, 0xfe]).unwrap();
    let error = HolidayData::load(&path, now()).unwrap_err();
    assert!(matches!(error, HolidayError::InvalidEncoding(_)));
    assert!(error.source().is_some());
    // ディレクトリの読み取り失敗は文字コード不正と区別する。
    assert!(matches!(
        HolidayData::load(&dir.0, now()),
        Err(HolidayError::Io(_))
    ));
    fs::write(&path, FIXTURE).unwrap();
    assert!(matches!(
        HolidayData::load(&path, "2028-01-01T00:00:00Z".parse().unwrap()),
        Err(HolidayError::Expired)
    ));
    assert!(path.exists()); // 無効データを削除する副作用は持たない。
}

#[test]
fn data_home_takes_priority_and_unset_uses_home() {
    assert_eq!(
        data_path_with(|key| match key {
            "XDG_DATA_HOME" => Some("/data".into()),
            _ => panic!("HOME must not be read"),
        })
        .unwrap(),
        Path::new("/data/bizdate/holidays/holidays.csv")
    );
    assert_eq!(
        data_path_with(|key| match key {
            "XDG_DATA_HOME" => None,
            "HOME" => Some("/home/example".into()),
            _ => panic!("unexpected variable"),
        })
        .unwrap(),
        Path::new("/home/example/.local/share/bizdate/holidays/holidays.csv")
    );
}

#[test]
fn missing_or_invalid_base_paths_never_resolve_relative_to_current_directory() {
    for value in ["", "relative"] {
        assert!(matches!(
            data_path_with(|_| Some(value.into())),
            Err(HolidayError::InvalidDataHome)
        ));
        assert!(matches!(
            data_path_with(|key| (key == "HOME").then(|| value.into())),
            Err(HolidayError::InvalidHome)
        ));
    }
    assert!(matches!(
        data_path_with(|_| None),
        Err(HolidayError::InvalidHome)
    ));
}

#[cfg(unix)]
#[test]
fn paths_can_contain_non_unicode_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let base = OsString::from_vec(b"/data/\xff".to_vec());
    assert_eq!(
        data_path_with(|_| Some(base.clone())).unwrap(),
        PathBuf::from(base).join("bizdate/holidays/holidays.csv")
    );
}

#[test]
fn public_path_resolver_reads_process_environment() {
    const CHILD: &str = "BIZDATE_HOLIDAY_PATH_TEST_CHILD";
    if let Some(expected) = env::var_os(CHILD) {
        assert_eq!(data_path().unwrap(), PathBuf::from(expected));
        return;
    }
    // 関数参照から名前を得て rename に追従する。libtest の名前は crate 名を含まない。
    let full_name = std::any::type_name_of_val(&public_path_resolver_reads_process_environment);
    let (_, filter) = full_name.split_once("::").unwrap();
    for (xdg, expected) in [
        (Some("/data"), "/data/bizdate/holidays/holidays.csv"),
        (
            None,
            "/home/example/.local/share/bizdate/holidays/holidays.csv",
        ),
    ] {
        let mut command = std::process::Command::new(env::current_exe().unwrap());
        command
            .args(["--exact", filter])
            .env(CHILD, expected)
            .env("HOME", "/home/example")
            .env_remove("XDG_DATA_HOME");
        if let Some(xdg) = xdg {
            command.env("XDG_DATA_HOME", xdg);
        }
        let output = command.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{filter}: {output:?}");
        assert!(
            stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "{filter}: {stdout}"
        );
    }
}
