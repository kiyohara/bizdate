# Copilot Review レビューガイドライン

I want to review in Japanese.

## レビューの目的

- 目的はマージ前に修正すべき問題を見つけることであり、改善案の網羅的な列挙ではない。**重要な問題が無ければ、追加コメントを生成しないことを正しい結果とする。**
- 本リポジトリは v1 の CLI 実装を完了している。実装言語は Rust で、`src/` に実装と unit test、`tests/` に実バイナリを起動する統合テストがある。
- CI（GitHub Actions）が PR ごとに `cargo fmt --check` / `clippy`（warning を error 扱い）/ `test` / `build` を検査する。フォーマット違反、clippy の lint、ビルドエラーは CI が検出するため指摘しない。
- 配布手段（GitHub Releases / Homebrew）は未決である。インストール手順や release workflow が無いことは指摘しない。

## 原則

- `Pull Request Overview` を含め、すべて日本語で出力する。
- すべてのコメントに、内容に応じて次のいずれかの prefix を必ず付ける。

| prefix | 使う場面 | マージへの影響 |
| --- | --- | --- |
| `[must]` | correctness / security / reliability / 秘密情報混入に影響する問題 | マージ前に修正が必要 |
| `[ask]` | 意図や前提の確認が必要 | 回答次第で要修正 |
| `[imo]` | 保守性向上の提案 | 影響なし（任意） |
| `[nits]` | 軽微な改善提案 | 影響なし（任意） |
| `[fyi]` | 情報共有のみ | 影響なし |

指摘は重要度の高い順に提示する。correctness → security / privacy → reliability → maintainability → guideline 違反。

## 情報統制

本リポジトリは public である。認証情報、API key、access token、秘密鍵、個人情報、認証情報付き URL、開発機のローカル絶対 path の混入は `[must]` で指摘する。secret は 1Password の secret reference（`op://...`）で書き、実値を置かない。

## 原則として指摘しない事項

マージを妨げない限り指摘しない。指摘する場合も `[imo]` / `[nits]` / `[fyi]` に留める。

- 個人の好みに依存する命名、コードスタイル / フォーマット
- 任意のリファクタリング、将来的な改善提案
- `_template.md` などひな形の未記入項目

## ドキュメント構成のレビュー観点

本リポジトリは「共通正本 + 薄い入口」の構成を取る。恒久ルールの本文は `doc/guidelines/` に置き、`.cursor/rules/`、`.claude/rules/`、`AGENTS.md`、`CLAUDE.md` は正本への参照だけを持つ。

- 入口に恒久ルールが直接書かれていたら指摘する。
- rule の追加・削除・rename では、共通正本・Cursor 入口・Claude Code 入口の basename が揃っているか、`AGENTS.md` にリンクがあるかを確認する。**Codex は `AGENTS.md` からしか正本に到達できないため、リンク漏れは `[must]` で指摘する。**
- 存在しないファイルへの参照とリンク切れは指摘する。
- `doc/design/` 直下の spec（`concept.md` / `business-day.md` / `cli-interface.md`）が仕様の正本である。spec 間の矛盾や decision log の決定との食い違いを指摘する。decision log を仕様の正本として扱う記述も指摘する。
- `doc/design/decision-log/*.md` は 1 テーマ 1 ファイル。背景・候補・検討内容・決定・理由・影響・見直し条件のうち判断に必要なものが欠けていれば指摘する。`index.md` は入口であり、詳細議論の詰め込み、参照漏れ、有効な方針と未決事項の混同を指摘する。
- `progress.md` は横断的な作業状況の一覧である。Issue 本文や skill 手順の複製、内容が混ざって読めなくなる変更を指摘する。
- 文末は読者層で分かれる。利用者向け（repo root `README.md`）はですます調、開発者向け（`doc/` 配下、`AGENTS.md`、`progress.md`、`working-branch-notes/`、各ディレクトリの `README.md`）は常体。混在は `[nits]` で指摘する。

## Working Branch Notes のレビュー観点

`working-branch-notes/**/*.md` は作業メモであり最終仕様書ではない。note 内の細かな整合性（stale 表現、実装との 1:1 整合）は指摘しないか `[fyi]` に留める。秘密情報・個人情報の混入は必ず `[must]` で指摘する。
