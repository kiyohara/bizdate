# 作業ブランチメモ

- ブランチ: `claude/drive-issue-reviewed-pr-zcvhfd`（cloud session が用意したブランチ）
- PR: #54
- 最終更新: 2026-09-13

## 目的

Issue #52 に従い、`number-working-branch-note` の完了タスク行に対するユーザー合意ゲートを、現状の運用に合わせて見直す。

ゲートは PR #44 で導入され PR #46 で維持されたが、PR #51（`drive-issue-to-reviewed-pr` の追加と自己適用）で実際に当たり、採番という機械的な経路の途中に人間の手番が 1 回入ることが分かった。採番は PR 作成直後に必ず通るため、ゲートが残る限り orchestrated flow は完全自律にならない。

本 Issue は `drive-issue-to-reviewed-pr` のフローで処理する。ゲートを廃する案を採るため、本 PR 自身の採番がその検証になる。

## 現在の状況

案 B（合意ゲートを廃し、判定できる行は書き換え、判定できない行は触らず報告する）を採用し、`.agents/skills/number-working-branch-note/SKILL.md` の 4 箇所と「stale 表現の定型置換」の前置きを揃えた。decision log 0020 を追加し、index へ 1 行足した。

`drive-issue-to-reviewed-pr` 側にゲートの記述は元から無く、案 B では追記も不要である。

PR #54 を作成し、変更後の手順で採番した。採番は合意待ちの停止なしに完了し、Issue #52 の主要な検証項目（人間の手番が入らないこと）を満たした。review 委譲前の head は `44fbd96`。

## 決定事項

- 案 B を採用する。合意ゲートを廃し、機械判定できる完了タスク行は合意を待たず書き換え、複合行と判断不能な行は触らず終了時に報告する。
- 案 A は採らない。A は「合意を要求する範囲を複合行と判断不能な行だけに絞る」案だが、その 2 種は PR #46 の時点で既に「触らず報告する」と定めた行である。A を採ると、書き換えないと決めた行に合意経路を新設することになり、Step 5 の既存規定と衝突する。合意を残す実質的な対象が無い。
- 案 C は採らない。ゲートの由来（判断を要する行は機械置換の外側へ分ける）は PR #46 の判定基準の明文化で解消済みであり、残す根拠が無い。orchestrator 側へゲートを転記しても、頻度の高い人間の手番はそのまま残る。
- 書き換え後の可視性は「終了時の報告」で担保する。合意を廃すると、ユーザーが変更を知る経路が報告だけになるため、書き換えた行の一覧を報告項目へ追加した。
- 判定基準そのもの（「本 skill の終了時点で未完のまま残るか」、複合行と判断不能行の除外）は PR #46 のまま変えない。本 Issue が変えるのは、判定できた行に合意が要るかどうかだけである。
- 他 skill に同種の合意ゲートは無い。`maintain-progress` の「判断に迷う箇所は止めてユーザーに確認する」は判断不能な場面の安全弁であり、機械判定できる出力にまで合意を要求するものではない。`review-pull-request` / `run-issue-task` / `register-progress-issue` / `drive-issue-to-reviewed-pr` のものは入力の曖昧性・前提条件の不成立に対する停止であって、成果物の内容判断を理由とするゲートではない。別 Issue には切らない。
- decision log は新規ログとして記録する。PR #44 / #46 は「skill 手順の調整」として log を作らない判断だったため追記先が無く、今回は以前の方針の撤回にあたる（`doc/guidelines/decision-log-guidelines.md` の「記録が必要な場面」）。
- `doc/guidelines/` は変更しない。ゲートは skill の手順にのみ存在し、guideline 側には記述が無い。

## 次にやること

- PR を作成し、note を採番する。（完了）
- 採番で人間の手番が入らないことを確認する（Issue #52 の検証項目）。（完了）
- SKILL.md の自己完結性を含めて review を subagent へ委譲する。
- 指摘へ対応し、各 thread へ処置を返信する。
- 対応結果の再確認を subagent へ委譲し、収束を確認する。
- thread の resolve と PR の merge（ユーザー）。

## 検証

| 項目 | 結果 |
|---|---|
| 4 箇所の整合（Step 5 / Step 10 / やらないこと / 終了時の報告） | 通る。合意を扱う記述は 4 箇所のみで、いずれも「機械判定できる行は合意を待たない」「複合行と判断不能な行は触らず報告する」で一致する。「手順」の前置きにあった「判断に迷うときは進めずに確認する」が完了タスク行にも掛かって読める点を、Step 5 への例外として明記した |
| 本 PR の採番が人間の手番なしで完了する | 通る。rename、`- PR:` 欄の `（採番前）` → `#54`、完了タスク行 1 行への `（完了）` 付記、commit（`44fbd96`）、push、description の path 置換まで、合意待ちの停止なしに完了した。判定できない 1 行（「採番で人間の手番が入らないことを確認する」）は規定どおり触らず報告に回した |
| `number-working-branch-note` と `drive-issue-to-reviewed-pr` の食い違い | 無し。orchestrator 側の採番言及は P1 の完了条件 1 箇所のみで、ゲートに触れていない。案 B のため追記も不要 |
| repo 相対 path と markdown link | 切れなし。変更行の backtick path は全件実在。既存の `.op/`（1Password 連携の前提 path）と `draft_...md`（汎用 placeholder）は実ファイルを指さない既存表記であり、本 PR の変更対象外 |
| 文体（常体） | 追加・変更文書で混在なし |
| `git diff --check` | 問題なし |
| 情報統制チェック | 該当なし（`password` / `secret` / `token` / `cookie` / `session id` / `PRIVATE KEY` に続く実値なし） |

Rust のコードに変更が無いため、`cargo` による再検証は省略する。

## リスク・ブロッカー

- 合意を廃すると、誤検出の書き換えがユーザーの却下を経ずに commit へ入る。書き換え形式が `- [x]` / 行末 `（完了）` の付記に限られ、行を削除せず、note の整合性スコープも「当ブランチの作業判断・経緯を読める範囲で十分」であるため、可逆かつ低リスクと判断した。誤検出は終了時の報告で可視化する。
- `github-op-integrated` MCP は cloud session で起動に失敗する。想定どおりであり、組み込み GitHub tool で進める（`doc/guidelines/cloud-session-guidelines.md`、`doc/guidelines/github-mcp-guidelines.md` の「cloud session」が skill 内の `github-op-integrated` の記載を組み込み tool へ読み替えると定めている）。

## セッションログ

- 2026-09-13: Issue #52 の依存（PR #51 の merge）を確認し、案 A / B / C を比較して B を採用した。SKILL.md の 4 箇所と定型置換の前置きを揃え、他 skill の合意ゲートを確認し、decision log 0020 と index を更新した。
- 2026-09-13: PR #54 を作成し、変更後の手順で note を採番した（`44fbd96`）。合意待ちの停止は発生せず、書き換えた行 1 件・触らなかった行 1 件を報告に回した。
