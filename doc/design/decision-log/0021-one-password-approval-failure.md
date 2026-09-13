<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 1Password 承認待ち失敗時の扱い

- 状態: decided
- 作成日: 2026-09-13
- 最終更新日: 2026-09-13
- 関連: [0011-ai-agent-lineup.md](0011-ai-agent-lineup.md), [0018-cloud-session-environment.md](0018-cloud-session-environment.md), [0020-note-numbering-consent-gate.md](0020-note-numbering-consent-gate.md), `doc/guidelines/one-password-approval-failure.md`

## 背景

開発環境によっては、git / GitHub 操作の複数の経路で 1Password の承認ダイアログが出る。commit 署名の signer（`op-ssh-sign`）、`ssh-keygen -Y sign` と 1Password の SSH agent、SSH remote での push / fetch、`op plugin run -- gh ...`、GitHub MCP server の wrapper が起動時に実行する `op run` がそれに当たる。どの経路を使うかはメンバーの環境に依存する。

ユーザーがマシンの前にいないとダイアログに応答できず、操作はタイムアウトで失敗する。実例として、`op run` が `authorization timeout` で 2 回失敗した記録がある（`working-branch-notes/3_add-github-mcp-integration.md`）。

これを避ける手段として、別の参考プロジェクトでは 1Password 連携を伴う操作の前段にユーザー合意ゲートを置いてきた。一方で本リポジトリの現行記述は、失敗時の扱いを「制約のない実行環境で同じコマンドを再実行する」に留めている。これは sandbox や TTY の制約を想定した記述で、ユーザー不在でタイムアウトした場合を定めていない。

なお本ログは 1Password の承認ダイアログに由来するゲートだけを扱う。成果物の内容判断を理由とするゲートは [0020-note-numbering-consent-gate.md](0020-note-numbering-consent-gate.md) が、影響の大きさや取り消しにくさを理由とする承認は `doc/guidelines/github-cli-guidelines.md` が扱う。

## 候補

1. 事前ゲート案: 1Password 連携を伴う操作の前にユーザー合意を取る。
2. 事後中断案: 操作は通常どおり実行し、承認待ちに起因して失敗した時点で中断してユーザーに選択肢を提示する。
3. 環境判定案: `op` の有無などで 1Password 連携の有無を判定し、ある環境だけゲートを置く。

置き場所についても 2 案を比較した。

- A 案: 既存 guideline に節を設ける（`doc/guidelines/git-operation-guidelines.md` に置き、`doc/guidelines/github-cli-guidelines.md` と `.agents/mcp/github-op-integrated/README.md` から参照する）。
- B 案: 新規 guideline を作り、既存 3 文書から参照する。

## 検討内容

事前ゲート案は、1Password 連携が無い環境でも人間の手番を作る。cloud session には `op` が無く署名も platform が行う（[0018-cloud-session-environment.md](0018-cloud-session-environment.md)）ため、そこでは純粋な待ち時間になり、`drive-issue-to-reviewed-pr` のような自律フローを止める。

環境判定案は、その副作用を避けるために skill へ `op` の有無などの条件分岐を持ち込む。これは判定のための事前チェックを誘発し、`doc/guidelines/github-mcp-guidelines.md` が禁じている preflight と同じ形になる。skill が環境差を抱える点でも保守しにくい。

事後中断案のコストは、失敗が 1 回起きることである。承認待ちの失敗は副作用を残さないか、残しても read-back で検出できる（`doc/guidelines/github-cli-guidelines.md` の「write 操作が失敗したとき」、`doc/guidelines/github-mcp-guidelines.md` の「MCP write が失敗したとき」）。1 回の失敗と引き換えに、連携が無い環境でフローが止まらない。

失敗後の危険は回避行動にある。署名の無効化、鍵や remote URL の無断変更、別の 1Password 連携経路への自動 fallback は、いずれも「とりあえず先へ進む」挙動として出やすい。特に MCP の起動失敗から `gh` への fallback は、`doc/guidelines/github-mcp-guidelines.md` の fallback 規定から導けてしまうが、`gh` も同じ承認を要求するため解決しない。よって中断手順には禁止事項を明示する必要がある。

置き場所については、対象が 5 経路あり、うち git コマンドは 3 経路、残りは `gh` と MCP server 起動である。A 案は入口の数が増えないが、`gh` と MCP の利用者が「Git 操作ルール」の一節へ誘導されることになり、文書の題名と適用範囲がずれる。B 案は入口が 4 箇所（共通正本、Cursor 入口、Claude Code 入口、`AGENTS.md`）増えるが、Codex は `AGENTS.md` の一覧からしか rule 正本へ到達しないため（`doc/guidelines/agent-configuration-management.md`）、独立した行として載せることで主題から直接到達できる。

## 決定

事後中断案（候補 2）を採用する。置き場所は B 案とし、`doc/guidelines/one-password-approval-failure.md` を新規の共通正本とする。

正本に定めた要点は次のとおり。

- 1Password 連携操作の前段にゲートを置かない。承認が通るかを確かめる preflight（`op whoami`、`ssh-add -l`、`gh auth status` など）も実行しない。
- skill に 1Password / `op` 固有の条件分岐を持たせない。失敗時は正本を参照する。
- 承認待ちに起因する失敗（タイムアウト、承認の拒否、app の lock、app の未起動）を検知したら中断し、失敗した操作、エラー出力の要旨、未反映の変更を報告して 4 つの選択肢を提示する。
- 別の 1Password 連携経路へ自動 fallback しない。`doc/guidelines/github-mcp-guidelines.md` の fallback 条件に当たらないことを明記する。
- 署名の無効化、鍵・`gpg.ssh.program`・`SSH_AUTH_SOCK`・credential helper・remote URL の無断変更、承認を迂回する設定変更を禁止する。
- 再実行前に read-back で反映を確認する。commit の署名失敗では commit が作られていないことを確認する。
- ユーザーへ直接問えない実行主体（subagent）は選択肢を選ばず、確認事項として呼び出し元へ返す。
- 1Password 起因か切り分けられない場合も、断定せず中断する。

あわせて、環境依存の前提を断定していた記述を改めた。`doc/guidelines/git-operation-guidelines.md` と入口 shim は remote を HTTPS と断定し、push / fetch を SSH agent 連携の対象外としていたが、scheme は clone 時の選択とメンバーの設定に依存するため、どちらも前提にしない形へ書き直した。

## 理由

- 連携が無い環境を含めてフローを止めないのは、事後中断案だけである。事前ゲートと環境判定はどちらも、連携が無い環境に不要なコストを課す。
- 承認待ちの失敗は検出可能で、副作用の有無は既存の read-back ルールで確かめられる。失敗 1 回のコストは、自律フローを常に止めるコストより小さい。
- 失敗後に起きやすいのは回避行動であり、そこを禁止事項として正本に固定することが実質的な安全策になる。前段のゲートはこの危険に対して何もしない。
- 新規 guideline にしたのは、適用範囲が git コマンドに収まらないためである。`gh` と MCP server 起動を含む主題を git の文書の一節に置くと、題名と範囲がずれ、Codex からは主題として見えない。

## 影響

- 新規: `doc/guidelines/one-password-approval-failure.md`、`.cursor/rules/one-password-approval-failure.mdc`、`.claude/rules/one-password-approval-failure.md`。`AGENTS.md` と `doc/guidelines/README.md` に登録する。
- `doc/guidelines/git-operation-guidelines.md`: remote の scheme 断定を撤回し、署名失敗時の「制約のない実行環境で再実行する」を新正本への参照に置き換えた。SSH agent 経路の socket path が macOS の例であることを明示した。
- `doc/guidelines/github-cli-guidelines.md`: `op plugin run` 失敗時の記述を新正本への参照に置き換え、`gh auth status` が preflight でないことを明示し、write 失敗時の扱いに承認待ちの分岐を加えた。
- `doc/guidelines/github-mcp-guidelines.md`: fallback 優先順位と write 失敗時の手順から、承認待ちに起因する失敗を除外した。
- `doc/guidelines/cloud-session-guidelines.md`: 本ルールが cloud session では発動しないことを表に加えた。
- `.agents/mcp/github-op-integrated/README.md`: トラブルシュートに `authorization timeout` の行を加えた。
- `.agents/skills/number-working-branch-note/SKILL.md`: `.op/` と `op` の有無による分岐を guideline 参照へ縮めた。
- `.agents/skills/drive-issue-to-reviewed-pr/SKILL.md`: 「停止とエスカレーション」に 1 行加えた。
- 入口 shim（`.claude/rules/git-operation-guidelines.md`、`.cursor/rules/git-operation-guidelines.mdc`）: remote の HTTPS 断定を書き直した。
- Rust のコードに変更は無い。

## 後から見直す条件

- 1Password 連携を使うメンバーがいなくなった場合。正本の適用範囲が空になるため、rule の削除を検討する。
- 承認ダイアログを出さずに署名や認証を通す構成（service account、長期 session など）を全メンバーが採る場合。
- 中断が頻発して自律フローの実効が落ちる場合。選択肢の提示方法や、ユーザー不在を前提にした待ち合わせ方を再検討する。
