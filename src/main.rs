use clap::Parser;

// bizdate の共通 option。
//
// 判定サブコマンドと `fetch-holidays` は後続の Issue で配線する。
// version 文字列は `Cargo.toml` の `version` を clap が読むため、ソースへ直書きしない。
#[derive(Parser)]
#[command(
    name = "bizdate",
    version,
    about = "その日が月の最初 / 最後の業務日かどうかを判定する",
    // 短 option は v1 では提供しないため (doc/design/cli-interface.md)、
    // clap 既定の `-h` / `-V` を無効にし、long option だけを定義する。
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// usage を表示して終了する
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// version を表示して終了する
    #[arg(long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let _cli = Cli::parse();
}
