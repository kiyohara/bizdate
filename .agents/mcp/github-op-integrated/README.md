# github-op-integrated MCP server

このディレクトリは、**github-op-integrated** MCP server の共通資材を置く場所である。公式の [GitHub MCP Server](https://github.com/github/github-mcp-server) Docker image を、1Password CLI(`op run`)で secret を解決しながら起動する薄い wrapper を提供する。

目的は、Claude Code / Codex / Cursor から GitHub の collaboration 操作（PR / issue / レビューコメントの read & write）を行うときに、操作ごとに `gh` を呼んで 1Password 承認ダイアログが出る状況を減らすことである。

AI agent が GitHub の PR / issue / レビューコメントを操作する場合、この server の tool が最初の試行先になる。`gh` で事前確認する前に、まず MCP host 上で `github-op-integrated` の tool が利用可能か確認する。

`gh` と MCP の使い分けは [doc/guidelines/github-mcp-guidelines.md](../../../doc/guidelines/github-mcp-guidelines.md) を参照する。MCP 共通資材の配置規約は [doc/guidelines/agent-configuration-management.md](../../../doc/guidelines/agent-configuration-management.md) の「MCP server 共通資材管理」を参照する。

## ファイル

| ファイル | commit | 役割 |
| --- | --- | --- |
| `mcp-github-op-integrated.sh` | yes | 各 MCP host から起動される wrapper script。`.config/github-op-integrated.conf` を `op run` で解決し、GitHub MCP Server の Docker image を exec する。 |
| `config-examples.md` | yes | Claude Code / Cursor / Codex の project MCP 設定と任意 override の説明。 |
| `README.md` | yes | このファイル。 |
| `.config/github-op-integrated.conf.example` | yes | project root に置く設定 template。placeholder と allowlist だけを含む。 |
| `.config/github-op-integrated.conf` | **no** | project root に置く local config file。各ユーザーの 1Password secret reference を書く。commit してはならない。 |

MCP host 用の project 設定ファイル（`.mcp.json`、`.cursor/mcp.json`、`.codex/config.toml`）は secret を含まない project 設定として commit する。個人環境に依存する絶対 path、実 vault 名、実 item 名、実 token は書かない。

## 前提条件

- [Docker](https://www.docker.com/) がインストールされ、daemon が起動していること。
- [1Password CLI(`op`)](https://developer.1password.com/docs/cli/) がインストール・認証済みで、biometric unlock または session が有効であること。
- 1Password に、このリポジトリへアクセスできる GitHub fine-grained Personal Access Token(PAT)が保存されていること。
  - Pull requests と Issues の read & write を許可する。
  - `Contents: write` は付与しない（review thread の resolve を自動実行しない方針のため。`doc/guidelines/github-mcp-guidelines.md` を参照）。

## セットアップ

1. config template をコピーし、1Password secret reference を記入する。

   ```sh
   cp .config/github-op-integrated.conf.example .config/github-op-integrated.conf
   $EDITOR .config/github-op-integrated.conf
   ```

   `op://<VAULT>/<ITEM>/<FIELD>` を、自分の 1Password 上の PAT を指す reference に置き換える。raw token は config file に書かない。`GITHUB_TOOLS` の allowlist は、レビューを経た変更でない限り初期値のまま使う。

2. wrapper が使う環境変数を確認する。token 値そのものは端末ログ・画面共有・MCP / IDE のログ収集に残るおそれがあるため、出力せず存在確認だけ行う。

   ```sh
   op run --env-file=.config/github-op-integrated.conf -- \
     sh -c '[ -n "$GITHUB_PERSONAL_ACCESS_TOKEN" ] && echo "PAT 解決済み"'
   ```

   `PAT 解決済み` と表示されれば OK。何も表示されない、または `op://...` のままになる場合は secret reference が解決できていないので、`op` の sign-in 状態と reference が指す item の存在を確認する。

3. MCP host を再起動し、tool 一覧に `github-op-integrated` の tool が現れることを確認する。

## worktree での利用

gitignored な `.config/github-op-integrated.conf` は新しい worktree へ自動配置されないため、worktree では wrapper が config file を見つけられず停止する。worktree を作成したら、その worktree の中で setup script を実行して local config を配置する。

```sh
.agents/scripts/worktree-setup.sh
```

main worktree に `.config/github-op-integrated.conf` があればコピーされる。無い場合は、上の「セットアップ」を先に main worktree で済ませてから再実行する。allowlist と script の詳細は `doc/guidelines/agent-configuration-management.md` の「worktree での ignored local config」を参照する。

## トラブルシュート

| 症状 | 確認する点 |
| --- | --- |
| `config file が見つからない` | `.config/github-op-integrated.conf` を作ったか。worktree なら `worktree-setup.sh` を実行したか。 |
| `'1Password CLI (op)' が PATH に見つからない` | `op` をインストールし、MCP host が読む PATH に含まれているか。 |
| `'docker' が PATH に見つからない` | Docker がインストールされ daemon が起動しているか。 |
| tool 一覧に現れない | MCP host を再起動したか。project の MCP 設定を有効化したか。 |
| 401 / 403 が返る | PAT の repository access にこのリポジトリが含まれるか。Pull requests / Issues の権限があるか。 |
