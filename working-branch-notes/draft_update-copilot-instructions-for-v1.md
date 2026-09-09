# 作業ブランチメモ

- ブランチ: `update-copilot-instructions-for-v1`
- PR: （採番前）
- 最終更新: 2026-09-09

## 目的

Issue #34 に従い、`.github/copilot-instructions.md` の「設計と開発環境整備のフェーズであり、実装コードはまだ無い」というフェーズ記述を、v1 実装後の現状（Rust 実装と統合テストが存在する、CI が `fmt` / `clippy` / `test` / `build` を検査する、配布手段は未決）に合わせて更新する。同 file の他の記述も `progress.md` の完了表と照らして stale な箇所を点検する。

## 現在の状況

- Issue #34 に依存は無い。v1 実装の Issue #7〜#15 は close 済み、対応 PR は merge 済みであることを `progress.md` の索引で確認した。
- 最新の `main`（`origin/main` と同一 commit）から作業ブランチを作成した。

## 決定事項

- フェーズ記述は「レビューの目的」の 1 項目を 3 項目に分け、実装の所在、CI の検査内容とその含意、配布手段が未決であることの含意を書く。Copilot がリンク先の正本を辿らない前提で、レビューの前提として使える要点に絞る（`doc/guidelines/agent-configuration-management.md`）。
- 「原則として指摘しない事項」の「設計段階のひな形ドキュメントにおける未記入項目」は、設計フェーズを前提としない表現（`_template.md` などひな形の未記入項目）へ改める。意図は変えない。
- 上記以外の記述（prefix 表、情報統制、ドキュメント構成と working branch note のレビュー観点）は `progress.md` の完了表と現行の guideline に照らして現状と一致しており、変更しない。
- Rust コード向けの新しいレビュー観点は追加しない（Issue #34 のスコープ外）。
- `progress.md` は更新しない。#34 は索引に無い単発 Issue である。

## 次にやること

- PR 作成後、note を採番する。
- PR のレビューとユーザーによる merge。

## 検証

- `git diff --check`: 成功。
- `.github/copilot-instructions.md` が参照する path / file 名の実在を確認した。`doc/guidelines/`、`.cursor/rules/`、`.claude/rules/`、`AGENTS.md`、`CLAUDE.md`、`progress.md`、`working-branch-notes/`、`src/`、`tests/`、`doc/design/` 直下の `concept.md` / `business-day.md` / `cli-interface.md`、`doc/design/decision-log/index.md`、`_template.md`（decision log と working-branch-notes の両方）がすべて存在する。
- 変更後の file に「実装コードはまだ無い」「設計段階」「フェーズ」「現段階」を含む記述が残っていないことを grep で確認した。
- 文末の常体を確認した。ですます調の文末は無い。
- 新記述の事実確認: `src/*/tests.rs` の unit test 4 file が存在すること、`tests/cli_e2e.rs` が `CARGO_BIN_EXE_bizdate` の実バイナリを `Command` で起動していること、`.github/workflows/ci.yml` が `cargo fmt --check` / `cargo clippy --locked --all-targets -- -D warnings` / `cargo test --locked` / `cargo build --locked` を実行していること、decision log index で配布手段が `open` であることを確認した。
- note の情報統制チェックで、禁則語に続く実値、長いランダム文字列、ローカル絶対 path が無いことを確認した。
- 未実施: Rust の `fmt` / `clippy` / `test` / `build`。変更は Copilot 向け Markdown と note のみで、crate と CI の実行内容に影響しないため。

## リスク・ブロッカー

- 現時点でブロッカーなし。

## セッションログ

- 2026-09-09: stale なフェーズ記述の指摘を受け、Issue #34 を作成した。最新 `main` からブランチを作成し、note を作成した。
- 2026-09-09: 対象 file の 2 箇所を更新し、参照・文体・事実確認の検証を実施した。
