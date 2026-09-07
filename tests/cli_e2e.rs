mod support;

use std::fs;
use support::{BINARY, Sandbox, assert_error, assert_output, csv, serve};

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
    let sandbox = Sandbox::new();
    for (state, diagnostic) in [
        ("missing", "missing"),
        ("expired", "expired"),
        ("malformed", "metadata"),
    ] {
        match state {
            "missing" => {}
            "expired" => sandbox.write_data("2000-01-01T00:00:00Z"),
            "malformed" => {
                fs::write(sandbox.data_path(), "invalid CSV\n").unwrap();
            }
            _ => unreachable!(),
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
