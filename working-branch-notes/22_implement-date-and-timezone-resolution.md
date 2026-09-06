# 作業ブランチメモ

- ブランチ: `implement-date-and-timezone-resolution`
- PR: #22
- 最終更新: 2026-09-07

## 目的

Issue #9 の日付・タイムゾーン解決層を実装する。仕様の正本は `doc/design/business-day.md`。

## 現在の状況

- 依存 #7 の PR #19 は merge 済み。open PR は無いことを確認した。
- `main` を fast-forward で最新化し、推奨ブランチを作成した。
- API と 12 件の unit test を実装し、Compose 検証を完了した。
- `progress.md` は merge 後の状態として V1-03 を done に更新した。

## 決定事項

- 既存依存の `jiff` を使う。日付形式は ASCII の `YYYY-MM-DD` に限定し、暦日としての有効性は `jiff` で検査する。
- CLI 配線は #12 の範囲とし、本 Issue は API とテストまでとする。
- 現在時刻、環境変数読み取り、local timezone の境界を分け、テストでプロセス環境を書き換えない。
- `src/lib.rs` は後続の CLI と処理層が共有する内部用 library target の入口とする。既存の `publish = false` は維持する。
- local timezone の取得失敗は型付きエラーとし、暗黙に UTC へ fallback しない。明示指定は IANA 名の lookup とし、Jiff の sentinel `Etc/Unknown` は拒否する。

## 次にやること

- PR のレビューと merge 判断を待つ。

## 検証

- `docker compose run --rm dev cargo test --locked`: 12 tests 成功。実環境の確認は 2 つの子プロセスでも各 1 test 成功。
- `docker compose run --rm dev cargo fmt --check`: 成功。
- `docker compose run --rm dev cargo clippy --locked --all-targets -- -D warnings`: 成功。
- `docker compose run --rm dev cargo build --locked`: 成功。
- 優先順位 3 通り、UTC / Asia/Tokyo の日・月・年境界、指定日の暦月、うるう年、不正形式・存在しない日・未知の IANA 名、`TZ` と `BIZDATE_TZ` の区別を網羅した。
- 現在時刻は固定した `Timestamp` を注入した。専用入力として `TZ` を読まないことは環境読み取りの差し替えで、local 解決に `TZ` が効くことは子プロセスで確認した。
- 初回ビルドでは `env::var` の関数参照の lifetime 制約で失敗した。closure へ変更して解消した。
- CLI 配線・exit code の検証は #12 の範囲のため未実施。GitHub CI の結果は PR の checks で別途確認する。

## リスク・ブロッカー

- 現時点で未解決のブロッカーは無い。

## セッションログ

- 2026-09-07: Issue 本文と仕様を確認し、着手した。
