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

## 追試による補正（2026-09-13）

cloud session には `op` が無く、承認待ちの失敗を再現できなかった。1Password 連携のあるローカル環境（macOS 26.6、1Password 8.12.36、`op` 2.34.1）で追試し、次を確認した。

前提どおりだった点:

- commit の署名が承認待ちで失敗しても commit は作られない。`git log -1` と `git status --short` で判別できる。
- app を lock した後の `op plugin run -- gh` は、直前に承認済みでも再度承認を要求して失敗する。session cache では通らない。「別経路も同じ承認を要求する」という fallback 禁止の前提は成り立つ。
- SSH agent 経路でも承認ダイアログは出る。macOS の socket path の例示も実在する。
- frontmatter 無しの `.claude/rules/one-password-approval-failure.md` は新規 session に全文ロードされる。

前提が崩れた点と補正:

- **MCP host は wrapper の診断を agent に渡さない。** 診断は host の debug log にしか残らず、host の画面にも tool 検索の結果にも出ない。当初の読み分け基準は「wrapper 自身の診断が見えるか」を観測点にしていたが、その観測点は agent から存在しない。`docker info` や config file の有無など **1Password に触れない確認**で切り分ける形へ改めた。これにより、docker 不在や config 不在のような機能失敗では `gh` fallback が従来どおり働く。
- **承認待ちは `authorization timeout` として見えない。** host の接続 timeout（30 秒）が `op` の承認 timeout（約 60 秒）より短いため、agent に届くのは `CONNECT_TIMEOUT` / `connection timed out after 30000ms` である。当初例示した `CONNECTION_CLOSED` も実際とずれていた。
- **承認待ちの最中は失敗ではなく「接続中」として見える。** tool の応答は再検索を促す。待ち続けないよう、1 回だけ再確認してから切り分けへ進む規定を足した。
- **即時の切断も非 1Password の証拠にならない。** app 未起動や承認の拒否でも `op` は 0〜3 秒で失敗するため、所要時間では読み分けられない。
- **host が諦めた後もダイアログは残る。** そのダイアログを承認しても接続は回復せず、host 側の再接続が必要である。選択肢の補足に加えた。
- **SSH agent 経路の出力に 1Password の語が無い。** `agent refused operation` / `Permission denied (publickey)` は鍵未登録の場合と区別できない。「判断できない場合」へ落ちることを明記した。
- **app 未起動時の `op` のエラーは app の更新を促す。** agent が更新へ進まないよう、禁止する回避策に app の更新・再インストール・再起動を加えた。
- 「切り分けのための調査コマンドを重ねない」と、上記の 1Password 非依存の確認が衝突しないよう、例外として許す範囲を明示した。

補正後も決定そのもの（事後中断案、新規 guideline という配置）は変えていない。変えたのは読み分けの観測点と、実測の文言である。

## 後から見直す条件

- 1Password 連携を使うメンバーがいなくなった場合。正本の適用範囲が空になるため、rule の削除を検討する。
- 承認ダイアログを出さずに署名や認証を通す構成（service account、長期 session など）を全メンバーが採る場合。
- 中断が頻発して自律フローの実効が落ちる場合。選択肢の提示方法や、ユーザー不在を前提にした待ち合わせ方を再検討する。
- MCP host の仕様が変わった場合。wrapper の stderr を agent へ渡すようになる、接続 timeout が `op` の承認 timeout より長くなる、timeout 値が設定可能になるなどの変更は、「起動失敗の読み分け」の前提を変える。
