<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# model の識別子の記載範囲と review metadata の Model

- 状態: decided
- 作成日: 2026-09-23
- 最終更新日: 2026-09-24
- 関連: `doc/guidelines/pull-request-guidelines.md`, `.agents/skills/review-pull-request/SKILL.md`, `doc/design/decision-log/0017-pr-tool-name-restriction.md`, `doc/design/decision-log/0019-issue-review-cycle-orchestration.md`

## 背景

PR #67（Issue #64）の review cycle で、review の canonical metadata の `Model` が投稿によって食い違った。subagent が投稿した review 本文には実行環境で確認した model の識別子が入ったが、orchestrator が投稿した address-comments の返信と verify-comments の完了要約は `unknown` だった。識別子は実行環境から確認できていた。

`unknown` とした根拠は、cloud session の harness が agent に与える既定の指示（model の識別子を commit message、PR の title と本文、code comment など repository に push する成果物に含めない）だった。orchestrator がこれを PR のコメントにも当てはめた。

bizdate の正本はどちらも、この場面を決めていなかった。

- 0017 と `doc/guidelines/pull-request-guidelines.md` は tool 名の扱いだけを定め、model の識別子に触れていない。
- `review-pull-request` の canonical metadata は、`Model` に実行環境で確認できる値を書き、確認できない場合だけ `unknown` とする。実行環境の既定の指示と食い違う場合にどちらに従うかは書いていない。

このため agent によって解釈が分かれた（Issue #68）。

## 候補

記載してよい範囲:

- A: tool 名と同じく、禁止を PR title に限る。それ以外は制限せず、書き手の裁量に任せる。
- B: PR title に加え、code と文書の本文でも禁止する。記録は trailer と PR 側に限る。
- C: harness の既定の指示に揃え、repository に残るもの（commit message、PR の title と本文、code と文書）では禁止し、PR と review のコメントだけ許可する。

review metadata の `Model`:

- D: 実行環境で確認した識別子を書く。実行環境の既定の指示を理由に `unknown` にしない。
- E: 実行環境の既定の指示が識別子を禁じる場合は `unknown` とする。
- F: D を基本とし、実行環境の指示が PR と review のコメントへの記載まで禁じる場合（または対象外と確認できない場合）に限って `unknown` とし、その旨を投稿に残す。

## 検討内容

- A: 0017 が tool 名について取った理由（禁止理由の可読性は title だけで満たせる、各 tool の既定動作と恒常的に衝突させない）が model の識別子にもそのまま当てはまる。model の識別子は tool 名と同じく title の一覧性を損なうため、title では禁止する。
- B: code と文書に識別子を書いて困った事例はまだ無い。`doc/guidelines/agent-configuration-management.md` の「先回りで仮想シナリオに備えない」に反する。
- C: 特定の harness の既定の指示を repository のルールへ写すことになる。harness ごとに指示が違い、変わり得るため、repository のルールとして追従しきれない。
- A の範囲では、任意の記載を harness の指示に従って省くことは repository のルールと衝突しない。衝突が起き得るのは、記載を必須とする review metadata の `Model` だけである。
- D: `Model` は review cycle を後から辿るための記録であり、同じ cycle の投稿で値が揃わないと記録として使えない。PR #67 の食い違いはこれに当たる。canonical metadata は PR と review のコメントに置かれ、repository に push される成果物ではない。
- E: 同じ Agent 種別でも投稿者（orchestrator と subagent）の解釈で値が変わる。記録目的を満たさない。
- D だけでは、実行環境の指示がコメントへの記載まで実際に禁じる場合も「書け」と読める。repository の guideline は agent の上位の system 指示を上書きできないため、その環境では agent が両方を守れない（PR #72 のクロスレビューの指摘）。#68 の事例で必要なのは、repository に push する成果物への禁止を PR コメントへ誤って広げないことであり、適用範囲で表せば足りる。
- F: #68 の事例（指示の対象が repository に push する成果物で、コメントを含まない）は D と同じ結果になる。コメントまで禁じる指示がある環境では上位の指示に従い、`unknown` の理由を投稿に残すことで「確認できなかった」場合と区別できる。

## 決定

- model の識別子の記載を禁止するのは **PR title** だけとする。tool 名、tool 由来の prefix と並べて `doc/guidelines/pull-request-guidelines.md` の「Tool 名と model の識別子の扱い」に置く。
- PR description、PR と review のコメント、commit message（trailer を含む）、code と文書の本文では制限しない。書くかどうかは書き手の裁量とし、harness の既定の指示に従って省いてもよい。この範囲は tool 名にも同じく適用する。
- review の canonical metadata の `Model` は必須のキーとし、実行環境で確認した識別子を書く（候補 F）。
  - 実行環境の指示が識別子の記載を禁じていても、PR と review のコメントがその対象外と確認できる場合は、それを理由に `unknown` にしない。
  - 指示がコメントへの記載まで禁じている場合、または対象外と確認できない場合は、上位の指示を repository のルールで上書きせず `unknown` と書き、上位の指示で記載を控えた旨を同じ投稿の本文に 1 行残す。
  - それ以外で `unknown` を使うのは、識別子を確認できない場合に限る。
- 正本は、記載範囲を `doc/guidelines/pull-request-guidelines.md`、`Model` のキー定義を `.agents/skills/review-pull-request/SKILL.md` とし、後者から前者を参照する。
- 確認の手段（cloud session での session 情報の取得など）は本ログで決めず、Issue #69 で扱う。

## 理由

禁止は、それが無いと困る場所（title の一覧性）にだけ置く。0017 と同じ考え方を model の識別子へ広げたもので、ルールを増やさずに済む。review metadata の `Model` は記録として揃うことに意味があるため、必須のキーだけは repository のルールで値の書き方を固定する。

## 影響

- `doc/guidelines/pull-request-guidelines.md` の「基本方針」と「Tool 名の扱い」を改訂し、節名を「Tool 名と model の識別子の扱い」にした。
- `.agents/skills/review-pull-request/SKILL.md` の「可視 metadata の canonical フォーマット」の `Model` の項に、コメントが実行環境の指示の対象外なら `unknown` にしないこと、対象なら `unknown` とし理由を残すことを追記した。
- 入口 shim（`.claude/rules/pull-request-guidelines.md`、`.cursor/rules/pull-request-guidelines.mdc`）は title の禁止内容を要約しているため同期した。`AGENTS.md` と `CLAUDE.md` は正本への参照だけを持ち、変更しない。
- `.github/copilot-instructions.md` は同期しない。Copilot code review は PR の diff を見るものであり、PR title と review metadata の書き方を扱わない。tool 名の扱いも元から載せていない。
- commit message の扱いは `doc/guidelines/git-operation-guidelines.md` に書かない。制限しない以上、commit 時に読む必要がないためである。
- 「PR title 以外では制限しない」は節全体に掛かるため、tool 名についても commit message と code / 文書の本文で制限しないことを明文化した。0017 は commit message を無規定とし、code / 文書に触れていなかった。どちらも実質の扱いは変わらず（無規定は制限しないことと同じで、文書は `.claude/rules/` のような tool 名を含む path を元から書いている）、記載範囲を 1 か所で読めるようにしたものである。0017 の決定（禁止は PR title のみ）は変わらない。

## 後から見直す条件

- code や文書の本文に識別子が書かれ、陳腐化や混乱が実際に問題になったとき。
- harness の指示が PR と review のコメントにも及ぶよう変わり、理由付きの `unknown` が増えて `Model` が記録として使えなくなったとき。
