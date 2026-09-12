---
name: drive-issue-to-reviewed-pr
description: bizdate の GitHub Issue を 1 件受け取り、実装と PR 作成から review、review comment 対応、対応結果の再確認までを 1 つのフローとして回し、レビュー済み PR に到達させる。`run-issue-task` と `review-pull-request` を呼び出す orchestrator であり、review と再確認は context を分離するため subagent へ委譲する。Issue に着手して review cycle の完了まで一気に進めたいとき、または既存 PR を指定して review 以降だけを回したいときに使う。
argument-hint: "<Issue番号 | Issue URL> | --from-pr <PR番号>"
---

# drive-issue-to-reviewed-pr

bizdate の Issue 1 件を、実装から「レビュー済み PR」まで 1 つのフローで運ぶ orchestrator skill。

本 skill は独自のレビュー手法を持たない。既存 skill の呼び出し順、委譲境界、判断基準、停止条件を固定することだけを責務とする。

- 実装と PR 作成の正本: `.agents/skills/run-issue-task/SKILL.md`、`doc/guidelines/issue-driven-task-execution.md`
- review / 対応 / 再確認の正本: `.agents/skills/review-pull-request/SKILL.md` とその `references/`
- 本 skill の記述が上記と食い違う場合は、被委譲 skill と guideline を優先する。

## なぜ review を分離するか

実装 context を持つ agent は、自分の変更を妥当と見做しやすい。そのため review と再確認は fresh context の subagent へ委譲し、実装担当（orchestrator 自身）と分離する。これは処理の効率化ではなく、指摘の質を保つための設計前提である。

同じ理由から、orchestrator は subagent の出力をそのまま採用しない。採否は orchestrator が対象 Issue、project の正本、実装に照らして判断する。

## 入力と入口

| 入力 | 開始フェーズ |
| --- | --- |
| Issue 番号 / Issue URL | P1（実装と PR 作成） |
| `--from-pr <PR 番号>` | P2（review）。PR が既にあり、review 以降だけを回す場合 |

次の場合は開始せず、ユーザーに確認する。

- 入力が Issue か PR か曖昧である。
- 別リポジトリの URL である。
- 複数 Issue を同時に指定されている。
- `--from-pr` の対象 PR が closed または merged である。

1 Issue = 1 ブランチ = 1 PR とする。複数 Issue を同時に回さない。

## 起動前提

P1 から始める場合、着手前に次を確認する。Issue 依存の確認手順は `run-issue-task` に含まれるため、本 skill で重複して実施しない。

- Issue の依存がすべて done であること。未完了なら開始せず、未完了依存を報告して終了する。
- 直列消化であること。レビュー待ちの自分の PR が他に無いことを確認し、あればユーザーに確認する。
- cloud session（Claude Code on the web）では、ブランチは session が決めたものを使い、GitHub 操作は組み込み tool を第一選択とする（`doc/guidelines/cloud-session-guidelines.md`）。

## フェーズ

| # | フェーズ | 担当 | 委譲先 | 完了条件 |
| --- | --- | --- | --- | --- |
| P1 | 実装と PR 作成 | orchestrator | `run-issue-task` | PR が open になり、note 採番、検証記録、`progress.md` 更新（該当時）まで済んだ時点 |
| P2 | review | subagent | `review-pull-request` の `review` | 完了要約 1 本（指摘があれば inline comment も）を投稿し、read-back で確認した時点 |
| P3 | review 結果の判断 | orchestrator | — | 各指摘の処置を決めた時点 |
| P4 | 対応 | orchestrator | `review-pull-request` の `address-comments` | 確認した各 inline comment へ処置を返信し、採用分を push した時点 |
| P5 | 再確認 | subagent | `review-pull-request` の `verify-comments` | 全対象 thread への返信と完了要約 1 本を投稿した時点 |
| P6 | 再確認結果の判断 | orchestrator | — | 追加対応の要否を決めた時点 |

- P3 で処置すべき指摘が 1 件も無ければ、P4 以降へ進まずフローを終了する。
- P6 で追加対応が必要なら P4 へ戻る。上限は「反復上限」に従う。
- P4 と P5 で担当を分けるのは、`review-pull-request` が前提とする役割分担（修正担当 Agent は自分で resolve せず、元の Review 担当 Agent が再確認する）を満たすためである。

## subagent への委譲

### 渡すもの

委譲 brief に次をすべて含める。省略した項目は subagent が推測するため、空欄を残さない。

| 項目 | 内容 |
| --- | --- |
| リポジトリ | `kiyohara/bizdate` |
| 対象 PR | PR 番号 |
| mode | `review` または `verify-comments` |
| 対象 head SHA | 委譲時点の PR head SHA（full） |
| review cycle ID | P2 では「新規に作る」と指示する。P5 では P2 が返した ID をそのまま渡す |
| 読むべき path | `.agents/skills/review-pull-request/SKILL.md` と該当モードの reference の repo 相対 path |
| 追加の review 観点 | 対象 Issue 番号と、その回で特に確認してほしい点 |

**skill は名前ではなく path で渡す。** subagent の skill 一覧に載っている保証は無く、同じ session で追加・変更した skill は特に載らない。brief には「この path を読んでから始めること」と明示する。

brief の形:

```text
bizdate リポジトリ（kiyohara/bizdate）の PR #<番号> を review してほしい。

まず次を読んでから始めること。
- .agents/skills/review-pull-request/SKILL.md
- .agents/skills/review-pull-request/references/<mode>.md

mode: <review | verify-comments>
対象 head SHA: <full SHA>
review cycle ID: <新規に作る | 元 review の ID>
関連 Issue: #<番号>
特に確認してほしい点: <観点>

完了後、次を報告すること。
<「返させるもの」の各項目>
```

### 返させるもの

| 項目 | 用途 |
| --- | --- |
| review cycle ID | P5 への受け渡しと突合 |
| 確認した head SHA | P3 / P6 で対象の同一性を確認する |
| 投稿した完了要約の URL | read-back と終了報告 |
| 指摘の一覧 | 1 件 1 行。prefix（`[must]` / `[ask]` / `[imo]` / `[nits]` / `[fyi]`）、対象 path、thread URL を含める |
| 件数 | 指摘の総数と prefix 別の内訳 |
| 実施した検証と未実施事項 | P3 / P6 の判断材料 |
| 未収束事項 | エスカレーション判断 |

subagent の最終報告はユーザーへ表示されない。orchestrator が受け取った内容を要約してユーザーへ伝える。

### subagent が使えない環境

subagent 機構を持たない agent では、同一 agent が review を実行してよい。ただし実装 context を持ったまま review すると指摘の質が落ちるため、次のいずれかを取り、取った方法を PR へ記録する。

- review の入力を GitHub 上の PR diff と head SHA に限り、実装時の判断を根拠に使わない。
- review を別 session として開始する。
- review をユーザーまたは別の agent に委ねる。

## 判断基準（P3 / P6）

「対応すべきものがあるか」は、`review-pull-request` の `.agents/skills/review-pull-request/references/address-comments.md` が定める処置の分類で判定する。分類の定義を本 skill に複製しない。

- 採用して修正するものが 1 件以上あれば P4 へ進む。
- すべてが「スコープ外」「既存実装で充足」「再現しない」「guideline と競合」「outdated / duplicate」で処置できる場合も、返信は必要である。push を伴わなくても P4 を実施し、P5 へ進む。
- 「追加情報が必要」が残る場合は、その件をユーザーへ上げてから続行の可否を判断する。

指摘の正しさは推論で決めず、確認して判断する（`.agents/skills/review-pull-request/references/address-comments.md` の手順 4）。

## head SHA と CI

- 各フェーズで確認した head SHA を記録し、次フェーズへ渡す。P4 の push で head は動くため、P5 は新しい head を対象とする。
- P2 と P5 へ委譲する直前に check runs を確認する。失敗している場合は、その失敗も review / 再確認の対象に含めることを brief へ明記する。
- 想定していないタイミングで head が動いていた場合は、古い diff を前提に投稿せず context を取り直す（`review-pull-request` の「対象 PR の特定と review source」）。

## working branch note

各フェーズの終わりに note を更新する。

- `セッションログ`: そのフェーズで何をしたか（review cycle ID、head SHA、指摘件数）を 1〜2 行で残す。
- `検証`: 実行した検証と結果を残す。未実施は理由とともに残す。
- `リスク・ブロッカー`: 未収束の指摘と未検証事項を残す。

## skill を変更する Issue を処理するとき

本 skill を含む skill 自体を追加・変更する Issue をこのフローで処理する場合、次の 2 点を守る。

1. **追加・変更した skill を同じ session で使う場合は、repo 相対 path で直読する。** skill の検出タイミングは実行環境に依存する。cloud session では session 途中に追加した skill が再起動なしで検出された実測があるが、検出は保証されない。フローを skill 一覧への登録に依存させず、path 直読を既定とする。subagent へ渡す brief も同様に path で指定する。
2. **description による発火は未検証事項として記録する。** skill が一覧に載ることを確認できても、description が意図した場面で実際に発火するかは、その場面を再現しない限り確認できない。note と PR に未検証事項として書き、確認手段（ローカルは Claude Code の再起動、cloud session は新しい session の開始）を添える。

さらに、自分が設計した skill を自分で読む場合、記述が不足していても context で補完して動けてしまう。このため **SKILL.md の自己完結性そのものを review 観点として brief に含める**。fresh context の subagent が曖昧と感じた箇所は、そのまま指摘として扱う。

## 停止とエスカレーション

| 状況 | 扱い |
| --- | --- |
| Issue の依存が未完了 | 開始せず、未完了依存を報告して終了する |
| Issue の指示と `doc/design/` の仕様が食い違う | 実装で解釈を補わず、止めて報告する（`doc/guidelines/issue-driven-task-execution.md`） |
| 反復上限に達しても収束しない | 自動反復を打ち切り、未収束の指摘、見解の相違点、推奨する次の対応を報告する |
| CI が落ち続け、原因を特定できない | 推測で修正を重ねず、失敗内容と切り分け結果を報告する |
| head SHA が繰り返し動く | 自分以外の push が入っている。context を取り直し、繰り返すなら報告する |
| subagent が完了要約を投稿できない | read 系 tool で反映を確認し、未反映なら報告する（`review-pull-request` の「MCP write failure の安全手順」） |
| 対象 PR が closed / merged になった | 処理を停止して報告する |

## 反復上限

`review-pull-request` の「反復の上限」に委ね、本 skill で別の上限を定義しない。同一 review cycle につき P4 → P5 を 2 周までとする。

## 終了時の報告

フロー終了時に次を報告する。

- PR の URL、state、最終 head SHA。
- review cycle ID と、各周の指摘件数および処置の内訳。
- resolve 可マーカーを付けた thread の一覧。
- 未収束の指摘と、その見解の相違点。
- 未検証事項（skill を追加・変更した場合は discoverability を含む）。
- 人間に残る作業: inline thread の resolve と PR の merge。

## やらないこと

- PR の merge。
- inline thread の resolve。
- `APPROVE` / `REQUEST_CHANGES` の投稿。
- 複数 Issue の並行実行。
- 反復上限を超える自動反復。
- subagent の出力を検証せずに採用する。
- 実行していない検証を実行したものとして記録する。
