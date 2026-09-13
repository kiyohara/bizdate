# Git 操作ルール

この文書は、bizdate リポジトリで AI agent が `git` コマンドを実行するときの共通ルールである。

## このリポジトリの前提

- GitHub remote は `origin`（`github.com/kiyohara/bizdate`）。URL の scheme（HTTPS / SSH）と push / fetch / pull の認証経路（credential helper / SSH agent）は clone 時の選択とメンバーの設定に依存するため、どちらかを前提にしない。
- `cursor-origin` remote は参照用に残してある。通常の作業では使わない。
- commit 署名が有効（`commit.gpgsign = true`、`gpg.format = ssh`）。署名鍵の保管場所と署名の実行経路はメンバーの環境に依存する。
- `main` は保護されている。直接 push できない。変更は必ず PR 経由で入れる。

## commit 署名

署名を伴う操作は次のとおり。

- `git commit`
- merge commit を作る `git merge`
- `git rebase --continue`
- `git cherry-pick`
- `git tag -s`

署名や remote への接続が失敗し、出力に 1Password や承認に関する文言がある場合、または原因を判断できない場合は、`doc/guidelines/one-password-integration-guidelines.md` に従って中断する。署名経路の確認手順と、再実行前の確認も同ガイドラインに置く。

### 署名の確認

commit 後に署名が付いたことを確認する。`G` が返れば good signature である。

```sh
git log -1 --format="%G? %GS"
```

## cloud session（Claude Code on the web）

- commit 署名と author は platform が管理する（`gpg.ssh.program` は platform の signer を指す）。1Password は関与しない。allowed signers が無いため `git log --format=%G?` は `G` を返さないので、署名の確認は省略する。
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
