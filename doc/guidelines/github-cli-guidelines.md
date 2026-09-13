# GitHub CLI 実行ルール

この文書は、bizdate リポジトリで AI agent が GitHub CLI(`gh`)を実行するときの共通ルールである。

`git commit` や `git push` など local git の操作は `doc/guidelines/git-operation-guidelines.md` に従う。

## GitHub MCP との関係

PR / issue / レビューコメントの操作は、原則 `doc/guidelines/github-mcp-guidelines.md` に従って GitHub MCP Server を優先する。本ルールは次の場合に適用する。

- MCP server が未設定、または現在の環境で利用できない。
- 対象の操作が MCP の allowlist に含まれていない（merge、file push、release、workflow の実行・再実行・cancel・log 削除、repository settings 変更などが該当する）。CI の **read**（workflow / run / job / artifact / job log / check run）は MCP 側にあるので `gh` を使わない。
- MCP 経由の実行が失敗し、`gh` で再試行する必要がある。

つまり本ルールは、GitHub MCP の fallback と、MCP 化対象外の操作に対する一次ルールである。

## 1Password op plugin 連携

`gh` を実行する前に、リポジトリ直下の `.op/` の有無と `op` コマンドの利用可否を確認する。

`.op/` が存在し、かつ `op` が実行可能な場合、`gh ...` を直接実行せず次の形式を使う。shell alias には依存しない。

```sh
op plugin run -- gh ...
```

この経路では、このリポジトリにスコープを限定した fine-grained PAT が使われる。`.op/` が無い、または `op` が使えない場合は通常の `gh ...` を使ってよい。

この確認は、承認が通るかを確かめる preflight ではなく、`gh` の実行形式（リポジトリにスコープを限定した PAT を使う経路かどうか）の選択である。`doc/guidelines/one-password-approval-failure.md` が禁じる preflight には当たらないため、そのまま維持する。

`op plugin run -- gh ...` は 1Password app との連携を必要とする。承認プロンプトが届かない、または承認待ちでタイムアウトする場合の扱いは `doc/guidelines/one-password-approval-failure.md` に従う。承認が通るかを確かめる preflight は実行せず、通常どおり実行して失敗した時点で中断する。

## 認証

認証切れが疑われる場合に限り、次で状態を確認する。`gh` 実行の preflight として毎回先に実行しない（`doc/guidelines/one-password-approval-failure.md`）。

```sh
op plugin run -- gh auth status
```

token の値そのものを出力・log しない。認証が切れている場合は、agent が再認証せずユーザーに依頼する。

## agent が実行してよい操作

- read 全般: `gh pr view` / `gh pr list` / `gh issue view` / `gh api` の GET / `gh run list` など。
- Issue の作成・更新・コメント。
- PR の作成・更新・コメント・レビューコメントへの返信。

## ユーザーの確認を得てから実行する操作

次は影響が大きい、または取り消しにくい。実行前に、何をどう変えるかを提示してユーザーの承認を得る。

- リポジトリ設定の変更（`gh api --method PATCH /repos/...`、visibility、branch protection / ruleset、merge 設定など）
- GitHub Release の作成・更新・削除
- workflow の実行・再実行・cancel・run log の削除（`gh workflow run` / `gh run rerun` / `gh run cancel` / `gh api --method DELETE .../logs`）
- Issue / PR / comment の削除
- label / milestone の一括変更

## agent が実行しない操作

- **PR の merge**（`gh pr merge`）。レビューと merge 判断は人間が行う（`doc/guidelines/development-loop.md`）。
- リポジトリの削除。
- secrets の登録・変更・削除。

## write 操作が失敗したとき

1Password の承認待ちに起因する失敗は `doc/guidelines/one-password-approval-failure.md` に従って中断する。別の 1Password 連携経路へ自動で切り替えない。

それ以外の理由で `gh` の write が失敗した場合、同じコマンドをそのまま再実行しない。二重作成・二重コメントを避けるため、次の順で扱う。

1. read 系コマンドで、対象が部分的に作成・更新されていないか確認する。
2. 未反映であることを確認してから再実行する。
3. 反映済みだった場合は、再実行せず更新系コマンドに切り替える。

## 出力の扱い

- `gh` の出力に token、個人情報、認証情報付き URL が含まれる場合、そのまま note や PR description に貼らない（`doc/guidelines/working-branch-notes-security.md`）。
- 必要な部分だけを引用し、識別子は最小限にする。
