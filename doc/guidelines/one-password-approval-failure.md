# 1Password 承認待ち失敗ルール

この文書は、bizdate リポジトリで AI agent が 1Password 連携を伴う操作を実行し、承認待ちに起因して失敗したときの共通正本である。

方針は 1 つである。**前段にゲートを置かず、失敗した時点で中断してユーザーに選択肢を提示する。**

## 適用範囲

開発環境によっては、git / GitHub 操作の複数の経路で 1Password の承認ダイアログが出る。どの経路を使うかはメンバーの環境に依存するため、いずれか 1 つを前提にしない。

| 経路 | 詳細 |
| --- | --- |
| commit 署名を 1Password の signer（`op-ssh-sign`）で行う | `doc/guidelines/git-operation-guidelines.md` |
| commit 署名を `ssh-keygen -Y sign` と 1Password の SSH agent で行う | 同上 |
| GitHub remote が SSH で、1Password の SSH agent が接続を担う | 同上 |
| `op plugin run -- gh ...` | `doc/guidelines/github-cli-guidelines.md` |
| GitHub MCP server の wrapper が起動時に実行する `op run` | `.agents/mcp/github-op-integrated/README.md` |

承認待ちに起因する失敗として扱うのは、承認のタイムアウト、承認の拒否、1Password app の lock、app の未起動、承認プロンプトが実行環境へ届かないこと（sandbox や TTY の制約）である。

出力は経路ごとに違う。macOS + 1Password 8.12 での実測例を挙げる。**lock と拒否は同じ文言になるため区別できない。本ルールは同じ扱いなので支障はない。**

| 経路 | 観測された出力 |
| --- | --- |
| commit 署名（`op-ssh-sign`） | `error: 1Password: agent returned an error`（lock / 拒否）、`error: 1Password: failed to fill whole buffer`（タイムアウト）、`error: 1Password: Could not connect to socket. Is the agent running?`（app 未起動）。いずれも `fatal: failed to write commit object` を伴う |
| `op plugin run -- gh` / `op run` | `authorization prompt dismissed, please try again`（lock / 拒否）、`authorization timeout`（タイムアウト）、`1Password CLI couldn't connect to the 1Password desktop app ...`（app 未起動） |
| MCP server の起動 | host の汎用表示だけが見える。`op` の文言は届かない（`doc/guidelines/github-mcp-guidelines.md` の「起動失敗の読み分け」） |
| SSH agent 経由 | `agent refused operation` と `Permission denied (publickey)`。**1Password の語を含まないため、出力だけでは本ルールの対象と判定できない**（「1Password 起因か判断できない場合」へ落ちる） |

`error: 1Password:` や `authorization ...` のように 1Password 由来と分かる文言は判定に使える。文言が無い経路は断定せず、「1Password 起因か判断できない場合」に従う。

1Password が関与しない実行環境では、このルールは発動しない。cloud session（`doc/guidelines/cloud-session-guidelines.md`）の sandbox には `op` が無く、commit 署名も platform が行うため対象外である。

## 前段にゲートを置かない

- 1Password 連携を伴う操作の前に、ユーザーの合意を取るゲートを置かない。操作は通常どおり実行する。
- 承認が通るかを確かめる目的の事前確認をしない。`op whoami`、`op account list`、`ssh-add -l`、`gh auth status` などを preflight として先に実行しない。
- skill に `op` コマンドの有無や `.op/` の存在による 1Password 固有の条件分岐を持たせない。skill からは本ルールを参照するだけにする。

理由は 2 つある。事前ゲートは 1Password 連携が無い環境でも人間の手番を作り、`drive-issue-to-reviewed-pr` のような自律フローを止める。環境を判定するための条件分岐は、不要な事前チェックを誘発する。

影響の大きさや取り消しにくさを理由とする承認は別の主題であり、本ルールは撤回しない。`doc/guidelines/github-cli-guidelines.md` の「ユーザーの確認を得てから実行する操作」、PR の merge と inline thread の resolve を agent が行わない規定はそのまま適用する。

## 失敗したら中断する

承認待ちに起因する失敗を検知したら、その場で作業を中断する。次を報告する。

- 失敗した操作（コマンドまたは tool 名）。
- エラー出力の要旨。token、secret、認証情報付き URL の値は引用しない（`doc/guidelines/working-branch-notes-security.md`）。
- 未反映の変更。未 commit の作業ツリー、未 push の commit、未投稿の comment など、次の手番に影響するものを挙げる。

あわせて次の選択肢を提示し、ユーザーの判断を待つ。

| 選択肢 | 補足 |
| --- | --- |
| ダイアログに応答できる状態にしてから再実行する | 1Password app の起動と unlock、承認できる端末の前にいること、sandbox や TTY の制約がない実行環境を使うこと。MCP server は同じ操作の再実行では復旧しない。host 側で再接続する（MCP 管理画面の reconnect、または session の作り直し）。**host の接続 timeout は `op` の承認 timeout より短く、host が諦めた後もダイアログは残る。そのダイアログを承認しても接続は回復しない** |
| 1Password を伴わない別経路で進める | **ユーザーが明示的に選んだ場合のみ。** 経路によって使う credential が変わる。`.op/` を介さない `gh` はリポジトリにスコープを限定した PAT を使わないため、何がどう変わるかを示してユーザーに選ばせる。agent が黙って切り替えない |
| ユーザーが手動で実行する | agent は結果の確認から再開する |
| 中断したまま保留する | 未反映の変更をそのまま残す |

### 自動で別経路へ fallback しない

1Password 起因の失敗を契機に、別の 1Password 連携経路へ自動で切り替えない。例として、MCP server の起動が `op run` の承認待ちで失敗したことを理由に `op plugin run -- gh ...` へ fallback しない。

別経路も同じ承認を要求する。実測では、直前に承認済みでも app を lock した後の `op plugin run -- gh` は再度承認を求めて失敗した（session cache では通らない）。したがってユーザーが応答できない状況のままでは、経路を変えても同じ失敗を繰り返す。応答できる状況であれば別経路は成功しうるが、その切り替えはユーザーが判断する。agent が自動で行うと、操作主体（MCP の PAT と op plugin の PAT）が黙って入れ替わる。

`doc/guidelines/github-mcp-guidelines.md` の「MCP 優先・`gh` fallback」は、MCP が未設定、操作が allowlist 外、MCP が機能として応答しない場合の経路選択を定めたものである。承認待ちの失敗はこの fallback 条件に当たらない。同 guideline の「MCP write が失敗したとき」の fallback 判断よりも本ルールの中断を優先する。

### 禁止する回避策

次は実行しない。ユーザーが明示的に指示した場合だけ、指示された範囲で従う。

- 署名の無効化。`--no-gpg-sign`、`-c commit.gpgsign=false`、`commit.gpgsign` 設定の書き換えなど。
- 署名鍵、`gpg.ssh.program`、credential helper、remote URL の無断変更。`SSH_AUTH_SOCK` の永続的な付け替えも含む（`doc/guidelines/git-operation-guidelines.md` が定める単発の socket 明示は、署名経路を確定させる手順なので対象外）。
- 承認を迂回する設定変更。1Password の連携設定や `op` の sign-in 方式の変更など。
- 1Password app の更新、再インストール、再起動。app 未起動時の `op` のエラーは更新を促す文言を返すが、それに従わない。app の起動と unlock は選択肢としてユーザーに委ねる。

## 再実行の前に確認する

ユーザーの指示で再実行する場合、先に反映の有無を確かめる。

- write 系の操作は read-back で確認する（`doc/guidelines/github-cli-guidelines.md` の「write 操作が失敗したとき」、`doc/guidelines/github-mcp-guidelines.md` の「MCP write が失敗したとき」）。
- commit の署名失敗では、commit が作られていないことを確認する。署名だけが失敗した場合と commit が作られた場合で続きの操作が変わる。

```sh
git log -1 --format="%H %s"
git status --short
```

確認せずに同じ操作を再実行しない。

## ユーザーへ直接問えない実行主体

subagent のように、ユーザーへ直接問えない実行主体が承認待ちの失敗に当たった場合は、選択肢をその場で選ばない。処理を中断し、「失敗したら中断する」の報告内容を確認事項として呼び出し元へ返す。呼び出し元がユーザーへ中継する（`.agents/skills/drive-issue-to-reviewed-pr/SKILL.md` の「停止とエスカレーション」）。

## 1Password 起因か判断できない場合

エラー出力から 1Password 起因か切り分けられない場合は、原因を断定せず本ルールに従って中断する。報告には、切り分けできなかったことと判断に使った出力を含める。

切り分けのための調査コマンドを重ねない。例外は、**1Password に触れない確認**である。これは承認を要求しないため、失敗後に行ってよい。`command -v op` のように `op` が PATH にあるかを見るだけの確認はこちらに入る（`op` を起動しないため承認を要求しない）。

一方、`op` を実際に走らせる操作（`op whoami`、`op run`、`op plugin run -- ...` など）、1Password を経由する操作の再試行、署名や認証を伴う操作は、切り分けの目的で繰り返さない。

GitHub MCP server の起動失敗では、原因は MCP host に出ない。wrapper が `op run` を使う環境では、1Password に触れない確認で原因が見つからない起動失敗も、承認待ちと断定せず本ルールへ寄せて中断する（`doc/guidelines/github-mcp-guidelines.md` の「起動失敗の読み分け」に確認する項目と、断定しない理由を挙げている）。

SSH agent 経由の失敗も、出力に 1Password の語が無いためここへ落ちる。**承認待ちかどうかを切り分ける目的で**、`~/.ssh/config` の `IdentityAgent` や agent の実体を調べに行かず、中断して報告する。

署名経路の確定は別目的であり、本ルールは禁じない。`gpg.ssh.program` の確認、`ssh-add -l`、socket を明示した再実行は `doc/guidelines/git-operation-guidelines.md` の「commit 署名」が定める手順である。どちらの経路を使っているかを確定させてから、その経路の失敗が承認待ちかを本ルールで判断する。

安全側に倒す理由は、1Password 起因でない失敗に本ルールを適用しても中断と報告にしかならない一方、1Password 起因の失敗を別の原因と誤認すると、禁止している回避策や自動 fallback へ進みやすいためである。
