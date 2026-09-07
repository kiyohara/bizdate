//! CLI の option 配線と exit code の写像。業務日の判定そのものは各 module に置く。

use std::{
    error::Error,
    fmt,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{ArgAction, Args, Parser, Subcommand};
use jiff::Timestamp;

use crate::{
    business_day::{BusinessCalendar, DayOffError, DaysOff},
    date::{self, DateError},
    holidays::{self, HolidayData, HolidayError},
};

/// 該当する。
const YES: u8 = 0;
/// 該当しない。判定の「否」であり、失敗ではない。
const NO: u8 = 1;
/// 用法誤り、不正入力、祝日データの不備、内部エラー。clap の既定 exit code と揃える。
const ERROR: u8 = 2;

// bizdate の共通 option。
//
// version 文字列は `Cargo.toml` の `version` を clap が読むため、ソースへ直書きしない。
// help の section heading は clap 側の文字列のまま残す
// (doc/design/decision-log/0007-cli-shape.md の追記)。
#[derive(Parser, Debug)]
#[command(
    name = "bizdate",
    version,
    about = "その日が月の最初 / 最後の業務日かどうかを判定する",
    // 短 option は v1 では提供しないため (doc/design/cli-interface.md)、
    // clap 既定の `-h` / `-V` を無効にし、long option だけを定義する。
    disable_help_flag = true,
    disable_version_flag = true,
    // subcommand は first / last / fetch-holidays だけとするため、
    // clap 既定の `help` subcommand を出さない。
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// usage を表示して終了する
    #[arg(long, action = ArgAction::Help, global = true)]
    help: Option<bool>,

    /// version を表示して終了する
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// 指定日がその月の最初の業務日かどうかを判定する
    First(Judgment),
    /// 指定日がその月の最後の業務日かどうかを判定する
    Last(Judgment),
}

/// `first` と `last` は探す端が違うだけで、option は共通である。
#[derive(Args, Debug)]
// 親の `--help` を global option として受け取るため、subcommand 側の既定 help flag を止める。
#[command(disable_help_flag = true)]
struct Judgment {
    /// 判定対象日。既定は採用タイムゾーンの今日
    #[arg(long, value_name = "YYYY-MM-DD")]
    date: Option<String>,

    /// 「今日」と年省略の基準タイムゾーン。既定は BIZDATE_TZ、次に local
    #[arg(long, value_name = "IANA")]
    timezone: Option<String>,

    /// 追加の休日。カンマ区切りの YYYY-MM-DD または MM-DD
    #[arg(long, value_name = "LIST")]
    day_off: Option<String>,

    /// stdout へ判定結果を出さない
    #[arg(long)]
    quiet: bool,
}

/// 月内のどちらの端を探すか。
#[derive(Clone, Copy, Debug)]
enum Edge {
    First,
    Last,
}

/// 各層が型として返すエラーを、exit code `2` へまとめて写像するための入口。
#[derive(Debug)]
enum CliError {
    Date(DateError),
    DayOff(DayOffError),
    Holiday(HolidayError),
    Output(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date(error) => error.fmt(f),
            Self::DayOff(error) => error.fmt(f),
            Self::Holiday(error) => error.fmt(f),
            Self::Output(error) => write!(f, "cannot write the result: {error}"),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Date(error) => Some(error),
            Self::DayOff(error) => Some(error),
            Self::Holiday(error) => Some(error),
            Self::Output(error) => Some(error),
        }
    }
}

impl From<DateError> for CliError {
    fn from(error: DateError) -> Self {
        Self::Date(error)
    }
}

impl From<DayOffError> for CliError {
    fn from(error: DayOffError) -> Self {
        Self::DayOff(error)
    }
}

impl From<HolidayError> for CliError {
    fn from(error: HolidayError) -> Self {
        Self::Holiday(error)
    }
}

/// 判定 1 回分の入口。clap が help / version / 用法誤りを先に処理する。
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let mut stdout = io::stdout().lock();
    let result = match execute(&cli.command, Timestamp::now(), &mut stdout) {
        // 書き出しに失敗した実行を、判定の成否として返さない。
        Ok(code) => stdout.flush().map(|()| code).map_err(CliError::Output),
        Err(error) => Err(error),
    };
    ExitCode::from(finish(result, &mut io::stderr()))
}

fn execute(command: &Command, now: Timestamp, stdout: &mut impl Write) -> Result<u8, CliError> {
    let (edge, args) = match command {
        Command::First(args) => (Edge::First, args),
        Command::Last(args) => (Edge::Last, args),
    };
    let matched = decide(edge, args, now, holidays::data_path)?;
    report(matched, args.quiet, stdout)
}

/// 保存先の解決は closure で受け取り、利用者入力の検証を先に済ませる。
fn decide(
    edge: Edge,
    args: &Judgment,
    now: Timestamp,
    data_path: impl FnOnce() -> Result<PathBuf, HolidayError>,
) -> Result<bool, CliError> {
    let timezone = date::resolve_timezone(args.timezone.as_deref())?;
    let target = date::resolve_date(args.date.as_deref(), &timezone, now)?;
    let days_off = match args.day_off.as_deref() {
        // 年省略は、判定対象日の年ではなく採用タイムゾーンの今日を基準にする。
        Some(input) => DaysOff::parse(input, date::today(&timezone, now))?,
        None => DaysOff::default(),
    };
    let holidays = HolidayData::load(&data_path()?, now)?;
    let calendar = BusinessCalendar::new(&holidays, &days_off, now);
    let edge_day = match edge {
        Edge::First => calendar.first_business_day(target)?,
        Edge::Last => calendar.last_business_day(target)?,
    };
    // 業務日が 0 日の月は該当日なしであり、データ不備ではない。
    Ok(edge_day == Some(target))
}

/// 判定結果を stdout へ 1 行だけ書き、exit code へ写像する。
fn report(matched: bool, quiet: bool, stdout: &mut impl Write) -> Result<u8, CliError> {
    if !quiet {
        writeln!(stdout, "{}", if matched { "yes" } else { "no" }).map_err(CliError::Output)?;
    }
    // exit code は `--quiet` の有無で変わらない。
    Ok(if matched { YES } else { NO })
}

/// エラーを stderr へ出し、exit code `2` へ写像する。stdout は汚さない。
fn finish(result: Result<u8, CliError>, stderr: &mut impl Write) -> u8 {
    match result {
        Ok(code) => code,
        Err(error) => {
            // 診断の書き出しに失敗しても、元のエラーの exit code を保つ。
            let _ = writeln!(stderr, "bizdate: {error}");
            ERROR
        }
    }
}

#[cfg(test)]
mod tests;
