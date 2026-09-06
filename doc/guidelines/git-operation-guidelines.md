# Git 操作ルール

この文書は、bizdate リポジトリで AI agent が `git` コマンドを実行するときの共通ルールである。

## このリポジトリの前提

- GitHub remote は `origin`（**HTTPS**: `https://github.com/kiyohara/bizdate.git`）。push / fetch / pull は `gh` の credential helper で認証するため、SSH agent 連携の対象外である。
- `cursor-origin` remote は参照用に残してある。通常の作業では使わない。
- commit 署名が有効（`commit.gpgsign = true`、`gpg.format = ssh`）。署名鍵は 1Password の SSH agent が保持する。
- `main` は保護されている。直接 push できない。変更は必ず PR 経由で入れる。

## 1Password 連携が必要な操作

署名を伴う次の操作は、1Password SSH agent との連携に依存する。

- `git commit`
- merge commit を作る `git merge`
- `git rebase --continue`
- `git cherry-pick`
- `git tag -s`

AI agent の実行環境、sandbox、権限分離、TTY 設定などにより、agent socket への接続や承認プロンプトが阻害される場合がある。

### 署名が通らないときの確認手順

`error: Load key ... agent refused operation`、`failed to write commit object`、承認プロンプト不達などが出た場合は、次の順で確認する。

1. 現在の環境で agent に鍵が見えているか確認する。

   ```sh
   ssh-add -l
   ```

2. `The agent has no identities.` と出る場合、`SSH_AUTH_SOCK` が 1Password の agent を指していない。1Password の socket を明示して再実行する。

   ```sh
   SSH_AUTH_SOCK="$HOME/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock" git commit ...
   ```

3. それでも通らない場合は、制約のない実行環境で同じコマンドを再実行する。

`~/.ssh/config` の `IdentityAgent` は `ssh` 接続には効くが、署名に使う `ssh-keygen -Y sign` は `SSH_AUTH_SOCK` を見る。両者は別経路である。

### 署名の確認

commit 後に署名が付いたことを確認する。`G` が返れば good signature である。

```sh
git log -1 --format="%G? %GS"
```

## 通常の実行環境でよい操作

次のようなローカル参照・差分確認は、原則として通常の実行環境で実行してよい。

- `git status` / `git diff` / `git log` / `git show`
- `git branch --show-current` / `git rev-parse`
- `git switch` / `git add`

## やらないこと

- `main` への直接 push。
- `main` への force push（ruleset で禁止している）。
- PR の merge（`doc/guidelines/development-loop.md`）。
- `cursor-origin` remote への push。

## 関連ルール

`gh pr create`、`gh run view` など、`git` コマンドではなく GitHub CLI(`gh`)を使う場合は `doc/guidelines/github-cli-guidelines.md` に従う。
