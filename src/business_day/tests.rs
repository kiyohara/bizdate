use super::*;
use crate::{
    date::{resolve_date, today},
    test_support::{FIXTURE, TempDir, now},
};
use jiff::tz::TimeZone;
use std::{fs, path::Path};

fn date(input: &str) -> Date {
    parse_date(input).unwrap()
}

fn fixture() -> HolidayData {
    HolidayData::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/holidays.csv"),
        now(),
    )
    .unwrap()
}

// #10 の fixture に人工的な境界ケースを足し、公開の読み取り API を通す。
fn extended_fixture(rows: &str) -> HolidayData {
    let dir = TempDir::new();
    let path = dir.0.join("holidays.csv");
    fs::write(&path, format!("{FIXTURE}{rows}")).unwrap();
    HolidayData::load(&path, now()).unwrap()
}

#[test]
fn weekdays_are_business_days_and_weekends_are_not() {
    let holidays = fixture();
    let days_off = DaysOff::default();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for (input, expected) in [
        ("2026-09-07", true),
        ("2026-09-08", true),
        ("2026-09-09", true),
        ("2026-09-10", true),
        ("2026-09-11", true),
        ("2026-09-12", false),
        ("2026-09-13", false),
    ] {
        assert_eq!(
            calendar.is_business_day(date(input)).unwrap(),
            expected,
            "{input}"
        );
    }
}

#[test]
fn listed_holidays_and_user_days_off_are_not_business_days() {
    let holidays = fixture();
    let days_off = DaysOff::parse("09-07", date("2026-09-06")).unwrap();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    // 元日、振替休日、国民の休日と利用者指定休日。
    for input in ["2026-01-01", "2026-05-06", "2026-09-22", "2026-09-07"] {
        assert!(!calendar.is_business_day(date(input)).unwrap(), "{input}");
    }
    // fixture に無い日付の祝日を自前で補わない。
    assert!(calendar.is_business_day(date("2026-02-11")).unwrap());
}

#[test]
fn parses_mixed_dates_and_collapses_duplicates() {
    let days_off = DaysOff::parse("2027-01-02,01-03,2026-01-03,01-03", date("2026-09-06")).unwrap();
    assert_eq!(days_off.0.len(), 2);
    assert!(days_off.contains(date("2027-01-02")));
    assert!(days_off.contains(date("2026-01-03")));
    assert!(!days_off.contains(date("2027-01-03")));
    assert!(!DaysOff::default().contains(date("2026-01-03")));
}

#[test]
fn omitted_year_uses_timezone_today_instead_of_target_year() {
    let instant = "2026-12-31T15:00:00Z".parse().unwrap();
    for (zone, expected, excluded) in [
        (TimeZone::UTC, "2026-01-02", "2027-01-02"),
        (
            TimeZone::get("Asia/Tokyo").unwrap(),
            "2027-01-02",
            "2026-01-02",
        ),
    ] {
        let target = resolve_date(Some("2028-01-02"), &zone, instant).unwrap();
        let days_off = DaysOff::parse("01-02,2025-12-31", today(&zone, instant)).unwrap();
        assert!(days_off.contains(date(expected)));
        assert!(days_off.contains(date("2025-12-31")));
        assert!(!days_off.contains(date(excluded)));
        assert!(!days_off.contains(target));
    }
}

#[test]
fn leap_day_depends_on_the_resolved_year() {
    assert!(
        DaysOff::parse("02-29", date("2028-01-01"))
            .unwrap()
            .contains(date("2028-02-29"))
    );
    assert!(DaysOff::parse("02-29", date("2026-01-01")).is_err());
    assert!(DaysOff::parse("2028-02-29", date("2026-01-01")).is_ok());
}

#[test]
fn rejects_invalid_elements_without_trimming_or_ignoring_them() {
    for input in [
        "",
        ",01-01",
        "2026-01-01,,2026-01-02",
        "2026-01-01,",
        "2026-1-01",
        "1-01",
        "01-1",
        "2026/01/01",
        "01/01",
        "0101",
        "02-30",
        "00-01",
        "13-01",
        "01-00",
        "99-99",
        "2026-02-29",
        "2026-01-01..2026-01-03",
        " 01-01",
        "01-01 ",
        "01-01\n",
        "2026-01-01, 01-02",
        "０１-０１",
        "0é-1",
        "2026-01-01T00:00:00Z",
    ] {
        assert!(
            DaysOff::parse(input, date("2026-01-01")).is_err(),
            "{input:?}"
        );
    }
}

#[test]
fn parse_error_retains_the_original_element_and_cause() {
    let error = DaysOff::parse("01-01,02-30", date("2026-01-01")).unwrap_err();
    assert_eq!(error.input, "02-30");
    assert!(matches!(error.source, DateError::InvalidDate(_)));
    assert!(error.source().is_some());
    assert!(error.to_string().contains("02-30"));
    let error = DaysOff::parse("01-01,", date("2026-01-01")).unwrap_err();
    assert_eq!(error.input, "");
    assert!(matches!(error.source, DateError::InvalidFormat(_)));
}

#[test]
fn month_boundaries_skip_weekends_and_listed_holidays() {
    let holidays = extended_fixture("2026/7/31,人工的な月末休日\n");
    let days_off = DaysOff::default();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for (target, first, last) in [
        ("2026-01-01", "2026-01-02", "2026-01-30"),
        ("2026-02-28", "2026-02-02", "2026-02-27"),
        ("2026-07-15", "2026-07-01", "2026-07-30"),
        ("2026-08-15", "2026-08-03", "2026-08-31"),
        ("2026-12-31", "2026-12-01", "2026-12-31"),
    ] {
        assert_eq!(
            calendar.first_business_day(date(target)).unwrap(),
            Some(date(first)),
            "{target}"
        );
        assert_eq!(
            calendar.last_business_day(date(target)).unwrap(),
            Some(date(last)),
            "{target}"
        );
    }
}

#[test]
fn user_days_off_move_both_month_boundaries() {
    let holidays = fixture();
    let days_off = DaysOff::parse("01-02,01-30", date("2026-01-01")).unwrap();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    assert_eq!(
        calendar.first_business_day(date("2026-01-15")).unwrap(),
        Some(date("2026-01-05"))
    );
    assert_eq!(
        calendar.last_business_day(date("2026-01-15")).unwrap(),
        Some(date("2026-01-29"))
    );
}

fn january_days_off_except(remaining: Option<i8>) -> DaysOff {
    let input = (1..=31)
        .filter(|day| Some(*day) != remaining)
        .map(|day| format!("2026-01-{day:02}"))
        .collect::<Vec<_>>()
        .join(",");
    DaysOff::parse(&input, date("2026-01-01")).unwrap()
}

#[test]
fn sole_business_day_is_both_first_and_last() {
    let holidays = fixture();
    let days_off = january_days_off_except(Some(15));
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    let sole = date("2026-01-15");
    assert!(calendar.is_business_day(sole).unwrap());
    assert_eq!(calendar.first_business_day(sole).unwrap(), Some(sole));
    assert_eq!(calendar.last_business_day(sole).unwrap(), Some(sole));
}

#[test]
fn month_with_no_business_days_has_neither_boundary() {
    let holidays = fixture();
    let days_off = january_days_off_except(None);
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for day in 1..=31 {
        let target = Date::new(2026, 1, day).unwrap();
        assert!(!calendar.is_business_day(target).unwrap());
        assert_eq!(calendar.first_business_day(target).unwrap(), None);
        assert_eq!(calendar.last_business_day(target).unwrap(), None);
    }
}

#[test]
fn year_end_and_summer_breaks_are_not_built_in() {
    let holidays = extended_fixture("2025/1/1,年カバー用\n2027/1/1,元日\n");
    let days_off = DaysOff::default();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for input in [
        "2025-01-03",
        "2025-12-29",
        "2025-12-30",
        "2025-12-31",
        "2026-01-02",
        "2026-08-13",
        "2026-08-14",
    ] {
        assert!(calendar.is_business_day(date(input)).unwrap(), "{input}");
    }
    assert!(!calendar.is_business_day(date("2026-01-01")).unwrap());
}

#[test]
fn leap_month_and_maximum_year_stay_within_the_month() {
    let holidays = extended_fixture("9999/1/1,年カバー用\n");
    let days_off = DaysOff::default();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for (target, first, last) in [
        ("2028-02-15", "2028-02-01", "2028-02-29"),
        ("9999-12-31", "9999-12-01", "9999-12-31"),
    ] {
        assert_eq!(
            calendar.first_business_day(date(target)).unwrap(),
            Some(date(first))
        );
        assert_eq!(
            calendar.last_business_day(date(target)).unwrap(),
            Some(date(last))
        );
    }
}

#[test]
fn expired_data_errors_even_on_weekends_days_off_and_empty_months() {
    let holidays = fixture();
    let days_off = january_days_off_except(None);
    let calendar = BusinessCalendar::new(
        &holidays,
        &days_off,
        "2027-09-06T02:24:33Z".parse().unwrap(),
    );
    for input in ["2026-01-01", "2026-01-02", "2026-01-03", "2026-01-04"] {
        let target = date(input);
        assert!(matches!(
            calendar.is_business_day(target),
            Err(HolidayError::Expired)
        ));
        assert!(matches!(
            calendar.first_business_day(target),
            Err(HolidayError::Expired)
        ));
        assert!(matches!(
            calendar.last_business_day(target),
            Err(HolidayError::Expired)
        ));
    }
}

#[test]
fn uncovered_year_errors_even_on_weekends_and_days_off() {
    let holidays = fixture();
    let days_off = DaysOff::parse("2027-01-01,2027-01-02,2027-01-03", date("2026-01-01")).unwrap();
    let calendar = BusinessCalendar::new(&holidays, &days_off, now());
    for input in ["2027-01-01", "2027-01-02", "2027-01-03"] {
        let target = date(input);
        for result in [
            calendar.is_business_day(target).map(Some),
            calendar.first_business_day(target).map(|d| d.map(|_| true)),
            calendar.last_business_day(target).map(|d| d.map(|_| true)),
        ] {
            assert!(matches!(
                result,
                Err(HolidayError::YearOutOfRange { year: 2027, .. })
            ));
        }
    }
}
