# 作業ブランチメモ

- ブランチ: `implement-first-last-cli`
- PR: #25
- 最終更新: 2026-09-07

## 目的

Issue #12 の判定サブコマンド `first` / `last` を CLI として配線する。option、入出力ストリーム、exit code 0 / 1 / 2 の写像を実装し、#9 / #10 / #11 が型として返すエラーを exit code `2` へ落とす。

## 現在の状況

- 依存 #11 の PR #24 が merge 済みであることを確認した。
- 最新の `main` から作業ブランチを作成した。
- `src/cli.rs` に CLI 層を置き、`src/main.rs` はその入口だけにした。
- CLI 層の unit test 10 件を追加した。既存 45 件と合わせて 55 件。
- 仕様正本、decision log 0007 / 0014 と index、進捗表を更新した。
- Compose の 4 検査と、ビルドしたバイナリでの process レベル確認がすべて成功した。
- PR #25 のレビュー指摘 3 件に対応した。`[imo]` を採用し、`[ask]` は現状維持のうえ仕様へ 1 行足し、`[fyi]` は情報として受けた。

## 決定事項

- 日本語 help の範囲は「説明文が日本語なら仕様を満たす」とする。`Usage:` / `Options:` の section heading は clap 側の文字列のままとし、`help_template` / `next_help_heading` による上書きは行わない。0014 の CLI パーサ節が #12 へ持ち越していた論点をここで閉じ、経緯を 0007 へ追記して `cli-interface.md` の文言を寄せた。
- clap 既定の `help` subcommand を無効にした。説明文が英語であり、0007 の subcommand 構成（`first` / `last` / `fetch-holidays`）にも含まれないため。
- exit code の写像は CLI 層に閉じる。`YES` / `NO` / `ERROR` を定数として置き、各層のエラーは `CliError` へ集約して `2` にする。判定の「否」は `1` であり、エラーと混ぜない。
- 業務日が 0 日の月は、`first` / `last` とも該当なしとして `no`（exit `1`）を返す。#11 で `Ok(None)` としたものをエラーへ格上げしない。
- 現在時刻は 1 回だけ取得し、タイムゾーン解決・「今日」・`--day-off` の年補完・祝日データの期限判定へ同じ値を渡す。
- 保存先の解決は closure で受け取り、利用者入力の検証を先に済ませる。`date.rs` / `holidays.rs` の `*_with` と同じ注入方式に揃えた。
- stdout への書き出し失敗は `2` にする。書けなかった実行を判定の成否として返さない。
- CLI レベルの検証は、CLI 層の unit test とビルドしたバイナリの手動実行で行う。バイナリを起動する統合テストの整備は #14 の作業内容であり、本 PR では作らない。
- レビュー対応 1 周目。`execute` も `data_path` を closure で受け取る形にし、subcommand から exit code までを unit test で通す。`Command` → `Edge` の対応が検査の外に残っていた。取り違えると判定が反転するため、#14 を待たずに塞ぐ。
- `--version` は root コマンドだけで受ける。subcommand へ伝播するのは `--help` だけとし、その旨を `cli-interface.md` の「共通」へ明記した。clap の挙動を実測した結果、`global = true` だけでは debug assertion で panic し（exit `101`）、`propagate_version = true` を併せると `bizdate first --version` が `bizdate-first 0.1.0` という root と別の名前を出す。version は CLI 全体に 1 つであり、subcommand ごとに別名を示す形は採らない。
- stdout が EPIPE になる場合の挙動は現状のままとする。読み手が読まずに終了する相手（`head -c 0` など）では `2` と診断 1 行になるが、`head -n 1` では `yes` を返して `0`、仕様のゲート形は pipe を使わないため影響しない。

## 次にやること

- PR のレビューと merge 判断を待つ。

## 検証

Compose の `dev` service で次を実行し、すべて成功した。

- `cargo test --locked`: 56 件成功（追加 11 件、既存 45 件）。
- `cargo fmt --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo build --locked`

ビルドしたバイナリを container 内で実行し、`XDG_DATA_HOME` を一時ディレクトリへ向けて #10 の fixture を置いて確認した。Issue の検証項目との対応は次のとおり。

- exit code: 該当日で `0`、非該当日で `1`。不正日付、未知 timezone、空文字 timezone、不正な `--day-off`（形式不正・存在しない日・空要素）、年カバー外、未知 option で `2`。祝日データが無い保存先では `2` であり `1` にならない。`--help` / `--version` は `0`。
- ストリーム: 成功時の stdout は `yes` / `no` の 1 行のみ。`--quiet` で stdout が空になり、exit code は変わらない。エラー時は stdout が空で、診断は stderr の 1 行。`--help` / `--version` は stdout、用法誤りの usage は stderr。
- ゲート用途: `bizdate last --date <月末業務日> --quiet && ...` で後続が走り、非該当日と祝日データ欠落では走らないことを確認した。
- help: コマンドと全 option の説明が日本語で出る。短 option（`-h` / `-V` / `-q` など）と positional 日付は `2` で拒否される。`help` subcommand は提供しない。

CLI 層の unit test では、判定の写像、ストリーム内容、`--quiet`、エラー分類、月の両端が同じ日になる月と業務日 0 日の月、long option の配線、help / version の exit code と日本語文面を検査した。

レビュー対応で追加した `subcommand_names_select_the_matching_month_edge` は、`Command::First` / `Command::Last` の対応を入れ替えると当該 1 件だけが赤になることを確認してから戻した。

レビュー指摘の裏取りとして、container 内で次を実測した。

- `bizdate first --version` は `2`、`bizdate --version` は `0`。
- `--version` に `global = true` だけを足すと clap の debug assertion で panic し、exit `101`。`propagate_version = true` を併せると `bizdate first --version` が `bizdate-first 0.1.0` を出す。確認後に working tree を元へ戻した。
- `| head -c 0` では bizdate 自身の exit code が `2` で stderr 1 行、`| head -n 1` では `yes` を返して `0`。fd 1 を閉じた起動（`>&-`）は `0`。

`git diff --check`、ドキュメントの参照・Issue との整合性、note の情報統制を確認した。バイナリを起動する統合テストと `fetch-holidays` は未実装であり、#13 / #14 の対象。上記はローカル Compose の結果であり、GitHub CI の結果は PR checks で確認する。

## リスク・ブロッカー

- ブロッカーなし。
- subcommand の help で、global option の `--help` が `--day-off` と `--quiet` の間に並ぶ。clap が global option へ与える表示順によるもので、`display_order` を明示すれば動かせるが、magic number を増やすため今回は既定のままにした。
- 読み手が読まずに終了する pipe（`head -c 0` など）では、判定に成功しても EPIPE で `2` を返す。cron から stderr がメール化される構成では診断 1 行が出る。仕様のゲート形は pipe を使わないため、実用上の影響は小さいと判断している。

## セッションログ

- 2026-09-07: Issue #12 を開始した。依存 #11（PR #24）の merge を確認し、日本語 help の範囲を決めた。
- 2026-09-07: CLI 層と unit test を実装した。値欠落エラーだけ clap が usage 行を出さないことと、既定の `help` subcommand が英語で残ることを実機で見つけ、後者は無効化した。
- 2026-09-07: 仕様正本と decision log を更新し、Compose の 4 検査とバイナリでの process レベル確認を通した。
- 2026-09-07: レビュー対応 1 周目。`execute` を引数注入に変えて subcommand の対応を検査対象へ入れ、`--version` の適用範囲を仕様へ明記した。指摘の前提は推論ではなく container 内の実測で確認した。
