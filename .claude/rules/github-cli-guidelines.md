# GitHub CLI 実行

- 詳細は `doc/guidelines/github-cli-guidelines.md`
- リポジトリ設定変更、Release、workflow 実行、削除系はユーザーの承認を得てから実行する。
- PR の merge、リポジトリ削除、secrets 操作は agent が実行しない。
- write が失敗したら、そのまま再実行せず read で反映有無を確認する。
