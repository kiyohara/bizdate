<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# Issue 起点 review cycle の orchestration

- 状態: decided
- 作成日: 2026-09-12
- 最終更新日: 2026-09-12
- 関連: `doc/guidelines/development-loop.md`, `doc/guidelines/issue-driven-task-execution.md`, `.agents/skills/drive-issue-to-reviewed-pr/SKILL.md`, `.agents/skills/run-issue-task/SKILL.md`, `.agents/skills/review-pull-request/SKILL.md`, `doc/design/decision-log/0012-development-loop.md`, `doc/design/decision-log/0018-cloud-session-environment.md`

## 背景

`run-issue-task` と `review-pull-request` は個別 skill として揃っていたが、Issue 着手から「レビュー済み PR」に到達するまでの流れは、毎回ユーザーが手順を書き起こした prompt で起動していた（Issue #50）。常用していた流れは、実装と PR 作成 → review → 判断 → 対応 → 再確認 → 判断の 6 段であり、review と再確認は subagent に実行させていた。

prompt を毎回書き起こす運用では、次が毎回曖昧になっていた。

- subagent へ渡す入力と、返させる出力の interface。
- 「対応すべきものがあるか」の判断基準。
- 対応の push で PR head SHA が動くために必要な、各段での head SHA の受け渡し。
- CI（check runs）を確認する位置。
- 反復上限。`review-pull-request` 側に「同一 review cycle につき 2 周」があり、prompt 側にも上限を書くと二重定義になる。
- 依存未完了、仕様解釈が必要、収束しない、といった停止条件。
- 途中再開（PR が既にある状態で review から入る）の入口。

## 候補

### orchestration の置き方

- A: orchestrator skill を 1 つ追加し、既存 2 skill へ委譲する。
- B: `run-issue-task` を拡張し、PR 作成後の review cycle まで同一 skill で扱う。
- C: `doc/guidelines/development-loop.md` に流れを書き、skill は増やさない。
- D: prompt template をリポジトリに置くだけにする。

### review の実行主体

- R1: review と再確認を subagent へ委譲し、実装担当と分離する。
- R2: 同一 agent が実装に続けて review する。
- R3: review は常にユーザーまたは別 tool に委ねる。

### 反復上限の置き場

- L1: `review-pull-request` の既存上限に委ね、orchestrator では定義しない。
- L2: orchestrator 側に独自の上限を持つ。

## 検討内容

**orchestration の置き方**。B は `run-issue-task` の責務を「Issue を 1 件実行する」から「review cycle まで回す」へ広げる。review だけを回したい場合や、PR 作成までで止めたい場合に skill を分割し直す必要が出るため、責務境界が保てない。C は流れを文書化できるが、guideline は「いつ正本を読むか」と恒久ルールを置く場所であり、委譲 interface や brief の雛形のような実行手順を置くと `doc/guidelines/development-loop.md` が肥大化する。skill と guideline の役割分担（`doc/guidelines/development-loop.md` の「各資材の役割」）にも反する。D は毎回の書き起こしを減らすが、判断基準と停止条件が正本として保守されないため、今回の課題が解決しない。A は既存 2 skill の責務を変えずに呼び出し順と境界だけを固定でき、個別 skill の単独利用も残る。

**review の実行主体**。R2 は実装 context を持つ agent が自分の変更を review することになり、自分の判断を妥当と見做しやすい。今回の Issue 自身がその典型で、skill の設計意図を持つ agent は SKILL.md の記述が不足していても context で補完して動けてしまい、記述不足を検出できない。R3 は質は高いが、ユーザーの手番を毎回必須にするため常用フローにならない。R1 は fresh context の reviewer を立てられ、`review-pull-request` が既に前提としている役割分担（修正担当 Agent は自分で resolve せず、元の Review 担当 Agent が再確認する）にもそのまま乗る。subagent 機構を持たない agent があるため、fallback は必要である。

**反復上限の置き場**。L2 は orchestrator と被委譲 skill の両方に上限が現れ、値が食い違ったときにどちらが正かを決められない。L1 は上限の正本を 1 つに保てる。orchestrator は「上限に達したら打ち切ってエスカレーションする」という扱いだけを持つ。

**再確認を行う subagent**。`review-pull-request` の `verify-comments` は「元の Review 担当 Agent が再検証する」前提で、担当一致を可視 metadata（review cycle ID の agent-slug と `Agent`）で確認する。review を実行した subagent を再利用すれば review 時の判断 context が残る。session の再開などで subagent が失われる場合があるため、GitHub 上の可視 metadata から context を再構築する新規 subagent への fallback を持たせる。担当一致は Agent 種別で判定されるため、どちらでも満たせる。

**skill を追加・変更する Issue をこのフローで処理する場合**。追加した skill を同じ session で使えるかは実行環境に依存する。cloud session での実測では、`.claude/skills/` の symlink と `SKILL.md` を作成した直後に、session の再起動なしで skill が利用可能として検出された。ただし検出は保証されないため、フローを skill 一覧への登録に依存させず、repo 相対 path での直読を既定とする。description による発火の確認は、その場面を再現しない限り同 session ではできないため、未検証事項として扱う。

## 決定

- orchestrator skill `drive-issue-to-reviewed-pr` を追加する（A）。既存 2 skill の責務は変えず、呼び出し順、委譲境界、判断基準、head SHA と CI の確認点、停止条件、終了報告だけを固定する。
- review と再確認は subagent へ委譲し、実装担当（orchestrator 自身）と分離する（R1）。subagent 機構が使えない環境の fallback を skill に記述する。
- 再確認を行う subagent は、review を実行した subagent の再利用を第一選択とし、失われている場合は GitHub 上の可視 metadata から context を再構築する新規 subagent へ fallback する。
- 反復上限は `review-pull-request` に委ね、orchestrator では定義しない（L1）。上限に達したら自動反復を打ち切り、未収束の指摘・見解の相違点・推奨する次の対応を報告する。
- 判断基準は `review-pull-request` の `address-comments` が持つ処置の分類を参照し、orchestrator に複製しない。
- PR の merge と inline thread の resolve は人間が行う前提を変えない。`APPROVE` / `REQUEST_CHANGES` も自動実行しない。
- skill を追加・変更する Issue をこのフローで処理する場合、同 session では repo 相対 path で直読し、description による発火は未検証事項として note と PR に記録する。あわせて、SKILL.md の自己完結性そのものを review 観点として subagent へ渡す。
- 本 skill は `progress.md` の索引登録と進捗整理の恒常的な例外には含めない。orchestrator は Issue 起点の通常作業を回すものであり、起点 Issue を持つ（`doc/guidelines/development-loop.md` の「基本方針」）。

## 理由

- 既存 skill の責務境界を保ったまま、毎回 prompt に書き起こしていた判断基準と停止条件を正本化できる。
- review を fresh context へ分離することが、この体制で指摘の質を保つ唯一の実効手段である。実装担当と reviewer を同じ context に置くと、記述不足や前提の飛びが検出されない。
- 上限や分類の正本を 1 箇所に保つことで、orchestrator と被委譲 skill の二重定義を避けられる。ルールを増やすほど遵守率が下がる前提（`doc/guidelines/agent-configuration-management.md` の「ルールをシンプルに保つ」）にも沿う。
- merge と resolve を人間に残す前提は 0012 と `review-pull-request` の制約に由来し、orchestrator を挟んでも変える理由が無い。

## 影響

- `doc/guidelines/development-loop.md` の「使う skill」表に本 skill の行が入り、orchestrator と個別 skill の関係を 1 文で示す。個別 skill の単独利用は続く。
- `.agents/skills/run-issue-task/SKILL.md` と `.agents/skills/review-pull-request/SKILL.md` に、本 skill から委譲され得ることを示す参照が入る。手順そのものは変わらない。
- ユーザーが毎回書き起こしていた prompt は、skill の起動（Issue 番号、または `--from-pr <PR 番号>`）に置き換わる。
- subagent を 1 review cycle あたり 1〜2 本起動するため、同一作業の token 消費は prompt 運用時より増える。指摘の質と再現性を取る判断とする。
- Codex / Cursor では subagent 機構が Claude Code と同じではないため、fallback の経路で運用する。

## 後から見直す条件

- 被委譲 skill の責務やモード構成が変わり、orchestrator の委譲境界が実態と合わなくなったとき。
- subagent の再利用可否や識別方法が変わり、`verify-comments` の担当一致の判定が成立しなくなったとき。
- 反復上限 2 周で収束しない事例が続き、上限そのものを見直す必要が出たとき。
- subagent 起動のコストが、得られる指摘の質に対して見合わないと判断されたとき。
- Codex / Cursor 側で同等の分離が実装され、fallback の記述が不要になったとき。
