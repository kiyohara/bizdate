# 作業ブランチメモ

- ブランチ: `claude/compassionate-noether-kv7kw7`
- PR: #56
- 最終更新: 2026-09-13

## 目的

Issue #53 を消化する。1Password 連携操作が承認待ちで失敗したときの中断手順を共通正本に定め、既存の skill / guideline / 入口を網羅的に見直す。

1Password 連携操作の前段にゲートを置かず、承認待ちに起因する失敗が起きた時点で中断してユーザーへ選択肢を提示する方針を正本化する。

## 現在の状況

review cycle（2 周）の収束後、1Password 連携のあるローカル環境で追試を行い、その結果を反映した。追試で読み分けの観測点が成り立たないことが判明したため、`github-mcp-guidelines.md` の「起動失敗の読み分け」を書き直している。

## 決定事項

### 置き場所: 新規 guideline（B 案）

`doc/guidelines/one-password-approval-failure.md` を新規の共通正本とした。比較した 2 案と採否理由は `doc/design/decision-log/0021-one-password-approval-failure.md` の「検討内容」「理由」に記録した。

- A 案（既存 guideline に節を設ける）: 入口は増えないが、対象 5 経路のうち `gh` と MCP server 起動は git コマンドでないため、「Git 操作ルール」の一節に置くと題名と適用範囲がずれる。
- B 案（新規 guideline）: 入口が 4 箇所増えるが、Codex は `AGENTS.md` の一覧からしか rule 正本へ到達しないため、独立した行として載せることで主題から直接到達できる。

### 方針

- 前段にゲートも preflight も置かない。
- 承認待ちに起因する失敗（タイムアウト / 拒否 / lock / app 未起動）で中断し、失敗した操作・エラー要旨・未反映の変更を報告して 4 つの選択肢を提示する。
- 別の 1Password 連携経路へ自動 fallback しない。`github-mcp-guidelines.md` の fallback 条件に当たらないことを明記した。
- 署名の無効化、鍵・`gpg.ssh.program`・`SSH_AUTH_SOCK`・credential helper・remote URL の無断変更、承認を迂回する設定変更を禁止する。
- subagent は選択肢を選ばず、確認事項として呼び出し元へ返す。
- MCP 起動失敗の読み分けは、host の表示文字列ではなく 1Password 非依存の確認で行う（追試による補正）。これにより docker 不在や config 不在のような機能失敗では `gh` fallback が従来どおり働き、原因が見つからない場合だけ中断へ寄る。
- 1Password 起因か切り分けられない場合も断定せず中断する。

## 見直しの結果（対象ファイルごとの該当有無と処置）

| 対象 | 該当 | 処置 |
| --- | --- | --- |
| `doc/guidelines/git-operation-guidelines.md` | あり | remote を HTTPS と断定し push / fetch を SSH agent 連携の対象外としていた前提を、scheme 依存の条件分岐へ書き直した。署名失敗時の「制約のない実行環境で再実行する」を新正本への参照に置き換えた。SSH agent 経路の socket path が macOS の例であることを明示した。関連ルールに参照を追加した |
| `doc/guidelines/github-cli-guidelines.md` | あり | `op plugin run` 失敗時の「制約のない実行環境で再実行する」を新正本への参照に置き換えた。`gh auth status` が preflight でないことを明示した。write 失敗時の手順に承認待ちの分岐を加えた |
| `doc/guidelines/github-mcp-guidelines.md` | あり | fallback 優先順位 2 と「MCP write が失敗したとき」から、承認待ちに起因する失敗を除外した（`gh` も同じ承認を要求するため） |
| `doc/guidelines/cloud-session-guidelines.md` | あり | 本ルールが cloud session では発動しないことを前提の表に加えた |
| `.agents/mcp/github-op-integrated/README.md` | あり | トラブルシュートに `authorization timeout` の行を加え、`gh` へ自動 fallback しないことを明示した |
| `.agents/skills/number-working-branch-note/SKILL.md` | あり | 「GitHub 操作形式」の `.op/` と `op` の有無による分岐を guideline 参照へ縮めた。承認待ち失敗時の参照を 1 行加えた |
| `.agents/skills/drive-issue-to-reviewed-pr/SKILL.md` | あり | 「停止とエスカレーション」に 1 行追加した。手順本体は複製していない |
| `.claude/rules/git-operation-guidelines.md` | あり | remote の HTTPS 断定を書き直した |
| `.cursor/rules/git-operation-guidelines.mdc` | あり | 同上 |
| `AGENTS.md` | あり | 共通正本一覧と「AI Agent 向けルール」に新 rule を登録した |
| `doc/guidelines/README.md` | あり | 一覧に新 rule を追加した |
| `.agents/skills/run-issue-task/SKILL.md` | なし | git / GitHub 操作の前段ゲートも 1Password 固有の分岐も無い。失敗時は guideline 参照のみ |
| `.agents/skills/review-pull-request/SKILL.md` と `references/` | なし | 停止条件はモード確定・Agent 種別・対象 PR の特定に由来する。1Password 由来のゲートは無い |
| `.agents/skills/maintain-progress/SKILL.md` | なし | 確認はすべて成果物の内容判断に由来する |
| `.agents/skills/register-progress-issue/SKILL.md` | なし | 確認は入力の曖昧性に由来する |
| `.claude/rules/` / `.cursor/rules/` のその他 | なし | 要約行に 1Password 由来のゲートは無い。`github-cli-guidelines` の「承認を得てから実行する」は影響の大きさ由来でスコープ外 |
| `doc/guidelines/` のその他（`issue-driven-task-execution` / `development-loop` / `pull-request-guidelines` / `working-branch-notes-handling` / `working-branch-notes-security` / `agent-configuration-management` / `decision-log-guidelines` / `document-style-guidelines` / `development-command-guidelines`） | なし | `1Password` / `op plugin` / `.op/` / `制約のない実行環境` を grep して棚卸しした。該当は `agent-configuration-management.md` の 2 件のみで、いずれも secret reference の置き方（`.config/` の config template、`.worktreeinclude` の allowlist）であり承認ゲートではない。処置不要 |
| `.github/copilot-instructions.md` | なし | レビュー観点は「入口に恒久ルールを直接書かない」「リンク切れ」「basename 揃いと `AGENTS.md` リンク」で本 PR の変更をすでにカバーする。1Password 固有の観点を足すと指摘ノイズになるため変更しない |
| `CLAUDE.md` | なし | `@AGENTS.md` の取り込み shim のまま。追記不要 |

### ゲートの区別の結論

1Password の承認ダイアログ回避を理由とするゲートは、本リポジトリには残っていなかった。同種のゲートは PR #44 / #46 で `number-working-branch-note` に置かれていたが、Issue #52（decision log 0020）で撤回済みである。

`number-working-branch-note` に残る合意に関する記述は、完了タスク行の書き換えという成果物の内容判断に由来するもので、Issue #53 のスコープ外である（0020 が扱う）。`github-cli-guidelines.md` の「ユーザーの確認を得てから実行する操作」、PR の merge と inline thread の resolve を agent が行わない規定は、影響の大きさと取り消しにくさに由来するためスコープ外とし、新正本にもその旨を明記した。

## 次にやること

- [x] 新 guideline の正本を作る
- [x] 既存 guideline / skill / 入口を見直す
- [x] decision log を記録し、入口を登録する
- [x] Issue の検証項目を実行する
- [x] PR を作成し、note を採番する
- [x] review cycle を収束させる（2 周）
- [ ] 人間が inline thread 5 本を resolve する（agent は行わない）
- [ ] 人間が merge を判断する（agent は行わない）

## 検証

| 項目 | 結果 |
| --- | --- |
| `git grep` で 1Password 固有の分岐が skill に残っていないこと | `op plugin` と `.op/` は `.agents/skills` 配下に 0 件。残る `1Password` の言及は `drive-issue-to-reviewed-pr` と `number-working-branch-note` の guideline 参照 2 行のみ |
| `git grep` で環境を断定する記述が残っていないこと | `HTTPS` の残存は (1) 環境依存と明記した git guideline / 入口 shim、(2) decision log の経緯記述、(3) `$HTTPS_PROXY` 環境変数、(4) cloud session の platform 仕様（HTTPS 固定は platform の事実）のみ。断定は残っていない |
| `制約のない実行環境` の残存 | guideline からは消え、decision log の経緯記述 2 箇所のみ |
| `承認` / `合意` の棚卸し | 上記「見直しの結果」と「ゲートの区別の結論」のとおり。1Password 由来のゲートは 0 件 |
| 入口 shim の要約行が正本と食い違わない | `git-operation-guidelines` の remote 行を正本と同じ「scheme は環境依存」へ揃えた。新 rule の入口 2 件は正本の要点 3 行のみで、手順本体を複製していない |
| 新 rule の basename 3 箇所揃い | `doc/guidelines/` / `.cursor/rules/`（`.mdc`）/ `.claude/rules/` の 3 箇所で `one-password-approval-failure` に揃っている |
| `AGENTS.md` と `doc/guidelines/README.md` への登録 | 共通正本一覧、「AI Agent 向けルール」、README 一覧の 3 箇所に追加済み |
| repo 相対 path と markdown link の切れ | 変更ファイル内の 240 件を機械検査。切れとして出たのは gitignored な local config（`.config/github-op-integrated.conf`、`.claude/settings.local.json`）と将来用 `doc/help/` のみで、いずれも既存行であり本 PR の追加行には含まれない |
| 文末の常体統一 | 変更ファイルをですます調の語尾で grep。検出は `AGENTS.md` の既存行 1 件（文体ルール自体を説明する「利用者向けはですます調」という引用）のみ |
| `git diff --check` | 通った（空白エラーなし） |
| `cargo` 再検証 | 省略。Rust コードと `Cargo.toml` / `Cargo.lock` に変更が無いことを `git diff --cached --name-only` で確認した（Issue の指示どおり） |
| review 指摘の現物確認（P3） | 5 件すべて確認済み。`[must]` は wrapper の診断メッセージ（`op` / `docker` / config の 3 種）と MCP host の表示を突き合わせ、原因不明の起動失敗が実在することを確認。`[imo]` は参照元 2 箇所が「プロンプト未到達」を新正本へ送っていることを確認。`[nits]` 2 件は該当行と未変更 guideline の grep（該当は `agent-configuration-management.md` の secret reference 2 件のみ）で確認 |
| P4 修正後の `git diff --check` | 通った |
| ローカル追試（1Password 実環境） | 実施。macOS 26.6 / 1Password 8.12.36 / `op` 2.34.1 の環境で、Q1 〜 Q8 と Q10 を実施した。結果は下記「ローカル追試の結果」 |

### ローカル追試の結果

cloud session では `op` が無く再現できなかったため、1Password 連携のあるローカル環境で追試した（実施: 2026-09-13、macOS 26.6.2 / 1Password 8.12.36 / `op` 2.34.1 / Claude Code 2.1.175 と 2.1.270）。追試は push せず read 中心で行い、commit の検証は scratch branch 上の `--allow-empty` に限定している。

前提どおりだった点:

| 確認項目 | 結果 |
| --- | --- |
| commit 署名の失敗で commit が作られないか | 作られない。lock / 拒否 / タイムアウト / app 未起動の全 4 ケースで HEAD が動かず `git status --short` も空。正本が示す確認コマンドで判別できる |
| lock 中に `op plugin run -- gh` は成功するか | 成功しない。直前に承認済みでも再度承認を要求して失敗する（session cache では通らない）。fallback 禁止の前提が成り立つ |
| SSH agent 経路で承認ダイアログは出るか | 出る（`ssh -T` による代理確認。remote は HTTPS のため本経路は非該当） |
| macOS の socket path の例示 | 1Password 8.12.36 で実在する |
| `ssh-add -l` と署名の可否は無関係か | 無関係。`op-ssh-sign` 経路では鍵 0 件でも署名は成功する |
| `.claude/rules/` の rule は全セッションにロードされるか | される（frontmatter 無し）。ただしブランチ切り替え後に開始した session でないと効かない |

前提が崩れた点:

| 確認項目 | 結果 |
| --- | --- |
| MCP host は wrapper の診断を agent に渡すか | **渡さない。** 診断は host の debug log にしか残らず、host の画面（起動時警告 / MCP 一覧 / doctor）にも tool 検索の結果にも出ない。当初の読み分け基準の第 1 bullet は観測点として存在しなかった |
| 承認待ちは `authorization timeout` として見えるか | **見えない。** host の接続 timeout（30 秒）が `op` の承認 timeout（約 60 秒）より短く、agent に届くのは `CONNECT_TIMEOUT` / `connection timed out after 30000ms`。当初例示した `CONNECTION_CLOSED` も実際とずれていた |
| 承認待ちの最中はどう見えるか | 失敗ではなく「接続中（再検索を促す）」として見える。規定が無かった |
| 即時の切断は非 1Password の証拠になるか | ならない。app 未起動や承認の拒否でも `op` は 0〜3 秒で失敗する |
| host が諦めた後のダイアログ承認で復旧するか | 復旧しない（ダイアログは host の timeout 後も残る）。host 側の再接続が必要 |
| SSH agent 経路の出力から 1Password 起因と判定できるか | できない。`agent refused operation` / `Permission denied (publickey)` は鍵未登録の場合と区別できない |
| app 未起動時の `op` のエラー文言 | app の更新を促す。agent が更新へ進む余地があった |

反映内容は「セッションログ」と `doc/design/decision-log/0021-one-password-approval-failure.md` の「追試による補正」に記録した。

### 未検証事項

- **Cursor での `description` 発火**: 未実施。追試環境では Cursor を常用していないため確認できていない。`.claude/rules/` 側は frontmatter 無しの全セッションロードで確認済みであり、発火条件に依存しない。確認手段は Cursor で git / GitHub 操作を含む作業を開始し、rule がロードされるかを見ること。
- **承認プロンプトが実行環境へ届かないケース**: 追試環境では再現しなかった（agent から実行したコマンドでもダイアログはユーザーの画面に届いた）。sandbox / TTY 制約のある構成でのみ起きる。
- **host の接続 timeout 後にダイアログを承認した場合の実測**: 復旧しないという判断は、host が先に失敗を確定させる事実からの推論であり、承認して回復しないことの実測ではない。
- **staged 変更がある状態での署名失敗時の index 保持**: 追試は `--allow-empty` に限ったため未観測。
- **Codex を host にした場合の MCP 起動失敗の見え方**: 未実施。

## リスク・ブロッカー

- 上記「未検証事項」の 5 件が残る。追試で解消した 2 件（承認待ちの再現、`.claude/rules/` のロード）は除いた後の数である。残る 5 件は、cloud session でも追試環境でも再現できなかったものと、推論に留まるものである。
- review cycle は 2 周で収束した。指摘 5 件は全件採用・修正済みで、2 周目の P5 も 5 件すべて resolve 可と判定した。未収束の指摘は無い。
- 追試の反映で `github-mcp-guidelines.md` の「起動失敗の読み分け」を書き直したため、`[must]` thread の対象本文は P5 時点から再び変わっている。resolve 可否の再判定が必要である。
- 読み分けの新しい基準は、MCP host の現在の仕様（wrapper の stderr を agent へ渡さない、接続 timeout 30 秒）に依存する。host 側の仕様変更で前提が変わるため、decision log 0021 の「後から見直す条件」に加えた。
- 2 周目の P5 で挙がった `[fyi]` 1 件（`mcp-github-op-integrated.sh` の 48 行目だけが `mcp-github-op-integrated:` prefix を持たない）は、本 PR では対処しないと判断した。48 行目は 47 行目の config file 診断の 2 行目として同じ `if` ブロックで直後に出力され、単独では現れない。guideline の本文は「診断の先頭が prefix」と書いており行単位の網羅を主張していない。log が末尾 1 行に切れた場合は prefix 無しと見え「原因不明 → 中断」へ倒れるため安全側である。対処は wrapper script の変更になり、本 Issue のスコープ外。
- `.agents/skills/number-working-branch-note/SKILL.md` は Issue #52 も変更対象としていた。#52 は PR #54 として merge 済みで、本ブランチはその後の `main` から切っているため衝突は無い。

## セッションログ

- 2026-09-13: Issue #53 に着手。依存なしを確認し、open PR が無いこと（直列消化）を確認した。`op plugin` / `.op/` / `1Password` / `承認` / `合意` / `HTTPS` / `制約のない実行環境` の git grep で対象箇所を洗い出した。
- 2026-09-13: 新 guideline `doc/guidelines/one-password-approval-failure.md` を作成し、入口 2 件と `AGENTS.md` / `doc/guidelines/README.md` へ登録した。既存 guideline 4 件、MCP README、skill 2 件、入口 shim 2 件を見直した。decision log 0021 を作成し index を更新した。
- 2026-09-13: Issue の検証項目を実行した。path / link 240 件の機械検査、文体 grep、`git diff --check` は通った。承認待ちの再現と Cursor での発火は未検証事項として記録した。
- 2026-09-13: P2（review）を fresh context の subagent へ委譲した。review cycle `claude-code-6f0df98-20260913031849`、head `6f0df98`、指摘 5 件（`[must]` 1 / `[ask]` 1 / `[imo]` 1 / `[nits]` 2）。
- 2026-09-13: P3 で 5 件すべてを現物確認した。wrapper の診断メッセージ、参照元 2 箇所の文言、cloud session 節、未変更 guideline の grep を確認し、全件を妥当と判断して採用した。
- 2026-09-13: P4 で 5 件へ対応した。MCP 起動失敗の読み分け基準を `github-mcp-guidelines.md` に新設し、新正本の適用範囲へプロンプト未到達を追加、`.op/` 確認が preflight でないことを明示、cloud session 節の文言を言い切り、見直し表へ未変更 guideline の行を追加した。
- 2026-09-13: P5（再確認）を同じ subagent が実行した。head `69e1c70`、5 thread すべて resolve 可、未対応 0 件。新規のブロッキング指摘は無く、非ブロッキングの `[fyi]` 2 件のみ。
- 2026-09-13: P6 で `[fyi]` 2 件を採用し、2 周目の P4 として反映した。wrapper の診断は 5 件すべて `mcp-github-op-integrated:` を prefix に持つことを script で確認し、3 件の列挙を prefix による網羅的な判定へ置き換えた。あわせて優先順位 2 から新設節への前方参照を足した。
- 2026-09-13: 2 周目の P5 が完了した。head `d7d7303`、5 thread すべて resolve 可、未収束 0 件。review cycle は 2 周で収束した（上限 2 周）。
- 2026-09-13: 1Password 連携のあるローカル環境で追試を実施した（別 session。push 無し、read 中心）。commit 署名と `op plugin` の前提は確認できたが、MCP 起動失敗の読み分けの観測点が成り立たないことが判明した。
- 2026-09-13: 追試結果を反映した。`github-mcp-guidelines.md` の「起動失敗の読み分け」を、host に出ない wrapper 診断ではなく 1Password 非依存の確認（`docker info`、config の有無、PATH、実行権限）で切り分ける形へ書き直した。新正本へ経路別の実測文言、MCP の再接続と 30 秒制約、別経路の PAT scope、app 更新の禁止を追記し、入口 shim へ MCP 起動失敗を足した。decision log 0021 に「追試による補正」を記録した。
- 2026-09-13: 追試結果の反映にあたり、新しい review cycle `claude-code-0273fda-20260913064236` を回した（指摘 5 件: `[must]` 2 / `[imo]` 2 / `[nits]` 1）。既存 5 thread は新 head でも resolve 可と再判定された。
- 2026-09-13: 2 cycle 目の指摘 5 件へ対応した。読み分けの確認項目を 6 項目へ広げ、「いずれも正常なら承認待ち」という断定をやめた（wrapper は `exec op run -- docker run` まで進むため、image や secret reference の解決失敗が一覧の外に残る）。SSH 経路の調査禁止が `git-operation-guidelines.md` の署名経路確定手順と衝突していたため目的で切り分け、`SSH_AUTH_SOCK` の禁止範囲を永続的な付け替えに限定した。`command -v op` と `op` の実行の区別、接続中の再確認の回数、decision log の 影響 節の取りこぼしも直した。
