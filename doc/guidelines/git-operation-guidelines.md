# Git 操作ルール

この文書は、bizdate リポジトリで AI agent が `git` コマンドを実行するときの共通ルールである。

## このリポジトリの前提

- GitHub remote は `origin`（**HTTPS**: `https://github.com/kiyohara/bizdate.git`）。push / fetch / pull は `gh` の credential helper で認証するため、SSH agent 連携の対象外である。
- `cursor-origin` remote は参照用に残してある。通常の作業では使わない。
- commit 署名が有効（`commit.gpgsign = true`、`gpg.format = ssh`）。署名鍵は 1Password が保持する。実行経路は `gpg.ssh.program` の設定によって変わる（後述）。
- `main` は保護されている。直接 push できない。変更は必ず PR 経由で入れる。

## commit 署名

署名を伴う操作は次のとおり。

- `git commit`
- merge commit を作る `git merge`
- `git rebase --continue`
- `git cherry-pick`
- `git tag -s`

署名の実行経路は `gpg.ssh.program` の設定で変わる。切り分けは必ずこの確認から始める。

```sh
git config --get gpg.ssh.program
```

### 1Password の signer を使う場合

`gpg.ssh.program` が 1Password の `op-ssh-sign` を指している場合、署名は 1Password app が直接処理する。**SSH agent を経由しないため、`SSH_AUTH_SOCK` を設定する必要はない。** `ssh-add -l` の結果は署名の可否と無関係なので、これを根拠に原因を判断しない。

署名に失敗する場合は次を確認する。

- 1Password app が起動し、unlock されているか。
- 署名の承認プロンプトが表示され、応答できる状態か。AI agent の実行環境、sandbox、TTY 設定によっては承認プロンプトが届かない場合がある。
- `gpg.ssh.program` が指す実行ファイルが存在するか。

解決できない場合は、制約のない実行環境で同じコマンドを再実行する。

### 標準の ssh-keygen を使う場合

`gpg.ssh.program` が未設定の場合、署名は `ssh-keygen -Y sign` が行い、鍵を SSH agent から取得する。この経路では `SSH_AUTH_SOCK` が対象の agent を指している必要がある。

```sh
ssh-add -l
```

署名鍵が一覧に出ない場合、その agent には利用可能な鍵がない。接続先が違うとは限らないため、まず使う agent を確定させる。1Password の SSH agent を使う構成なら、socket を明示して再実行する。

```sh
SSH_AUTH_SOCK="$HOME/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock" git commit ...
```

`~/.ssh/config` の `IdentityAgent` は `ssh` 接続には効くが、`ssh-keygen -Y sign` は `SSH_AUTH_SOCK` を見る。両者は別経路である。

### 署名の確認

commit 後に署名が付いたことを確認する。`G` が返れば good signature である。

```sh
git log -1 --format="%G? %GS"
```

## cloud session（Claude Code on the web）

- commit 署名と author は platform が管理する（`gpg.ssh.program` は platform の signer を指す）。1Password は関与せず、「commit 署名」の切り分けは適用しない。allowed signers が無いため `git log --format=%G?` は `G` を返さないので、署名の確認は省略する。
- 作業ブランチは session 作成時に platform が決めたもの（`claude/<slug>` の形）に固定され、push はそのブランチにだけ許可される。別名のブランチを作らない。
- remote は HTTPS のまま platform の proxy が認証する。`main` への直接 push 禁止と PR 経由の原則は変わらない。

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
