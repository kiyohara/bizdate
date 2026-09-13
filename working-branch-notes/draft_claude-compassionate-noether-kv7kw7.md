# 作業ブランチメモ

- ブランチ: `claude/compassionate-noether-kv7kw7`
- PR: 未作成
- 最終更新: 2026-09-13

## 目的

Issue #53 を消化する。1Password 連携操作が承認待ちで失敗したときの中断手順を共通正本に定め、既存の skill / guideline / 入口を網羅的に見直す。

1Password 連携操作の前段にゲートを置かず、承認待ちに起因する失敗が起きた時点で中断してユーザーへ選択肢を提示する方針を正本化する。

## 現在の状況

P1（実装と PR 作成）の実装と検証を終えた。

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
- [ ] PR を作成し、note を採番する

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

### 未検証事項

- **承認待ちタイムアウトの再現**: 再現できなかった。本 session は cloud session で、sandbox に `op` も `gh` も無く、`gpg.ssh.program` は platform の signer を指すため 1Password が関与しない。新正本の「適用範囲」が cloud session を対象外としている状態そのものであり、手順どおりの中断と選択肢提示は実環境で確認が必要。確認手段は、1Password 連携のあるローカル環境で app を lock したまま `git commit` または `op plugin run -- gh` を実行すること。
- **新 rule の description による発火**: `.claude/rules/one-password-approval-failure.md` は frontmatter なしの全セッションロード、`.cursor/rules/one-password-approval-failure.mdc` は `description` による判断に委ねる構成にした。Cursor が意図した場面でこの rule を読み込むかは、その場面を Cursor で再現しない限り確認できない。確認手段は Cursor で git / GitHub 操作を含む作業を開始し、rule がロードされるかを見ること。

## リスク・ブロッカー

- 上記「未検証事項」の 2 件が残る。いずれも本 session の実行環境では再現できない。
- `.agents/skills/number-working-branch-note/SKILL.md` は Issue #52 も変更対象としていた。#52 は PR #54 として merge 済みで、本ブランチはその後の `main` から切っているため衝突は無い。

## セッションログ

- 2026-09-13: Issue #53 に着手。依存なしを確認し、open PR が無いこと（直列消化）を確認した。`op plugin` / `.op/` / `1Password` / `承認` / `合意` / `HTTPS` / `制約のない実行環境` の git grep で対象箇所を洗い出した。
- 2026-09-13: 新 guideline `doc/guidelines/one-password-approval-failure.md` を作成し、入口 2 件と `AGENTS.md` / `doc/guidelines/README.md` へ登録した。既存 guideline 4 件、MCP README、skill 2 件、入口 shim 2 件を見直した。decision log 0021 を作成し index を更新した。
- 2026-09-13: Issue の検証項目を実行した。path / link 240 件の機械検査、文体 grep、`git diff --check` は通った。承認待ちの再現と Cursor での発火は未検証事項として記録した。
