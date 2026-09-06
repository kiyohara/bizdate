# AI Agent ガイド

このファイルは、このリポジトリにおける AI agent 向け共通入口である。

index として利用し、これ自体を唯一の正本として扱わない。

## 共通正本

各 rule の本文はこの一覧の正本に置く。Codex は `AGENTS.md` 経由で正本に到達するため、新規 rule を作るときは必ずここに追加する。

- Decision log 記録ルール（方針決定ログの作成・更新・index 管理）: `doc/guidelines/decision-log-guidelines.md`
- ドキュメント文体ガイドライン（読者層別の文末・トーン・用語表記）: `doc/guidelines/document-style-guidelines.md`

## ドキュメント配置

- ドキュメント配置の入口: `doc/README.md`
- ガイドライン: `doc/guidelines/README.md`
- 設計文書: `doc/design/README.md`
- コンセプトの正本: `doc/design/concept.md`
- 業務日・日付・祝日の仕様: `doc/design/business-day.md`
- CLI 仕様: `doc/design/cli-interface.md`
- 進捗管理表: `progress.md`
- 方針決定ログ index: `doc/design/decision-log/index.md`
- 方針決定ログ template: `doc/design/decision-log/_template.md`

## Agent 固有の入口

対象 agent と配置規約は `doc/design/decision-log/0011-ai-agent-lineup.md` を参照する。

- AI 向け共通入口は **`AGENTS.md`**（本ファイル）。
- **Claude Code**: `CLAUDE.md`（`@AGENTS.md` で本ファイルを取り込む）と `.claude/rules/*.md` を読む。
- **Cursor**: `.cursor/rules/*.{md,mdc}` を frontmatter に従ってロードする。
- **Codex**: `AGENTS.md` から `doc/guidelines/` の正本へ移動して読む。

## AI Agent 向けルール

- ドキュメントを作成・移動・分類変更するときは、まず `doc/README.md` と該当ディレクトリの `README.md` を確認する。
- 日本語ドキュメントを作成・編集するときは `doc/guidelines/document-style-guidelines.md` に従い、読者層別の文末（利用者向けはですます調、開発者向けは常体）とトーンを守る。
- 設計判断、方針変更、重要な検討経緯を記録するときは `doc/guidelines/decision-log-guidelines.md` に従う。
- Decision log を記録するときは、まず `doc/design/decision-log/index.md` を読み、必要に応じて個別ログを作成または更新する。
- 恒久的なプロジェクト方針を agent 固有 shim にだけ書いてはならない。
- AI と人間で別ドキュメントを持たない。配置判断は `doc/README.md` と各ディレクトリの `README.md` に従う。
