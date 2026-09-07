mod support;

use std::fs;
use support::{BINARY, Sandbox, assert_error, assert_output, csv, serve, serve_with_type};

#[test]
fn fetch_then_judge_round_trip() {
    for shift_jis in [false, true] {
        let sandbox = Sandbox::new();
        let csv = csv();
        let body = if shift_jis {
            let (bytes, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&csv);
            assert!(!had_errors);
            bytes.into_owned()
        } else {
            csv.as_bytes().to_vec()
        };
        let (url, server) = serve("200 OK", body);
        let output = sandbox.run(&["fetch-holidays", "--source", &url]);
        server.join().unwrap();
        assert_output(output, 0, &format!("{}\n", sandbox.data_path().display()));
        let saved = fs::read_to_string(sandbox.data_path()).unwrap();
        assert!(saved.starts_with("# bizdate-meta schema=1\n# fetched_at="));
        assert!(saved.contains(&format!("\n# source_url={url}\n{csv}")));
        // server は終了済み。判定は保存データだけで完結する。
        assert_output(sandbox.run(&["first", "--date", "2026-01-02"]), 0, "yes\n");
        assert_output(sandbox.run(&["last", "--date", "2026-08-28"]), 0, "yes\n");
        assert_output(sandbox.run(&["first", "--date", "2026-01-01"]), 1, "no\n");
    }
}

#[test]
fn month_edges_and_days_off_have_exact_output_and_quiet_exit_codes() {
    let sandbox = Sandbox::new();
    sandbox.seed();
    // 2026/2 は日曜始まり・土曜終わり。5/1 と 8/31 は fixture の休日。
    for (command, date, day_off, code) in [
        ("first", "2026-02-01", None, 1),
        ("first", "2026-02-02", None, 0),
        ("first", "2026-02-03", None, 1),
        // 年補完後も 2 月の判定に影響しない MM-DD を使い、受理経路を固定する。
        ("first", "2026-02-02", Some("12-25"), 0),
        ("last", "2026-02-27", None, 0),
        ("last", "2026-02-28", None, 1),
        ("first", "2026-05-01", None, 1),
        ("first", "2026-05-04", None, 0),
        ("last", "2026-08-31", None, 1),
        ("last", "2026-08-28", None, 0),
        ("first", "2026-05-04", Some("2026-05-04"), 1),
        ("first", "2026-05-05", Some("2026-05-04"), 0),
        ("last", "2026-08-28", Some("2026-08-28"), 1),
        ("last", "2026-08-27", Some("2026-08-28"), 0),
    ] {
        let mut args = vec![command, "--date", date];
        if let Some(day_off) = day_off {
            args.extend(["--day-off", day_off]);
        }
        assert_output(
            sandbox.run(&args),
            code,
            if code == 0 { "yes\n" } else { "no\n" },
        );
        args.push("--quiet");
        assert_output(sandbox.run(&args), code, "");
    }
}

#[test]
fn single_business_day_is_both_first_and_last_and_zero_days_is_no() {
    let sandbox = Sandbox::new();
    sandbox.seed();
    let days_off = (1..=28)
        .filter(|day| *day != 16)
        .map(|day| format!("2026-02-{day:02}"))
        .collect::<Vec<_>>()
        .join(",");
    for command in ["first", "last"] {
        assert_output(
            sandbox.run(&[command, "--date", "2026-02-16", "--day-off", &days_off]),
            0,
            "yes\n",
        );
        assert_output(
            sandbox.run(&[command, "--date", "2026-02-17", "--day-off", &days_off]),
            1,
            "no\n",
        );
        assert_output(
            sandbox.run(&[
                command,
                "--date",
                "2026-02-16",
                "--day-off",
                &format!("{days_off},2026-02-16"),
            ]),
            1,
            "no\n",
        );
    }
}

#[test]
fn invalid_input_is_error_even_in_quiet_mode() {
    let sandbox = Sandbox::new();
    sandbox.seed();
    for command in ["first", "last"] {
        for (extra, diagnostic) in [
            (vec!["--date", "2026-02-30"], "date"),
            (vec!["--date", "2026/02/02"], "date"),
            (vec!["--date", "2027-02-01"], "2027"),
            (vec!["--timezone", "Invalid/Zone"], "Invalid/Zone"),
            (vec!["--timezone", ""], "time zone"),
            (vec!["--day-off", "bad"], "bad"),
            (vec!["--day-off", "2026-02-30"], "2026-02-30"),
            (vec!["--day-off", "2026-02-02,"], "day-off"),
            (vec!["--day-off", ""], "day-off"),
            (vec!["--unknown"], "--unknown"),
        ] {
            for quiet in [false, true] {
                let mut args = vec![command];
                if !extra.contains(&"--date") {
                    args.extend(["--date", "2026-02-02"]);
                }
                args.extend(extra.iter().copied());
                if quiet {
                    args.push("--quiet");
                }
                assert_error(sandbox.run(&args), diagnostic);
            }
        }
    }
}

#[test]
fn missing_expired_and_malformed_data_are_errors_including_on_weekends() {
    enum DataState {
        Missing,
        Expired,
        Malformed,
    }
    for (state, diagnostic) in [
        (DataState::Malformed, "metadata"),
        (DataState::Missing, "missing"),
        (DataState::Expired, "expired"),
    ] {
        let sandbox = Sandbox::new();
        match state {
            DataState::Missing => {}
            DataState::Expired => sandbox.write_data("2000-01-01T00:00:00Z"),
            DataState::Malformed => {
                fs::create_dir_all(sandbox.data_path().parent().unwrap()).unwrap();
                fs::write(sandbox.data_path(), "invalid CSV\n").unwrap();
            }
        }
        for command in ["first", "last"] {
            for date in ["2026-02-02", "2026-02-01"] {
                for quiet in [false, true] {
                    let mut args = vec![command, "--date", date];
                    if quiet {
                        args.push("--quiet");
                    }
                    assert_error(sandbox.run(&args), diagnostic);
                }
            }
        }
    }
}

#[test]
fn fetch_failures_are_errors_and_preserve_existing_data() {
    let sandbox = Sandbox::new();
    sandbox.seed();
    let original = fs::read(sandbox.data_path()).unwrap();
    for (status, body, diagnostic) in [
        (
            "503 Service Unavailable",
            b"unavailable".to_vec(),
            "cannot fetch",
        ),
        ("200 OK", b"<html>not a CSV</html>".to_vec(), "unusable"),
    ] {
        let (url, server) = serve(status, body);
        let output = sandbox.run(&["fetch-holidays", "--source", &url]);
        server.join().unwrap();
        assert_error(output, diagnostic);
        assert_eq!(fs::read(sandbox.data_path()).unwrap(), original);
        assert_eq!(
            fs::read_dir(sandbox.data_path().parent().unwrap())
                .unwrap()
                .count(),
            1
        );
    }
    assert_error(
        sandbox.run(&["fetch-holidays", "--source", "file:///unused.csv"]),
        "http(s)",
    );
    // root で動く Compose でも再現する保存失敗。権限ビットに依存させない。
    let blocked = Sandbox::new();
    fs::write(blocked.0.join("data"), "not a directory").unwrap();
    let (url, server) = serve("200 OK", csv().into_bytes());
    let output = blocked.run(&["fetch-holidays", "--source", &url]);
    server.join().unwrap();
    assert_error(output, "cannot save");
}

#[test]
fn shell_gate_runs_the_following_command_only_for_yes() {
    let sandbox = Sandbox::new();
    sandbox.seed();
    for (date, expected_code, expected_stdout) in [
        ("2026-02-27", 0, "ran\n"),
        ("2026-02-26", 1, ""),
        ("2026-02-30", 2, ""),
    ] {
        // 値は shell 本文へ展開せず positional parameter で渡す。
        let output = sandbox
            .command("/bin/sh")
            .args([
                "-c",
                "\"$1\" last --date \"$2\" --quiet && printf 'ran\\n'",
                "gate",
                BINARY,
                date,
            ])
            .output()
            .unwrap();
        if expected_code == 2 {
            assert_error(output, "date");
        } else {
            assert_output(output, expected_code, expected_stdout);
        }
    }
}

#[test]
fn declared_charset_is_used_from_the_http_response() {
    let sandbox = Sandbox::new();
    let csv = csv();
    let (bytes, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&csv);
    assert!(!had_errors);
    let body = bytes.into_owned();
    let (url, server) = serve_with_type("200 OK", "text/csv; charset=Shift_JIS", body.clone());
    let output = sandbox.run(&["fetch-holidays", "--source", &url]);
    server.join().unwrap();
    assert_output(output, 0, &format!("{}\n", sandbox.data_path().display()));
    let saved = fs::read_to_string(sandbox.data_path()).unwrap();
    assert!(saved.ends_with(&csv));

    // 宣言を捨てても上の成功ケースは推定で通るため、宣言が推定より優先されることも検査する。
    for (content_type, diagnostic) in [
        (
            "text/csv; charset=unknown-test-charset",
            "unknown source charset",
        ),
        ("text/csv; charset=UTF-8", "source is not valid UTF-8"),
    ] {
        let (url, server) = serve_with_type("200 OK", content_type, body.clone());
        let output = sandbox.run(&["fetch-holidays", "--source", &url]);
        server.join().unwrap();
        assert_error(output, diagnostic);
        assert_eq!(fs::read_to_string(sandbox.data_path()).unwrap(), saved);
    }
}

#[test]
fn successful_refresh_replaces_data_and_changes_the_judgment() {
    let sandbox = Sandbox::new();
    // 同じサーバーを生かしたまま次を bind し、source_url が確実に異なるようにする。
    let (first_url, first_server) = serve("200 OK", csv().into_bytes());
    let updated_csv = format!("{}2026/1/2,追加休日\n", csv());
    let (second_url, second_server) = serve("200 OK", updated_csv.clone().into_bytes());
    assert_ne!(first_url, second_url);
    let output = sandbox.run(&["fetch-holidays", "--source", &first_url]);
    first_server.join().unwrap();
    assert_output(output, 0, &format!("{}\n", sandbox.data_path().display()));
    assert_output(sandbox.run(&["first", "--date", "2026-01-02"]), 0, "yes\n");

    let output = sandbox.run(&["fetch-holidays", "--source", &second_url]);
    second_server.join().unwrap();
    assert_output(output, 0, &format!("{}\n", sandbox.data_path().display()));
    let saved = fs::read_to_string(sandbox.data_path()).unwrap();
    assert!(saved.contains(&format!("\n# source_url={second_url}\n{updated_csv}")));
    assert!(!saved.contains(&first_url));
    assert_eq!(
        fs::read_dir(sandbox.data_path().parent().unwrap())
            .unwrap()
            .count(),
        1
    );
    assert_output(sandbox.run(&["first", "--date", "2026-01-02"]), 1, "no\n");
    assert_output(sandbox.run(&["first", "--date", "2026-01-05"]), 0, "yes\n");
}

#[test]
fn help_version_and_usage_exit_through_the_correct_stream() {
    let sandbox = Sandbox::new();
    // 祝日データを置かず、パーサの終了経路だけを通す。
    for args in [
        vec!["--help"],
        vec!["first", "--help"],
        vec!["last", "--help"],
        vec!["fetch-holidays", "--help"],
    ] {
        let output = sandbox.run(&args);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Usage: bizdate"), "{stdout:?}");
    }
    assert_output(
        sandbox.run(&["--version"]),
        0,
        &format!("bizdate {}\n", env!("CARGO_PKG_VERSION")),
    );
    for args in [
        vec![],
        vec!["--unknown"],
        vec!["help"],
        vec!["-h"],
        vec!["first", "--version"],
    ] {
        assert_error(sandbox.run(&args), "Usage:");
    }
}

#[test]
fn omitted_date_uses_today_without_fixed_year_coverage() {
    let sandbox = Sandbox::new();
    let now = jiff::Timestamp::now();
    let year = now.to_zoned(jiff::tz::TimeZone::UTC).year();
    // 年を跨ぐ実行でも年カバー外にならない。日付計算自体の期待値は unit test が担う。
    let body = format!(
        "国民の祝日・休日月日,国民の祝日・休日名称\n{}/1/1,休日\n{}/1/1,休日\n",
        year - 1,
        year + 1
    );
    sandbox.write_csv(&now.to_string(), &body);
    for command in ["first", "last"] {
        let output = sandbox.run(&[command]);
        let code = output.status.code().unwrap();
        assert!(matches!(code, 0 | 1), "{output:?}");
        assert_output(output, code, if code == 0 { "yes\n" } else { "no\n" });
    }
}

#[test]
fn unset_xdg_data_home_uses_home_for_fetch_and_judgment() {
    let sandbox = Sandbox::new();
    let (url, server) = serve("200 OK", csv().into_bytes());
    let output = sandbox
        .command(BINARY)
        .env_remove("XDG_DATA_HOME")
        .args(["fetch-holidays", "--source", &url])
        .output()
        .unwrap();
    server.join().unwrap();
    let path = sandbox.0.join(".local/share/bizdate/holidays/holidays.csv");
    assert_output(output, 0, &format!("{}\n", path.display()));
    assert!(fs::read_to_string(path).unwrap().ends_with(&csv()));
    assert!(!sandbox.data_path().exists());
    let output = sandbox
        .command(BINARY)
        .env_remove("XDG_DATA_HOME")
        .args(["first", "--date", "2026-01-02"])
        .output()
        .unwrap();
    assert_output(output, 0, "yes\n");
}
