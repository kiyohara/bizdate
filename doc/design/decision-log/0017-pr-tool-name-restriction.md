<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# PR での tool 名記載範囲

- 状態: decided
- 作成日: 2026-09-12
- 最終更新日: 2026-09-12
- 関連: `doc/guidelines/pull-request-guidelines.md`, `doc/design/decision-log/0011-ai-agent-lineup.md`, `doc/design/decision-log/0012-development-loop.md`

## 背景

`doc/guidelines/pull-request-guidelines.md` は当初、「PR title / description には、作成に使った tool 名やそれを示す prefix を含めない」と定めていた。`[codex]` のような prefix が title に付き、レビュー一覧で変更内容が読み取りにくくなることを避ける意図だった。

その後、AI agent の利用が一般化し、`Co-Authored-By` や `🤖 Generated with ...` のような attribution trailer を description に付ける運用が各 tool の標準になった。実際に Issue #34 の本文にも trailer が入っており、正本と実運用が食い違っていた。

類似構成の別プロジェクトで同じ緩和が先に入っており、その判断を bizdate にも取り込む（Issue #43）。

## 候補

- A: 現状維持（title / description の両方で禁止）
- B: 禁止を title のみに限定し、description と PR コメントは制限しない
- C: 禁止を撤廃する

## 検討内容

- A: trailer を付けるたびに正本違反となる。tool 側の既定動作に逆らって毎回取り除く運用コストが継続的にかかり、遵守率も下がる。
- B: 禁止理由（レビュー一覧での可読性）は title だけで満たせる。description は本文を読む場面であり、末尾の trailer が変更内容の把握を妨げない。attribution が残ることは、どの agent が書いたかを後から辿る材料としてむしろ有用である。
- C: title に `[codex]` のような prefix が戻る余地を残す。当初の問題意識が失われるため採らない。
- `doc/guidelines/agent-configuration-management.md` の「ルールをシンプルに保つ」に従い、trailer 以外の表現は禁止も許可も明言せず、書き手の裁量に任せる。

旧ルールにあった「変更対象の実ファイルパスとして `.cursor/rules/` や `.claude/rules/` などを書く必要がある場合は、その範囲に限って記載してよい」という例外条項は削除する。禁止対象が title の tool 名・prefix に限定された結果、ファイルパスが抵触しないことは文面から自明であり、行を残す必要がない。

## 決定

- 禁止対象は **PR title** の tool 名と tool 由来 prefix のみとする。
- PR description と PR コメントは制限しない。`Co-Authored-By` や `🤖 Generated with ...` のような trailer は明示的に許可する。
- trailer 以外の description の表現は、禁止も許可も明言せず書き手の裁量に任せる。
- commit message における tool 名の扱いは、`doc/guidelines/git-operation-guidelines.md` が元々無規定であり、今回も変更しない。

## 理由

title はレビュアーが変更内容を一覧で把握するためのものであり、ガードレールとしてはここだけで足りる。description まで広げると、各 tool の既定動作と恒常的に衝突し、守られないルールが残るだけになる。

## 影響

- `doc/guidelines/pull-request-guidelines.md` の「基本方針」と「Tool 名の扱い」を改訂した。
- 入口 shim（`.claude/rules/pull-request-guidelines.md`、`.cursor/rules/pull-request-guidelines.mdc`）は禁止内容の要約行を持つため同期した。
- `.agents/skills/number-working-branch-note/SKILL.md` の Step 10 にあった「tool 名なし」の要約を正本に揃えた。
- 過去の working branch note に旧方針の記述が残るが、note は当時の判断記録であり最終仕様書ではないため、`doc/guidelines/working-branch-notes-handling.md` のメンテコスト判断に従い据え置く。

## 後から見直す条件

- title に tool 名や prefix が混入する事例が繰り返し発生し、title の禁止だけでは抑止できないと分かったとき。
- description の体裁のばらつきがレビューの妨げになり、構成や表現に追加のルールが必要になったとき。
