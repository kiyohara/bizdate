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

承認待ちに起因する失敗として扱うのは、承認のタイムアウト（`authorization timeout` など）、承認の拒否、1Password app の lock、app の未起動、承認プロンプトが実行環境へ届かないこと（sandbox や TTY の制約）である。

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
| ダイアログに応答できる状態にしてから再実行する | 1Password app の起動と unlock、承認できる端末の前にいること、sandbox や TTY の制約がない実行環境を使うこと |
| 1Password を伴わない別経路で進める | **ユーザーが明示的に選んだ場合のみ** |
| ユーザーが手動で実行する | agent は結果の確認から再開する |
| 中断したまま保留する | 未反映の変更をそのまま残す |

### 自動で別経路へ fallback しない

1Password 起因の失敗を契機に、別の 1Password 連携経路へ自動で切り替えない。例として、MCP server の起動が `op run` の承認待ちで失敗したことを理由に `op plugin run -- gh ...` へ fallback しない。同じ承認ダイアログを再度要求するだけで、失敗の原因は解消しない。

`doc/guidelines/github-mcp-guidelines.md` の「MCP 優先・`gh` fallback」は、MCP が未設定、操作が allowlist 外、MCP が機能として応答しない場合の経路選択を定めたものである。承認待ちの失敗はこの fallback 条件に当たらない。同 guideline の「MCP write が失敗したとき」の fallback 判断よりも本ルールの中断を優先する。

### 禁止する回避策

次は実行しない。ユーザーが明示的に指示した場合だけ、指示された範囲で従う。

- 署名の無効化。`--no-gpg-sign`、`-c commit.gpgsign=false`、`commit.gpgsign` 設定の書き換えなど。
- 署名鍵、`gpg.ssh.program`、`SSH_AUTH_SOCK`、credential helper、remote URL の無断変更。
- 承認を迂回する設定変更。1Password の連携設定や `op` の sign-in 方式の変更など。

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

エラー出力から 1Password 起因か切り分けられない場合は、原因を断定せず本ルールに従って中断する。切り分けのための調査コマンドを重ねない。報告には、切り分けできなかったことと判断に使った出力を含める。

GitHub MCP server の起動失敗は、原因が MCP host に出ないことが多い。wrapper が `op run` を使う環境では、原因を特定できない起動失敗も本ルールへ寄せて中断する（`doc/guidelines/github-mcp-guidelines.md` の「起動失敗の読み分け」）。

安全側に倒す理由は、1Password 起因でない失敗に本ルールを適用しても中断と報告にしかならない一方、1Password 起因の失敗を別の原因と誤認すると、禁止している回避策や自動 fallback へ進みやすいためである。
