# 1Password 連携ルール

この文書は、bizdate リポジトリで AI agent が実行する git / GitHub 操作に 1Password が関与するときの共通正本である。1Password 連携がある環境と無い環境を同じ手順で扱うことを目的とする。

## 前提

- 1Password 連携の有無と経路はメンバーの環境に依存する。cloud session（`doc/guidelines/cloud-session-guidelines.md`）のように 1Password が無い環境もある。agent はどの環境かを事前に判定せず、各 guideline が定めるコマンドと tool をそのまま実行する。
- 1Password が関与しうる経路は次のとおり。いずれも承認ダイアログが出ることがあり、ユーザーが応答できないと操作は失敗する。

| 経路 | 使う場面 |
| --- | --- |
| commit 署名（`gpg.ssh.program` が 1Password の `op-ssh-sign` を指す構成、または `ssh-keygen -Y sign` と 1Password の SSH agent を使う構成） | `git commit` など署名を伴う操作（`doc/guidelines/git-operation-guidelines.md`） |
| SSH remote への接続（1Password の SSH agent） | `git push` / `git fetch` / `git pull` |
| `op plugin run -- gh ...` | `gh` の実行（`doc/guidelines/github-cli-guidelines.md`） |
| `op run`（GitHub MCP server の wrapper が起動時に実行する） | `github-op-integrated` の起動（`.agents/mcp/github-op-integrated/README.md`） |

## 原則

- 操作の前段にゲートを置かない。「これから 1Password の承認が必要な操作を行う」という事前の確認や合意取りをしない。
- 承認が通るかを確かめる preflight（`op whoami`、`op account list`、`ssh-add -l`、`gh auth status` など）を先に実行しない。
- skill や guideline に、承認が通るかどうかを理由に挙動を変える 1Password 固有の分岐や事前チェックを持ち込まない。`gh` の実行形式の選択（`.op/plugins/gh.json` があれば `op plugin run -- gh`。`doc/guidelines/github-cli-guidelines.md`）は、メンバーがその環境で宣言した設定に従う静的な選択であり、承認可否の判定ではないためこれに当たらない。失敗したときの扱いは本ルールを参照するだけにする。
- 影響の大きさや取り消しにくさを理由とする承認（`doc/guidelines/github-cli-guidelines.md` の「ユーザーの確認を得てから実行する操作」、PR の merge や review thread の resolve を agent が行わない規定など）は別の主題であり、本ルールは変えない。

## 1Password 起因の失敗の見分け方

次の出力は、1Password の承認待ちに起因する失敗（承認のタイムアウト、承認の拒否、app の lock、app の未起動、承認プロンプトが実行環境へ届かないこと）として扱う。文言は macOS の 1Password 8 で観測したもので、版によって変わりうる。

| 経路 | 出力の例 |
| --- | --- |
| commit 署名 | `error: 1Password: agent returned an error`、`error: 1Password: failed to fill whole buffer`、`error: 1Password: Could not connect to socket. Is the agent running?`。いずれも `fatal: failed to write commit object` を伴う |
| `op plugin run` / `op run` | `authorization timeout`、`authorization prompt dismissed, please try again`、`1Password CLI couldn't connect to the 1Password desktop app` |
| SSH remote | `agent refused operation`、`Permission denied (publickey)`。1Password の語を含まない |
| MCP server の起動 | host の汎用表示だけが見える（`CONNECT_TIMEOUT`、`connection timed out`、`Connection closed` など）。host の接続 timeout が `op` の承認 timeout より短いため、`op` の文言は届かない |

出力から 1Password 起因と断定できない場合（SSH remote、MCP server の起動、見慣れない文言）も、切り分けの調査を重ねず本ルールに従って中断する。1Password 起因でない失敗を本ルールで扱っても、報告が 1 回増えるだけである。逆に 1Password 起因の失敗を別の原因と誤認すると、後述の「やらないこと」へ進みやすい。

## 失敗したら中断する

上記の失敗を検知したら、その操作を再試行せず作業を中断し、次を報告する。

- 失敗した操作（コマンドまたは tool 名）。
- エラー出力の要旨。token、secret、認証情報付き URL、開発機の絶対 path は引用しない（`doc/guidelines/working-branch-notes-security.md`）。
- 未反映の変更。未 commit の作業ツリー、未 push の commit、未投稿の comment など、次の手番に影響するもの。

あわせて次の選択肢を提示し、ユーザーの判断を待つ。

1. ダイアログに応答できる状態にしてから、同じ操作を再実行する。MCP server は同じ操作の再実行では復旧しないため、host 側で再接続する。
2. ユーザーが手動で実行する。agent は結果の確認から再開する。
3. 1Password を伴わない別経路で進める。使う credential や署名の有無が変わるため、ユーザーが明示的に選んだ場合だけ、指示された範囲で行う。
4. 中断したまま保留する。未反映の変更はそのまま残す。

subagent のようにユーザーへ直接問えない実行主体は、選択肢を選ばず、上記の報告内容を確認事項として呼び出し元へ返す。

### 再実行の前に確認する

ユーザーの指示で再実行する場合、先に反映の有無を確かめる。

- write 系の GitHub 操作は read-back で確認する（`doc/guidelines/github-cli-guidelines.md` の「write 操作が失敗したとき」、`doc/guidelines/github-mcp-guidelines.md` の「MCP write が失敗したとき」）。
- commit の署名失敗では commit は作られない。`git log -1 --format="%H %s"` と `git status --short` で確認してから続ける。

### やらないこと

ユーザーが明示的に指示した場合を除き、次を行わない。

- 別の 1Password 連携経路への自動切り替え。MCP server の起動失敗を契機に `op plugin run -- gh ...` へ進む、などが該当する。別経路も同じ承認を要求するため、ユーザーが応答できない状況では同じ失敗を繰り返す。`doc/guidelines/github-mcp-guidelines.md` の `gh` fallback は、MCP 未設定、allowlist 外の操作、起動済みの MCP の応答失敗に限る。
- 署名の無効化。`--no-gpg-sign`、`-c commit.gpgsign=false`、`commit.gpgsign` の書き換えなど。
- 署名鍵、`gpg.ssh.program`、`SSH_AUTH_SOCK`、credential helper、remote URL、1Password の連携設定の変更。単発の `SSH_AUTH_SOCK` 明示（「署名経路の確認」）は、ユーザーが選択肢 1 を選んだ後の再実行に限る。
- 1Password app の起動、再起動、更新、再インストール。app 未起動時の `op` のエラーは更新を促すが、従わない。
- 承認待ちの再現や切り分けを目的とした、署名や認証を伴う操作の繰り返し。

## 署名経路の確認

署名の失敗を報告するとき、経路を添えると再実行の判断が早い。経路は `gpg.ssh.program` で決まる。

```sh
git config --get gpg.ssh.program
```

- 1Password の `op-ssh-sign` を指す場合、署名は 1Password app が直接処理する。SSH agent を経由しないため、`SSH_AUTH_SOCK` と `ssh-add -l` の結果は署名の可否と無関係である。
- 未設定の場合、署名は `ssh-keygen -Y sign` が行い、鍵を `SSH_AUTH_SOCK` が指す SSH agent から取る。`~/.ssh/config` の `IdentityAgent` は `ssh` 接続にだけ効き、`ssh-keygen -Y sign` には効かない。1Password の SSH agent を使う構成で socket が別の agent を指していた場合は、選択肢 1 の再実行で socket を明示する。

  ```sh
  SSH_AUTH_SOCK="$HOME/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock" git commit ...
  ```

  socket の path は macOS の例である。

## 1Password が無い環境

cloud session のように 1Password が関与しない環境では、本ルールは発動しない。cloud session で `github-op-integrated` の起動失敗（`Connection closed` など）が表示されるのは 1Password と無関係な想定どおりの挙動であり、`doc/guidelines/github-mcp-guidelines.md` の cloud session 節に従って組み込み tool を使う。環境の判定や、本ルールのための追加手順は要らない。
