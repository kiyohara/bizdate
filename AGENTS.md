# AI Agent ガイド

このファイルは、このリポジトリにおける AI agent 向け共通入口である。

index として利用し、これ自体を唯一の正本として扱わない。

## 共通正本

各 rule の本文はこの一覧の正本に置く。Codex は `AGENTS.md` 経由で正本に到達するため、新規 rule を作るときは必ずここに追加する。

- Agent 設定管理ルール（skill / rule / agent 固有入口の作成・削除・rename・配置）: `doc/guidelines/agent-configuration-management.md`
- Pull Request 作成ガイドライン: `doc/guidelines/pull-request-guidelines.md`
- Working branch notes 取り扱いルール（性質・整合性スコープ・ライフサイクル・ファイル名規約）: `doc/guidelines/working-branch-notes-handling.md`
- Working branch notes 情報統制ルール（`working-branch-notes/**/*.md` のセキュリティ禁則）: `doc/guidelines/working-branch-notes-security.md`
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

## 作業プロセスドキュメント

- 作業ブランチメモ: `working-branch-notes/README.md`

## Agent 固有の入口

対象 agent と配置規約は `doc/design/decision-log/0011-ai-agent-lineup.md` を参照する。

- AI 向け共通入口は **`AGENTS.md`**（本ファイル）。
- **Claude Code**: `CLAUDE.md`（`@AGENTS.md` で本ファイルを取り込む）と `.claude/rules/*.md` を読む。
- **Cursor**: `.cursor/rules/*.{md,mdc}` を frontmatter に従ってロードする。
- **Codex**: `AGENTS.md` から `doc/guidelines/` の正本へ移動して読む。
- **GitHub Copilot code review**: `.github/copilot-instructions.md` をレビュー時に読む。Copilot は `AGENTS.md` やリンク先正本を辿らないため、効かせたい要点は Copilot 用ファイル内に直接書く。

## AI Agent 向けルール

- AI agent 用の設定ファイル、rule、skill、agent 固有入口を作成・削除・rename するときは `doc/guidelines/agent-configuration-management.md` に従う。
- PR を作成または更新するときは `doc/guidelines/pull-request-guidelines.md` に従う。PR の merge は agent が行わない。
- `working-branch-notes/**/*.md` を作成・編集・レビューするときは `doc/guidelines/working-branch-notes-handling.md` と `doc/guidelines/working-branch-notes-security.md` の両方に従う。
- ドキュメントを作成・移動・分類変更するときは、まず `doc/README.md` と該当ディレクトリの `README.md` を確認する。
- 日本語ドキュメントを作成・編集するときは `doc/guidelines/document-style-guidelines.md` に従い、読者層別の文末（利用者向けはですます調、開発者向けは常体）とトーンを守る。
- 設計判断、方針変更、重要な検討経緯を記録するときは `doc/guidelines/decision-log-guidelines.md` に従う。
- Decision log を記録するときは、まず `doc/design/decision-log/index.md` を読み、必要に応じて個別ログを作成または更新する。
- 恒久的なプロジェクト方針を agent 固有 shim にだけ書いてはならない。
- AI と人間で別ドキュメントを持たない。配置判断は `doc/README.md` と各ディレクトリの `README.md` に従う。
