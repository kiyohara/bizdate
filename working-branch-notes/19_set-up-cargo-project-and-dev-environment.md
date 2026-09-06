# 作業ブランチメモ

- ブランチ: `set-up-cargo-project-and-dev-environment`
- PR: #19
- 最終更新: 2026-09-06

## 目的

Issue #7 に従い、v1 実装の基盤を置く。Cargo プロジェクトの初期化、依存クレートの選定と decision log 0014 への記録、`compose.yaml`、`development-command-guidelines.md` の整備、繰り越し 2 件の解消までを行う。

## 現在の状況

Issue #7 の作業内容 1〜5 をすべて実施し、検証も完了した。PR #19 を作成し、レビュー待ち。

## 決定事項

### toolchain

- edition は `2024`、MSRV は `1.98`。`Cargo.toml` の `rust-version`、`Dockerfile` の base image tag、CI の toolchain をこの値で揃える。
- MSRV を最新 stable に固定した。配布はビルド済みバイナリ起点であり、古い toolchain を支える利益が現時点で無い。MSRV を後ろに置く案は、CI に MSRV ジョブが増えて image tag と 2 系統の保守になるため採らなかった。
- 公式 `rust` image は minimal profile で rustfmt / clippy を含まない。`Dockerfile` で `rustup component add rustfmt clippy` を足した image を使う。Issue の「`rust` 公式 image を base にする」はこの形で満たす。

### 依存クレート

| 系統 | 採用 | 主な理由 |
|---|---|---|
| CLI パーサ | `clap`（derive） | サブコマンド / long option / 日本語 help / `--version` をすべて満たす |
| 日付・タイムゾーン | `jiff` | 単一クレートで IANA 解決・local TZ・暦月境界を扱える。`chrono` は `chrono-tz` + `iana-time-zone` の 3 クレート構成になる |
| CSV | `csv` | quote を含む祝日名を自前パーサで読み違えるリスクを避ける |
| HTTP client | `ureq`（`default-features = false`, `features = ["rustls"]`） | GET 1 回に `reqwest` + tokio は過大 |
| 文字コード変換 | `encoding_rs` | CP932 の変換表を自前保守しない |
| XDG パス解決 | クレートを追加せず自前実装 | `directories` / `dirs` は macOS で `~/Library/Application Support` を返し、`cli-interface.md` の `~/.local/share` fallback と食い違う |

- 依存ツリーを実機で確認し、`openssl` / `native-tls` / `aws-lc-rs` が入らないことを確認した。rustls の crypto provider は `ring` に解決される。`ring` は vendored な C / asm を含むがシステムライブラリへのリンクを要求しないため、0008 の意図（C ライブラリ依存によるクロスビルド破綻の回避）は満たす。
- ルート証明書は `webpki-roots` で同梱され、実行環境の証明書ストアに依存しない。
- error 処理クレート（`anyhow` / `thiserror` 等）は 0014 の対象外とし、必要になった Issue で判断する。
- 選定した 5 クレートは、後続 Issue で使うまで未使用のまま `Cargo.toml` に置いてある。選定結果を実行可能な形にし、pure Rust であることを実機で確認するため。

### 短 option の扱い

`cli-interface.md` の「短 option は v1 では提供しない」は clap 既定の `-h` / `-V` にも及ぶと読み、`disable_help_flag` / `disable_version_flag` を立てて `--help` / `--version` を long option として明示定義した。`-V` は exit code 2 で拒否される（`cli-interface.md` の「未知の option は exit code 2」と整合）。

### compose

- 開発用 service 名は `dev`。cargo registry と `target/` は named volume に保持する。
- host の `TZ` と `BIZDATE_TZ` を引き継ぐ。host 側 `TZ` が未設定なら container は UTC で動く。
- `target/` が named volume にあるため、ビルドしたバイナリも container 内で実行する。

## 次にやること

- PR #19 のレビュー対応。merge はユーザーが行う。
- 後続は Issue #8（CI workflow）。0014 の MSRV と同じ toolchain で fmt / clippy / test を回す。

## 検証

Issue #7 の検証項目をすべて Compose 経由で実施した。

| 項目 | 結果 |
|---|---|
| `docker compose run --rm dev cargo build` | 成功 |
| `docker compose run --rm dev cargo test` | 成功（0 tests） |
| `docker compose run --rm dev cargo fmt --check` | 成功（差分なし） |
| `docker compose run --rm dev cargo clippy --all-targets -- -D warnings` | 成功（警告なし） |
| `./target/debug/bizdate --version` | `bizdate 0.1.0` を出力 |
| `development-command-guidelines` の basename が 5 箇所で揃う | 共通正本 / `.cursor/rules/` / `.claude/rules/` / `AGENTS.md` / `doc/guidelines/README.md` で確認 |
| 存在しないファイルへの参照が無い | tracked な `*.md` の markdown link と repo 相対 path を機械的に確認。新規・変更ファイルに参照切れなし |
| `progress.md` と `concept.md` の繰り越し 2 件 | 解消を確認 |
| decision log 0014 の作成と index への追加 | 確認 |

追加で確認したもの。

- `docker compose config -q` が通る。
- `TZ=Asia/Tokyo` を与えると container 側へ引き継がれる。未設定時は UTC になる。
- `--help` の出力に実装メモが混ざらない（struct の doc comment を通常のコメントへ変えて対処した）。
- `-V` が exit code 2 で拒否される。

未実施のもの。

- CI での実行。Issue #8 のスコープ。
- amd64 での実行確認。手元は arm64 のみ。CI で matrix を組む段（Issue #8）で確認する。

## リスク・ブロッカー

- `jiff` は 0.x であり semver 上は破壊的変更があり得る。公開ライブラリクレートを提供しないため影響はクレート内に閉じる（0014 の「後から見直す条件」に記載）。
- `encoding_rs` のライセンスに BSD-3-Clause 部分が含まれる。配布物へ third-party ライセンス表記を含める必要がある。配布手段を決める段で扱う（0014 の「影響」に記載）。
- 選定した 5 クレートのうち 4 つは未使用のままである。後続 Issue で実際に使うまで、API 上の相性は未検証。

## セッションログ

- 2026-09-06: ブランチ作成、正本確認、note 作成。
- 2026-09-06: `Cargo.toml` / `src/main.rs` / `compose.yaml` / `Dockerfile` / `.dockerignore` を作成。公式 rust image に rustfmt / clippy が無いことが判明し、`Dockerfile` を追加した。
- 2026-09-06: 依存ツリーを確認して 0014 を作成。`development-command-guidelines.md` と入口 2 件、`AGENTS.md` / `doc/guidelines/README.md` / decision log index を更新。
- 2026-09-06: `progress.md` の繰り越し節を削除し V1-01 を done に、`concept.md` のライセンス記述 2 箇所を修正。検証を実施。
