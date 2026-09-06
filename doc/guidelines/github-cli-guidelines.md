# GitHub CLI 実行ルール

この文書は、bizdate リポジトリで AI agent が GitHub CLI(`gh`)を実行するときの共通ルールである。

PR / Issue / レビューコメント / workflow など、GitHub 上のリソースを操作する場合はこのルールに従う。`git commit` や `git push` など local git の操作は `doc/guidelines/git-operation-guidelines.md` に従う。

## 認証

`gh` は `gh auth login` 済みの認証情報を使う。実行前に確認する場合は次を使う。

```sh
gh auth status
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
- workflow の手動実行（`gh workflow run`）
- Issue / PR / comment の削除
- label / milestone の一括変更

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
