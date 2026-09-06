# 作業ブランチメモ

- ブランチ: `add-github-mcp-integration`
- PR: #3
- 最終更新: 2026-09-06

## 目的

開発環境整備フェーズ 3 として、GitHub MCP 連携を導入する。GitHub の PR / Issue / レビューコメント操作を MCP 経由で行えるようにし、`gh` 実行のたびに 1Password の承認ダイアログが出る状況を減らす。

対象は次の 6 つ。

1. MCP server 共通資材（`.agents/mcp/github-op-integrated/`）
2. 利用ルール（`doc/guidelines/github-mcp-guidelines.md`）と tool 固有入口
3. secret-free な config template（`.config/github-op-integrated.conf.example`）
4. 各 tool の project MCP 設定（`.mcp.json` / `.cursor/mcp.json` / `.codex/config.toml`）
5. worktree 用 local config 配置（`.worktreeinclude` / `.agents/scripts/worktree-setup.sh`）
6. フェーズ 1・2 で保留した記述の解消

## 現在の状況

共通資材、guideline、各 tool 設定、worktree 補助の配置と、保留していた記述の解消を完了した。MCP server の起動を実地で確認済み。PR 作成待ち。

## 決定事項

### フェーズ 1・2 で保留した記述をここで解消する

- `agent-configuration-management.md` に MCP server 共通資材の管理セクションと worktree 用 local config のセクションを追加する（PR #1 で「実体を導入するフェーズ 3 に送る」と決めていた分）。
- `issue-driven-task-execution.md` の GitHub 操作を「`gh` に従う」から「MCP を優先し、`gh` は fallback」へ更新する（PR #2 で予告した分）。
- `github-cli-guidelines.md` に、MCP との優先順位と、MCP 化対象外の操作を `gh` で行う旨を追加する。

### secret の扱い

- PAT の実値をリポジトリに置かない。`.config/github-op-integrated.conf` に 1Password secret reference（`op://...`）だけを書き、wrapper が `op run --env-file` で解決する。
- `.config/*.conf` は `.gitignore` 済み。commit するのは `.config/github-op-integrated.conf.example`（placeholder のみ）。
- PAT は slapex 用の fine-grained PAT にアクセス可能リポジトリとして bizdate を追加したものを使う。allowlist に `Contents: write` は含めない。

### tool allowlist

`GITHUB_TOOLS` で明示 allowlist を使う。`GITHUB_TOOLSETS` による広い有効化はしない。thread resolve は allowlist に含めるが、運用としては自動実行しない（`Contents: Read and Write` を付与しない方針のため）。

## 次にやること

- 共通資材、guideline、各 tool 設定、worktree 補助を配置する。
- 保留していた記述を解消する。
- MCP server の起動を実地で確認する。
- PR を作成し、note を採番する。

## 検証

- **MCP server の起動（end-to-end）**: wrapper に JSON-RPC の `initialize` / `tools/list` を stdio 越しに流し、`github-mcp-server v1.0.4` が応答することを確認。返った tool は **13 件**で、`.config/github-op-integrated.conf.example` の `GITHUB_TOOLS` allowlist と完全一致した（`add_comment_to_pending_review` / `add_issue_comment` / `add_reply_to_pull_request_comment` / `create_pull_request` / `issue_read` / `issue_write` / `list_issues` / `list_pull_requests` / `pull_request_read` / `pull_request_review_write` / `search_issues` / `search_pull_requests` / `update_pull_request`）。allowlist が実際に効いている。
- **fail-loud の確認**: local config が無い状態で wrapper を起動し、config file の不在と復旧手順を stderr に出して停止することを確認した。
- **PAT の解決**: `op run --env-file` 経由で `GITHUB_PERSONAL_ACCESS_TOKEN` が解決されることを、値を出力せず存在確認のみで確認した。
- **rule basename の整合**: 11 本すべてについて 5 箇所（共通正本 / `.cursor/rules/` / `.claude/rules/` / `AGENTS.md` / `doc/guidelines/README.md`）が揃うことを確認。
- **参照の解決**: `.agents/` 配下を含めて走査し、未解決 0 件。
- **secret 混入の確認**: commit 対象の `.config/` 配下は `github-op-integrated.conf.example` のみ。staged diff に `<VAULT>` placeholder 以外の `op://` 参照が無いことを確認した。`.config/github-op-integrated.conf` は gitignored。
- 実行可能なアプリケーションコードを含まないため、アプリのテストは実行していない。

## リスク・ブロッカー

- wrapper は Docker を必要とする。Docker が停止していると MCP server が起動しない。README のトラブルシュート表に記載した。
- MCP host（Claude Code / Cursor / Codex）は起動時に MCP 設定を読むため、本 PR の設定を反映するには host の再起動が必要である。作業中のセッションでは即座に tool が生えない。
- Claude Code で `.mcp.json` の server を毎セッション自動有効化するには、gitignored な `.claude/settings.local.json` に `enabledMcpjsonServers` を書く必要がある。tool の permission 事前付与を含むため本 PR では作成せず、利用者の判断に委ねる。

## セッションログ

- 2026-09-06: PR #2 merge 後に main から分岐。本 note を作成した。
- 2026-09-06: PR #3 を作成し、note を採番した。
- 2026-09-06: 共通資材（wrapper / README / config-examples）、`github-mcp-guidelines.md` と入口、各 tool の MCP 設定、`.worktreeinclude` と `worktree-setup.sh` を配置。
- 2026-09-06: フェーズ 1 で送った MCP 管理・worktree セクションを `agent-configuration-management.md` へ追加。フェーズ 2 で予告した `issue-driven-task-execution.md` の MCP 優先化と `github-cli-guidelines.md` の位置づけ更新も実施した。
- 2026-09-06: MCP server の起動を実地確認。allowlist 13 tool が一致した。
