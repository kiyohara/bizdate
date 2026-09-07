use super::*;
use crate::test_support::{TempDir, now};
use clap::error::ErrorKind;
use std::path::Path;

fn fixture() -> Result<PathBuf, HolidayError> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/holidays.csv"))
}

fn args(date: &str) -> Judgment {
    Judgment {
        date: Some(date.to_owned()),
        timezone: Some("Asia/Tokyo".to_owned()),
        day_off: None,
        quiet: false,
    }
}

/// 1 回の実行を stdout / stderr / exit code の 3 つで観測する。
fn observe(edge: Edge, args: &Judgment) -> (u8, String, String) {
    observe_with(edge, args, fixture)
}

fn observe_with(
    edge: Edge,
    args: &Judgment,
    data_path: impl FnOnce() -> Result<PathBuf, HolidayError>,
) -> (u8, String, String) {
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let result = decide(edge, args, now(), data_path)
        .and_then(|matched| report(matched, args.quiet, &mut stdout));
    let code = finish(result, &mut stderr);
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

fn parse_error(command: &[&str]) -> clap::Error {
    Cli::try_parse_from(command).unwrap_err()
}

/// argv の parse から exit code までを `execute` ごと通す。
fn observe_argv(argv: &[&str]) -> (u8, String, String) {
    observe_argv_with(argv, fixture)
}

fn observe_argv_with(
    argv: &[&str],
    data_path: impl FnOnce() -> Result<PathBuf, HolidayError>,
) -> (u8, String, String) {
    let cli = Cli::try_parse_from(argv).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let result = execute(&cli.command, now(), data_path, download, &mut stdout);
    let code = finish(result, &mut stderr);
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

/// 取得の実体を差し替える。判定サブコマンドではそもそも呼ばれない。
fn download(source: &str) -> Result<Payload, FetchError> {
    if source.contains("unreachable") {
        return Err(FetchError::Request(Box::new(io::Error::from(
            io::ErrorKind::ConnectionRefused,
        ))));
    }
    Ok(Payload {
        charset: None,
        body: SOURCE_BODY.as_bytes().to_vec(),
    })
}

/// 取得元が返す本文。メタ行は保存側が付ける。
const SOURCE_BODY: &str = "国民の祝日・休日月日,国民の祝日・休日名称\r\n2026/1/1,元日\r\n";

#[test]
fn matching_days_exit_zero_and_others_exit_one() {
    // 2026-09 は火曜始まりで月末が水曜。2026-01 は元日と土日で両端がずれる。
    for (edge, date, expected) in [
        (Edge::First, "2026-09-01", YES),
        (Edge::First, "2026-09-02", NO),
        (Edge::Last, "2026-09-30", YES),
        (Edge::Last, "2026-09-29", NO),
        (Edge::First, "2026-01-01", NO),
        (Edge::First, "2026-01-02", YES),
        (Edge::Last, "2026-01-30", YES),
        (Edge::Last, "2026-01-31", NO),
    ] {
        let (code, stdout, stderr) = observe(edge, &args(date));
        assert_eq!(code, expected, "{edge:?} {date}");
        assert_eq!(
            stdout,
            if expected == YES { "yes\n" } else { "no\n" },
            "{edge:?} {date}"
        );
        assert!(stderr.is_empty(), "{edge:?} {date}: {stderr}");
    }
}

#[test]
fn stdout_carries_only_the_judgment_and_diagnostics_go_to_stderr() {
    let (code, stdout, stderr) = observe(Edge::First, &args("2026-09-01"));
    assert_eq!((code, stdout.lines().count()), (YES, 1));
    assert!(stderr.is_empty());

    let (code, stdout, stderr) = observe(Edge::First, &args("2026-02-30"));
    assert_eq!(code, ERROR);
    // エラー時に stdout へ `no` を出さない。
    assert!(stdout.is_empty());
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.starts_with("bizdate: "), "{stderr}");
}

#[test]
fn quiet_silences_stdout_without_changing_the_exit_code() {
    for (date, expected) in [("2026-09-01", YES), ("2026-09-02", NO)] {
        let loud = observe(Edge::First, &args(date));
        let quiet = observe(
            Edge::First,
            &Judgment {
                quiet: true,
                ..args(date)
            },
        );
        assert_eq!((loud.0, quiet.0), (expected, expected), "{date}");
        assert!(!loud.1.is_empty(), "{date}");
        assert!(quiet.1.is_empty(), "{date}");
    }
}

#[test]
fn invalid_input_is_an_error_and_never_a_negative_judgment() {
    for judgment in [
        // 不正な日付。
        Judgment {
            date: Some("2026-02-30".to_owned()),
            ..args("2026-09-01")
        },
        Judgment {
            date: Some("2026/09/01".to_owned()),
            ..args("2026-09-01")
        },
        // 未知のタイムゾーンと空文字。
        Judgment {
            timezone: Some("Invalid/Zone".to_owned()),
            ..args("2026-09-01")
        },
        Judgment {
            timezone: Some(String::new()),
            ..args("2026-09-01")
        },
        // 不正な `--day-off`。形式不正、存在しない日、空要素。
        Judgment {
            day_off: Some("2026-9-1".to_owned()),
            ..args("2026-09-01")
        },
        Judgment {
            day_off: Some("02-30".to_owned()),
            ..args("2026-09-01")
        },
        Judgment {
            day_off: Some("2026-09-01,".to_owned()),
            ..args("2026-09-01")
        },
        // 祝日データの年カバー外。
        Judgment {
            date: Some("2030-09-02".to_owned()),
            ..args("2026-09-01")
        },
    ] {
        let (code, stdout, stderr) = observe(Edge::First, &judgment);
        assert_eq!(code, ERROR, "{judgment:?}");
        assert!(stdout.is_empty(), "{judgment:?}");
        assert!(!stderr.is_empty(), "{judgment:?}");
    }
}

#[test]
fn missing_holiday_data_is_an_error_rather_than_a_negative_judgment() {
    // 祝日データを置いていない保存先へ向ける。同じ日付は fixture があれば yes になる。
    let dir = TempDir::new();
    let path = dir.0.join("holidays.csv");
    let (code, stdout, stderr) = observe_with(Edge::Last, &args("2026-09-30"), || Ok(path));
    assert_eq!(observe(Edge::Last, &args("2026-09-30")).0, YES);
    assert_eq!(code, ERROR);
    assert!(stdout.is_empty());
    assert!(stderr.contains("holiday data"), "{stderr}");
}

#[test]
fn user_days_off_move_the_edges_including_the_month_day_form() {
    let day_off = |list: &str| Judgment {
        day_off: Some(list.to_owned()),
        ..args("2026-09-02")
    };
    // 月初を休みにすると、翌営業日が最初の業務日になる。
    assert_eq!(observe(Edge::First, &day_off("2026-09-01")).0, YES);
    // 年省略は採用タイムゾーンの今日の年で補う。now() は Asia/Tokyo で 2026-09-06。
    assert_eq!(observe(Edge::First, &day_off("09-01")).0, YES);
    // 指定しなければ 2026-09-02 は最初の業務日ではない。
    assert_eq!(observe(Edge::First, &args("2026-09-02")).0, NO);
}

#[test]
fn the_same_day_can_be_both_edges_and_a_month_can_have_none() {
    let until_29 = (1..=29)
        .map(|day| format!("2026-09-{day:02}"))
        .collect::<Vec<_>>()
        .join(",");
    // 30 日以外をすべて休みにすると、業務日は 2026-09-30 の 1 日だけになる。
    let only_last = Judgment {
        day_off: Some(until_29.clone()),
        ..args("2026-09-30")
    };
    assert_eq!(observe(Edge::First, &only_last).0, YES);
    assert_eq!(observe(Edge::Last, &only_last).0, YES);

    // 全日を休みにすると業務日が 0 日になる。どの日も該当しないが、エラーではない。
    let none = Judgment {
        day_off: Some(format!("{until_29},2026-09-30")),
        ..args("2026-09-30")
    };
    assert_eq!(observe(Edge::First, &none).0, NO);
    assert_eq!(observe(Edge::Last, &none).0, NO);
}

#[test]
fn subcommand_names_select_the_matching_month_edge() {
    // first と last の取り違えは判定を反転させるため、subcommand 名から exit code まで通す。
    for (name, date, expected, line) in [
        ("first", "2026-09-01", YES, "yes\n"),
        ("first", "2026-09-30", NO, "no\n"),
        ("last", "2026-09-30", YES, "yes\n"),
        ("last", "2026-09-01", NO, "no\n"),
    ] {
        let (code, stdout, stderr) =
            observe_argv(&["bizdate", name, "--timezone", "Asia/Tokyo", "--date", date]);
        assert_eq!((code, stdout.as_str()), (expected, line), "{name} {date}");
        assert!(stderr.is_empty(), "{name} {date}: {stderr}");
    }

    // エラーと `--quiet` も subcommand 経由で同じ写像になる。
    let (code, stdout, stderr) = observe_argv(&[
        "bizdate",
        "last",
        "--date",
        "2026-02-30",
        "--timezone",
        "UTC",
    ]);
    assert_eq!(code, ERROR);
    assert!(stdout.is_empty());
    assert!(stderr.starts_with("bizdate: "), "{stderr}");
    let (code, stdout, _) = observe_argv(&[
        "bizdate",
        "last",
        "--date",
        "2026-09-30",
        "--timezone",
        "Asia/Tokyo",
        "--quiet",
    ]);
    assert_eq!((code, stdout.as_str()), (YES, ""));
}

#[test]
fn long_options_are_wired_to_both_subcommands() {
    for (name, edge) in [("first", Edge::First), ("last", Edge::Last)] {
        let cli = Cli::try_parse_from([
            "bizdate",
            name,
            "--date",
            "2026-09-01",
            "--timezone",
            "Asia/Tokyo",
            "--day-off",
            "09-22",
            "--quiet",
        ])
        .unwrap();
        let args = match (&cli.command, edge) {
            (Command::First(args), Edge::First) | (Command::Last(args), Edge::Last) => args,
            _ => panic!("{name} parsed as {:?}", cli.command),
        };
        assert_eq!(args.date.as_deref(), Some("2026-09-01"));
        assert_eq!(args.timezone.as_deref(), Some("Asia/Tokyo"));
        assert_eq!(args.day_off.as_deref(), Some("09-22"));
        assert!(args.quiet);
    }

    // option 省略時は各 API の既定へ委ねる。
    let cli = Cli::try_parse_from(["bizdate", "first"]).unwrap();
    let Command::First(args) = &cli.command else {
        panic!("{:?}", cli.command)
    };
    assert_eq!(
        (&args.date, &args.timezone, &args.day_off),
        (&None, &None, &None)
    );
    assert!(!args.quiet);
}

#[test]
fn help_and_version_exit_zero_and_usage_errors_exit_two() {
    for command in [
        vec!["bizdate", "--help"],
        vec!["bizdate", "--version"],
        vec!["bizdate", "first", "--help"],
        vec!["bizdate", "last", "--help"],
        vec!["bizdate", "fetch-holidays", "--help"],
    ] {
        let error = parse_error(&command);
        assert_eq!(error.exit_code(), 0, "{command:?}");
        assert!(
            matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ),
            "{command:?}: {:?}",
            error.kind()
        );
    }

    for command in [
        // サブコマンド欠落、未知の option、未知のサブコマンド、短 option。
        vec!["bizdate"],
        vec!["bizdate", "--unknown"],
        vec!["bizdate", "first", "--unknown"],
        vec!["bizdate", "neither"],
        vec!["bizdate", "-h"],
        vec!["bizdate", "-V"],
        vec!["bizdate", "first", "-q"],
        // --version は root だけで受ける。判定の option は取得サブコマンドに無い。
        vec!["bizdate", "fetch-holidays", "--version"],
        vec!["bizdate", "fetch-holidays", "--date", "2026-09-01"],
        vec!["bizdate", "fetch-holidays", "--quiet"],
        vec!["bizdate", "fetch-holidays", "--source"],
        // 取得の option は判定サブコマンドに無い。
        vec!["bizdate", "first", "--source", "https://example.com/h.csv"],
        // 提供しない positional 日付と、値を要求する option の値欠落。
        vec!["bizdate", "first", "2026-09-01"],
        vec!["bizdate", "first", "--date"],
        // clap 既定の help subcommand は出さない。
        vec!["bizdate", "help"],
        vec!["bizdate", "help", "first"],
    ] {
        let error = parse_error(&command);
        assert_eq!(error.exit_code(), i32::from(ERROR), "{command:?}");
        assert!(
            !matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ),
            "{command:?}: {:?}",
            error.kind()
        );
        // 値欠落だけは clap が usage を省き、`--help` へ誘導する。
        let rendered = error.to_string();
        assert!(
            rendered.contains("Usage:") || rendered.contains("a value is required"),
            "{command:?}\n{rendered}"
        );
    }
}

#[test]
fn help_describes_the_command_and_every_option_in_japanese() {
    let root = parse_error(&["bizdate", "--help"]).to_string();
    for text in [
        "その日が月の最初 / 最後の業務日かどうかを判定する",
        "指定日がその月の最初の業務日かどうかを判定する",
        "指定日がその月の最後の業務日かどうかを判定する",
        "usage を表示して終了する",
        "version を表示して終了する",
    ] {
        assert!(root.contains(text), "{text}\n{root}");
    }

    assert!(
        root.contains("祝日データを取得してローカルへ保存する"),
        "{root}"
    );

    let fetch = parse_error(&["bizdate", "fetch-holidays", "--help"]).to_string();
    for text in ["祝日データの取得先", "usage を表示して終了する"] {
        assert!(fetch.contains(text), "{text}\n{fetch}");
    }
    // 既定の取得先を help で示す。
    assert!(fetch.contains(fetch::DEFAULT_SOURCE), "{fetch}");

    let first = parse_error(&["bizdate", "first", "--help"]).to_string();
    for text in [
        "判定対象日。既定は採用タイムゾーンの今日",
        "「今日」と年省略の基準タイムゾーン",
        "追加の休日。カンマ区切りの YYYY-MM-DD または MM-DD",
        "stdout へ判定結果を出さない",
        "usage を表示して終了する",
    ] {
        assert!(first.contains(text), "{text}\n{first}");
    }
    // 短 option と、英語説明の help subcommand を案内しない。
    for text in ["-d,", "-t,", "-q,", "-h,"] {
        assert!(!first.contains(text), "{text}\n{first}");
    }
    assert!(!root.contains("Print this message"), "{root}");
}

#[test]
fn fetch_holidays_saves_the_data_and_prints_the_destination() {
    let dir = TempDir::new();
    let path = dir.0.join("bizdate/holidays/holidays.csv");
    let (code, stdout, stderr) = observe_argv_with(
        &[
            "bizdate",
            "fetch-holidays",
            "--source",
            "https://example.com/h.csv",
        ],
        || Ok(path.clone()),
    );
    // 取得サブコマンドは判定の `NO` を使わない。
    assert_eq!(code, SAVED);
    assert_eq!(stdout, format!("{}\n", path.display()));
    assert!(stderr.is_empty(), "{stderr}");
    // 保存したデータはそのまま判定に使える。
    assert_eq!(
        observe_argv_with(
            &[
                "bizdate",
                "first",
                "--date",
                "2026-01-01",
                "--timezone",
                "Asia/Tokyo"
            ],
            || Ok(path.clone()),
        )
        .0,
        NO
    );
}

#[test]
fn fetch_holidays_maps_every_failure_to_the_error_code() {
    let dir = TempDir::new();
    let path = dir.0.join("holidays.csv");
    for source in [
        // 到達不能な取得元と、http(s) ではない取得元。
        "https://unreachable.example/h.csv",
        "file:///etc/passwd",
    ] {
        let (code, stdout, stderr) =
            observe_argv_with(&["bizdate", "fetch-holidays", "--source", source], || {
                Ok(path.clone())
            });
        assert_eq!(code, ERROR, "{source}");
        // 失敗した実行は path を出さず、保存先も作らない。
        assert!(stdout.is_empty(), "{source}");
        assert_eq!(stderr.lines().count(), 1, "{source}: {stderr}");
        assert!(stderr.starts_with("bizdate: "), "{source}: {stderr}");
        assert!(!path.exists(), "{source}");
    }

    // 保存先の解決に失敗した場合も同じ写像になる。
    let (code, stdout, _) = observe_argv_with(&["bizdate", "fetch-holidays"], || {
        Err(HolidayError::InvalidHome)
    });
    assert_eq!((code, stdout.as_str()), (ERROR, ""));
}

#[test]
fn the_source_option_defaults_to_the_cabinet_office_csv() {
    let cli = Cli::try_parse_from(["bizdate", "fetch-holidays"]).unwrap();
    let Command::FetchHolidays(args) = &cli.command else {
        panic!("{:?}", cli.command)
    };
    assert_eq!(args.source, fetch::DEFAULT_SOURCE);

    let cli = Cli::try_parse_from([
        "bizdate",
        "fetch-holidays",
        "--source",
        "http://example.com/h.csv",
    ])
    .unwrap();
    let Command::FetchHolidays(args) = &cli.command else {
        panic!("{:?}", cli.command)
    };
    assert_eq!(args.source, "http://example.com/h.csv");
}
