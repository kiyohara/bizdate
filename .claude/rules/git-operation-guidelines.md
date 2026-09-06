# Git 操作

- 詳細は `doc/guidelines/git-operation-guidelines.md`
- commit 署名は 1Password SSH agent に依存する。`ssh-add -l` で鍵が見えない場合は `SSH_AUTH_SOCK` を 1Password の socket に向けて再実行する。
- `main` は保護されている。直接 push しない。変更は PR 経由で入れる。
- GitHub remote は HTTPS。push / fetch は SSH agent 連携の対象外。
