# guidelines

このディレクトリには、AI agent と人間が共通で従う作業ルール、運用ガイドラインを置く。

## 置くもの

- 記録の仕方、文体、開発作業の進め方などの恒久ルール
- 1 テーマ 1 ファイル

## 一覧

| ファイル | 扱う範囲 |
|---|---|
| `agent-configuration-management.md` | skill / rule / agent 固有入口の作成・削除・rename・配置 |
| `development-loop.md` | 開発ループの入口。Issue / progress.md / PR / note の役割と流れ |
| `issue-driven-task-execution.md` | Issue 指定タスクの消化手順 |
| `git-operation-guidelines.md` | commit 署名、remote、保護ブランチの取り扱い |
| `github-mcp-guidelines.md` | GitHub 操作の MCP 優先と `gh` fallback の判断 |
| `github-cli-guidelines.md` | `gh` の実行範囲と承認が必要な操作 |
| `pull-request-guidelines.md` | PR title / description / 検証記載 |
| `working-branch-notes-handling.md` | 作業メモの性質・整合性スコープ・ライフサイクル・ファイル名規約 |
| `working-branch-notes-security.md` | 作業メモの情報統制（秘密情報・個人情報の禁則） |
| `decision-log-guidelines.md` | 方針決定ログの作成・更新・index 管理 |
| `document-style-guidelines.md` | 日本語ドキュメントの文末・トーン・用語表記 |

## 置かないもの

- プロダクトの仕様: `doc/design/` に置く
- 作業状況: `progress.md` に置く

## 追加するとき

- 新しい guideline を足したら、この README と `AGENTS.md` の共通正本一覧を更新する。
- AI agent から読ませる場合は、正本を複製せず `.cursor/rules/` と `.claude/rules/` に薄い shim を置く。Codex は `AGENTS.md` のリンクから正本へ辿る。
