# GitHub CLI 実行ルール

この文書は、bizdate リポジトリで AI agent が GitHub CLI(`gh`)を実行するときの共通ルールである。

`git commit` や `git push` など local git の操作は `doc/guidelines/git-operation-guidelines.md` に従う。

## GitHub MCP との関係

PR / issue / レビューコメントの操作は、原則 `doc/guidelines/github-mcp-guidelines.md` に従って GitHub MCP Server を優先する。本ルールは次の場合に適用する。

- MCP server が未設定である。MCP server が起動しない場合は `gh` へ進まず、`doc/guidelines/github-mcp-guidelines.md` の優先順位 3 に従って中断する。
- 対象の操作が MCP の allowlist に含まれていない（merge、file push、release、workflow の実行・再実行・cancel・log 削除、repository settings 変更などが該当する）。CI の **read**（workflow / run / job / artifact / job log / check run）は MCP 側にあるので `gh` を使わない。
- MCP 経由の実行が失敗し、`gh` で再試行する必要がある。
- cloud session（Claude Code on the web）で、組み込みの GitHub MCP tool に無い操作を `gh api` で補う。範囲は `doc/guidelines/github-mcp-guidelines.md` の「cloud session」に従い、実行形式は下記「cloud session」に従う。

つまり本ルールは、GitHub MCP の fallback と、MCP 化対象外の操作に対する一次ルールである。

## 1Password op plugin 連携

`gh` を実行する前に、リポジトリ直下の `.op/` の有無と `op` コマンドの利用可否を確認する。`.op/plugins/gh.json` は `op plugin init gh` を実行したメンバーが「このリポジトリでは 1Password の gh plugin を使う」と宣言した設定であり、これに従う実行形式の選択である。承認が通るかを確かめる preflight ではない（`doc/guidelines/one-password-integration-guidelines.md`）。

`.op/` が存在し、かつ `op` が実行可能な場合、`gh ...` を直接実行せず次の形式を使う。shell alias には依存しない。

```sh
op plugin run -- gh ...
```

この経路では、このリポジトリにスコープを限定した fine-grained PAT が使われる。`.op/` が無い、または `op` が使えない場合は通常の `gh ...` を使ってよい。

`op plugin run -- gh ...` は 1Password の承認ダイアログを伴う。承認待ちで失敗した場合は `doc/guidelines/one-password-integration-guidelines.md` に従って中断する。

## cloud session（Claude Code on the web）

cloud session の sandbox には `op` が無く、`gh` は environment の setup script が入れる（`doc/guidelines/cloud-session-guidelines.md`）。

- `op plugin run --` を介さず、`gh ...` を直接実行する。`.op/` は repository に含まれるが、`op` が無いため上記「1Password op plugin 連携」の形式は使わない。
- 認証は platform の GitHub proxy が request ごとに差し替える。`gh auth login` をせず、`GH_TOKEN` / `GITHUB_TOKEN` や PAT を environment の環境変数に設定しない。
- proxy は REST だけを通す。`gh api`（REST）を基本とし、GraphQL を使う subcommand と `gh auth status` を使わない。詳細は `doc/guidelines/github-mcp-guidelines.md` の「cloud session」。
- 認証エラーを疑うときは、`gh auth status` ではなく `gh api user` など REST の read で確かめる。
- `gh` が無い session では、agent が `gh` の導入を試みない。組み込み tool だけで進め、足りない操作はユーザーに報告する。
- 本ルールの「ユーザーの確認を得てから実行する操作」と「agent が実行しない操作」は cloud session でもそのまま適用する。加えて、`APPROVE` / `REQUEST_CHANGES`、review thread の resolve、auto-merge の変更、API 経由の file push は、`gh api`（proxy の専用 route を含む）でも実行しない（`doc/guidelines/github-mcp-guidelines.md` の「Review event と resolve の制約」と「cloud session」）。

## 認証

認証エラーが出たときは次で状態を確認する。preflight としては実行しない。cloud session では上記「cloud session」に従う。

```sh
op plugin run -- gh auth status
```

token の値そのものを出力・log しない。認証が切れている場合は、agent が再認証せずユーザーに依頼する。

## agent が実行してよい操作

- read 全般: `gh pr view` / `gh pr list` / `gh issue view` / `gh api` の GET / `gh run list` など（cloud session では GraphQL を使う subcommand は使えない）。
- Issue の作成・更新・コメント。
- PR の作成・更新・コメント・レビューコメントへの返信。

## ユーザーの確認を得てから実行する操作

次は影響が大きい、または取り消しにくい。実行前に、何をどう変えるかを提示してユーザーの承認を得る。

- リポジトリ設定の変更（`gh api --method PATCH /repos/...`、visibility、branch protection / ruleset、merge 設定など）
- GitHub Release の作成・更新・削除
- workflow の実行・再実行・cancel・run log の削除（`gh workflow run` / `gh run rerun` / `gh run cancel` / `gh api --method DELETE .../logs`）
- Issue / PR / comment の削除
- label / milestone の一括変更
- commit status の作成（`gh api --method POST .../statuses/<sha>`。CI の結果と見分けにくく、merge 判断を誤らせ得る）

## agent が実行しない操作

- **PR の merge**（`gh pr merge`）。レビューと merge 判断は人間が行う（`doc/guidelines/development-loop.md`）。
- リポジトリの削除。
- secrets の登録・変更・削除。

## write 操作が失敗したとき

`gh` の write が失敗した場合、同じコマンドをそのまま再実行しない。二重作成・二重コメントを避けるため、次の順で扱う。

1. read 系コマンドで、対象が部分的に作成・更新されていないか確認する。
2. 未反映であることを確認してから再実行する。
3. 反映済みだった場合は、再実行せず更新系コマンドに切り替える。

## 出力の扱い

- `gh` の出力に token、個人情報、認証情報付き URL が含まれる場合、そのまま note や PR description に貼らない（`doc/guidelines/working-branch-notes-security.md`）。
- 必要な部分だけを引用し、識別子は最小限にする。
