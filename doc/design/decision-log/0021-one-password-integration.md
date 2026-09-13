<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 1Password 連携操作の失敗時の扱い

- 状態: decided
- 作成日: 2026-09-13
- 最終更新日: 2026-09-13
- 関連: [0011-ai-agent-lineup.md](0011-ai-agent-lineup.md), [0018-cloud-session-environment.md](0018-cloud-session-environment.md), [0020-note-numbering-consent-gate.md](0020-note-numbering-consent-gate.md), `doc/guidelines/one-password-integration-guidelines.md`, Issue #53

## 背景

git / GitHub 操作の複数の経路で 1Password の承認ダイアログが出る。commit 署名（`op-ssh-sign`、または `ssh-keygen -Y sign` と 1Password の SSH agent）、SSH remote への接続、`op plugin run -- gh ...`、GitHub MCP server の wrapper が起動時に実行する `op run` がそれに当たる。どの経路を使うかはメンバーの環境に依存する。

ユーザーがマシンの前にいないとダイアログに応答できず、操作はタイムアウトで失敗する。別の参考プロジェクトでは、これを避けるために 1Password 連携を伴う操作の前段にユーザー合意ゲートを置いてきた。

従来は全メンバーが macOS ローカルで 1Password 連携を持っていたが、cloud session（[0018](0018-cloud-session-environment.md)）の導入で 1Password が無い環境が加わった。事前ゲートは 1Password が無い環境でも人間の手番を作り、`drive-issue-to-reviewed-pr`（[0019](0019-issue-review-cycle-orchestration.md)）のような自律フローを止める。一方で、既存 guideline の失敗時の記述は「制約のない実行環境で再実行する」に留まり、ユーザー不在でタイムアウトした場合を定めていなかった。

本ログは 1Password の承認ダイアログに由来する扱いだけを対象とする。成果物の内容判断を理由とするゲートは [0020](0020-note-numbering-consent-gate.md) が、影響の大きさや取り消しにくさを理由とする承認は `doc/guidelines/github-cli-guidelines.md` が扱う。

## 候補

方針について:

1. 事前ゲート案: 1Password 連携を伴う操作の前にユーザー合意を取る。
2. 事後中断案: 操作は通常どおり実行し、承認待ちで失敗した時点で中断してユーザーに選択肢を提示する。
3. 環境判定案: `op` の有無などで 1Password 連携の有無を判定し、ある環境だけゲートを置く。

置き場所について:

- A 案: 既存 guideline（git 操作ルール）に節を設け、`gh` と MCP の guideline から参照する。
- B 案: 1Password 連携ルールを新規 guideline として 1 本作り、git / gh / MCP の各 guideline には参照だけを置く。

MCP server の起動失敗について:

- X 案: 起動失敗の原因（1Password の承認待ちか、docker や config の不備か）を agent が切り分け、後者なら `gh` へ fallback する。
- Y 案: 起動失敗は原因を切り分けず、`gh` へも進まず中断して報告する。

## 検討内容

- 事前ゲート案と環境判定案は、1Password が無い環境に不要なコストを課す。環境判定案は skill へ `op` の有無による分岐を持ち込み、判定のための事前チェックを誘発する。事後中断案のコストは失敗が 1 回起きることだが、承認待ちの失敗は副作用を残さないか、残しても read-back で検出できる。
- 失敗後に危険なのは回避行動である。署名の無効化、鍵や remote URL の無断変更、別の 1Password 連携経路への自動切り替えは「とりあえず先へ進む」挙動として出やすい。中断手順には禁止事項を明示する。
- 置き場所は、1Password 連携ルールを 1 本にまとめる B 案を採る。あらゆる操作の場面で「こうだったらこう」と細かな分岐を定義するのは過剰であり、git / gh / MCP の各 guideline に処理フローを書くとルールが肥大化する（`doc/guidelines/agent-configuration-management.md` の「ルールをシンプルに保つ」）。導線は `AGENTS.md` の 1 行（エラーが出たら承認が間に合わなかった可能性を疑い新正本を読む）で足りる。
- MCP server の起動失敗は、agent には host の汎用表示（`CONNECT_TIMEOUT` や `Connection closed`）しか見えず、wrapper や `op` の診断は届かない（PR #56 の追試で実測）。X 案は agent から観測できない情報に依存し、判定表と例外規定が増える。Y 案は失敗 1 回の後に人間が原因を見る形で、ルールが 1 行で済む。cloud session では組み込み tool を使うため、Y 案の中断は発動しない。
- `gh` の実行形式（`.op/` と `op` があれば `op plugin run -- gh`）は、承認可否の preflight ではなく、リポジトリにスコープを限定した PAT を使うための実行形式の選択である。これは `doc/guidelines/github-cli-guidelines.md` に残し、skill 側の複製は guideline 参照へ縮める。
- git 操作ルールにあった署名経路の切り分け（`op-ssh-sign` か `ssh-keygen` + SSH agent か）は、1Password の有無で分岐する記述であるため新正本へ移す。git 操作ルールには失敗時の参照先だけを残す。
- remote を HTTPS と断定していた記述は、scheme と認証経路がメンバーの環境に依存するため撤回する。

## 決定

事後中断案（候補 2）、置き場所は B 案、MCP 起動失敗は Y 案を採用する。新規の共通正本を `doc/guidelines/one-password-integration-guidelines.md` とする。

正本に定めた要点は次のとおり。

- 1Password 連携の有無を事前に判定せず、前段にゲートや preflight を置かない。skill や guideline に `op` の有無による分岐を持ち込まない。
- 1Password や承認に関する文言を伴う失敗、または原因を判断できない失敗を検知したら中断し、失敗した操作、エラー出力の要旨、未反映の変更を報告して 4 つの選択肢（応答できる状態にして再実行 / ユーザーが手動実行 / 明示指示による別経路 / 保留）を提示する。
- 別の 1Password 連携経路への自動切り替え、署名の無効化、鍵・`gpg.ssh.program`・`SSH_AUTH_SOCK`・credential helper・remote URL・連携設定の変更、1Password app の操作を禁止する。
- 再実行前に read-back と commit 未作成の確認を行う。subagent は選択肢を選ばず呼び出し元へ返す。
- 1Password が無い環境では本ルールは発動せず、追加手順も要らない。

## 理由

- 1Password が無い環境を含めてフローを止めないのは事後中断案だけである。
- 失敗後に起きやすいのは回避行動であり、そこを禁止事項として正本に固定することが実質的な安全策になる。前段のゲートはこの危険に対して何もしない。
- ルールを 1 本にまとめ、各 guideline には参照だけを置くことで、1Password 連携がある環境も無い環境も同じ手順で扱える。
- MCP 起動失敗の切り分けを agent に求めないことで、観測できない情報に依存する判定表を持たずに済む。失敗 1 回のコストは、判定ルールを保守するコストより小さい。

## 影響

- 新規: `doc/guidelines/one-password-integration-guidelines.md`、`.claude/rules/one-password-integration-guidelines.md`、`.cursor/rules/one-password-integration-guidelines.mdc`。`AGENTS.md` と `doc/guidelines/README.md` に登録した。
- `doc/guidelines/git-operation-guidelines.md`: remote の HTTPS 断定を撤回し、署名経路の切り分け節を新正本へ移した。失敗時の「制約のない実行環境で再実行する」を新正本への参照に置き換えた。
- `doc/guidelines/github-cli-guidelines.md`: `.op/` の確認が preflight ではないことを明記し、失敗時の記述を新正本への参照に置き換えた。
- `doc/guidelines/github-mcp-guidelines.md`: fallback の優先順位から MCP の起動失敗を外し、起動失敗は中断とする 1 項を加えた。
- `.agents/skills/number-working-branch-note/SKILL.md`: `.op/` と `op` の有無による分岐を guideline 参照へ縮めた。署名失敗時の参照先を新正本へ向けた。
- `.agents/skills/drive-issue-to-reviewed-pr/SKILL.md`: 「停止とエスカレーション」に 1 行加えた。
- `.agents/mcp/github-op-integrated/README.md`: トラブルシュートに承認待ち失敗の行を加えた。
- `.claude/rules/git-operation-guidelines.md` / `.cursor/rules/git-operation-guidelines.mdc`: 署名切り分けと HTTPS 断定の行を書き換えた。
- Rust のコードに変更は無い。

## 後から見直す条件

- 1Password 連携を使うメンバーがいなくなった場合。正本の適用範囲が空になるため、rule の削除を検討する。
- 承認ダイアログを出さずに署名や認証を通す構成（service account など）を全メンバーが採る場合。
- 中断が頻発して自律フローの実効が落ちる場合。MCP 起動失敗の切り分け（X 案）を再検討する。
- MCP host が wrapper の診断を agent へ渡すようになった場合。X 案の前提が変わる。
