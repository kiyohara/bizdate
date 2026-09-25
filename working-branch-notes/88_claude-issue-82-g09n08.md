# 作業ブランチメモ

- ブランチ: `claude/issue-82-g09n08`
- PR: #88
- 最終更新: 2026-09-25

## 目的

Issue #82。`drive-issue-to-reviewed-pr` を `--from-pr` で P2 から始めるとき、先に P1 の完了条件を確かめ、満たしていなければ P1 の残りの手順から進めるようにする。P1 の途中（PR の作成後、採番の前など）で session が途切れて `--from-pr` で再開すると、採番、`progress.md` の更新、`run-issue-task` の報告から引き上げた項目の記録が飛ばされるため。

## 現在の状況

- cloud session（Claude Code on the web）で `drive-issue-to-reviewed-pr` を進めている。P2 の review（cycle `claude-code-42a9dd1-20260925050328`）の指摘 4 件に P4 で対応し、P5 の再確認で 4 件とも resolve 可、新規の指摘は 0 件だった。review cycle は 1 周で完了した。#82 は依存なし、`progress.md` の索引外の単発 Issue である。
- #86 と並列に進めるトライアルの 1 つ（ユーザーの承認による一回限りの例外）。直列消化の確認は今回に限り省く。#86 の branch と PR には触らない。decision log は 0019 への追記とし、新規が要る場合は 0028 を使う（0027 は #86 に予約）。衝突は main の merge で解き、rebase と force push をしない。
- SKILL.md の「入力と入口」に「`--from-pr` で P2 から始めるとき」を足し、「停止とエスカレーション」に 1 行、「終了時の報告」に 1 文足した。0019 に追記し、index の 0019 行に 1 文足した。

## 決定事項

- 確認は P2 から始める場合に限る。未収束の review cycle から P4 で再開する場合は対象外とした（Issue の範囲。本 skill の流れでは review cycle は P1 を終えた後の P2 で作られる）。
- 起点 Issue を持たない PR（索引登録、進捗整理、リリース準備、Dependabot の更新 PR など）は `run-issue-task` で作らないため、確かめずに P2 から始める。既存の「追加の review 観点」が `--from-pr` でこれらの PR を扱うことを想定しており、書かないと「対象 Issue を決められない」として止まり得るため、明記した。起点 Issue を持つかどうかや対象 Issue を決められない場合（`Closes` が無い、または複数ある）は、「追加の review 観点」と同じくユーザーに確認する（P2 の指摘 3）。
- P1 の完了条件は push までを含むため、PR の head の内容で確かめる。PR が open であることは既存の入口の確認で済んでいる。
- 各条件の確かめる手段と残りの手順は、表で `run-issue-task` の手順 6 / 7 / 9、`number-working-branch-note`、「note の探し方」、本 skill の「working branch note」への参照として書いた。手順の中身は複製していない。
- 採番を別の session で行い、採番の報告が手元に無く note にも残っていない場合は、引き上げた項目の記録を対象外とし、報告が無いことと採番の commit を終了時の報告に書く。note には残さず、未解決事項にもしない（0019 の G3）。当初は note に残して未解決事項にする案（G2）としたが、`run-issue-task` を単独で使って作った PR では報告が同 skill の手順 10 で行われ、note に残す規定が無い。G2 では別の session からの再開で必ず commit が増え、行われた報告を未解決事項として扱うことになるため改めた（P2 の指摘 2）。報告は推測で埋めない。ユーザーに確認しても失われた報告は戻らず、進め方も変わらないため止めない。`run-issue-task` は Issue のスコープ外のため変えず、本 skill 側に書いた。
- 一意に読めない例として、番号付き note と `draft_` の note が両方ある（`number-working-branch-note` の Step 3 も止まる）、どちらも無い、記録どうしが食い違う、を挙げた。
- 残りの手順は PR の head branch で行う。local branch の対応は `.agents/skills/review-pull-request/references/address-comments.md` の手順 6 と同じく確かめ、push したら GitHub 上の PR の head で条件を確かめ直す（同じ reference の手順 7）。別の cloud session から再開すると、session の branch に push して PR の head が変わらないまま P2 に進み得るため（P2 の指摘 1）。kiyohara/slapex#233 の入口の規定のうち、P1 の残りの手順に当たる部分だけを取り込み、入口全体の規定は引き続き対象外とした。
- local branch を対応させられない、確かめ直して満たしていない、残りの手順が止まった（`number-working-branch-note` の停止など）場合は P2 に進まず、止まった理由と未反映の変更を報告する。「停止とエスカレーション」にも行を足した（P2 の指摘 4）。
- frontmatter の description は変えない。`--from-pr` の用途（既存 PR を指定して review 以降だけを回す）は変わらず、P1 の残りはその前提を整える手順であるため。
- decision log は新規を作らず 0019 に追記した。

## 次にやること

- [x] SKILL.md の「入力と入口」に確認の規定を足す
- [x] 0019 に追記し、index の要約を揃える
- [x] Issue の「検証」を行い、結果を残す
- [x] PR を作成し、note を採番する
- [x] review cycle を回す

## 検証

cloud session で実行した。

### 状態ごとの開始フェーズ（SKILL.md を読んで確かめた）

変更後の SKILL.md の「入力と入口」を、各状態を想定して読んだ。状態の欄に書かない限り、PR は open で `Closes #<番号>` を 1 つ持つ Issue 駆動の PR とし、local branch は PR の head branch に対応するものとする。「P1 の残り → P2」の行では、push したら GitHub 上の PR の head で条件を確かめ直してから P2 に進む。上の 5 行が Issue の指定した状態（3 つ目の状態は採番の報告の有無で 2 行に分けた）、それ以降は境界の確認のために足した状態である。実装した本人が読んだ結果であり、P2 の review の指摘 4 件を受けて直した後に読み直した。P5 では reviewer が変更後の SKILL.md だけを読み直し、13 行すべてで同じ結論になることを確かめた。

| 状態 | 開始フェーズ | P2 の前に行う手順 |
|---|---|---|
| note が採番前、review cycle が無い | P1 の残り → P2 | 「note の探し方」で `draft_` だけが見つかる → 採番（`run-issue-task` の手順 9、`number-working-branch-note`）。索引にある Issue なら手順 7 の PR 欄も確かめる。採番の報告を「working branch note」に従って `セッションログ` に残し、push する |
| 採番済み、索引にある Issue の `progress.md` が未更新、review cycle が無い | P1 の残り → P2 | `run-issue-task` の手順 7 で索引の行を更新し、push する |
| 採番済み、引き上げた項目が note に無く、採番の報告が手元にある（この session で `run-issue-task` を単独で使った後など）、review cycle が無い | P1 の残り → P2 | 「working branch note」に従って残し、push する |
| 採番済み、引き上げた項目が note に無く、採番の報告が手元に無い（別の session で採番した。`run-issue-task` を単独で使った PR を含む）、review cycle が無い | P2 | なし（対象外）。終了時の報告に、報告が無いことと採番の commit を書く |
| P1 の完了条件をすべて満たし、review cycle が無い | P2（従来どおり） | なし |
| 番号付き note と `draft_` の note が両方ある | 開始しない | ユーザーに確認する（一意に読めない） |
| note がどちらも無い | 開始しない | ユーザーに確認する（一意に読めない） |
| note の `検証` に Issue の検証の結果が無い | P1 の残り → P2 | `run-issue-task` の手順 6 を行い、push する |
| 満たしていない条件があり、local branch を PR の head branch に対応させられない（別の cloud session から再開し、session の branch が PR の head branch と異なるなど） | P2 に進まない | 止まった理由と未反映の変更をユーザーへ報告する |
| 残りの手順が止まる（`number-working-branch-note` の停止など）、または push 後に PR の head で確かめ直して満たしていない | P2 に進まない | 止まった理由と未反映の変更をユーザーへ報告する |
| 起点 Issue を持たない PR（索引登録、進捗整理、リリース準備、Dependabot の更新 PR など） | P2（従来どおり） | なし（P1 の完了条件を当てはめない） |
| Issue 駆動の PR だが `Closes` が無い、または複数ある | 開始しない | ユーザーに確認する（対象 Issue を決められない） |
| 未収束の review cycle がある | P4（従来どおり） | なし（確認は P2 から始める場合に限る） |

### その他

| 項目 | 結果 |
|---|---|
| repo 相対 path の存在 | main との差分で追加した行の path 14 件（branch 名の `claude/issue-82-g09n08` を除く）はすべて存在する（P4 の修正後に確かめ直した） |
| 参照先の節名の存在 | 追加した行の「」の参照（「note の探し方」「フェーズ」「head SHA と CI」「working branch note」「入力と入口」「終了時の報告」「停止とエスカレーション」「渡すもの」「`--from-pr` で P2 から始めるとき」「Issue 番号がある場合」「被委譲 skill の報告の引き上げ」「反復上限の置き場」「編集時の確認」）は、参照先に同名の見出しがある。「追加の review 観点」は「渡すもの」の表の項目名、「呼ばなかった」は `run-issue-task` の本文の語の引用である |
| 参照した手順番号 | `run-issue-task` の「Issue 番号がある場合」の手順 6（検証）、7（`progress.md`）、9（note の rename）、10（報告）と、`.agents/skills/review-pull-request/references/address-comments.md` の手順 6（local branch の対応）、7（push の確認）が、参照した内容と一致する |
| 文体 | 追加した行は常体。ですます調の文末は無い（`doc/guidelines/document-style-guidelines.md` の開発者向け） |
| `git diff --check` | OK |
| note の情報統制 | `doc/guidelines/working-branch-notes-security.md` の「編集時の確認」の観点で確認し、該当なし |
| Rust の検証（`cargo fmt` / `clippy` / `test`） | 省略。Rust のコードと Cargo の設定を変えていないため |

## リスク・ブロッカー

- 未検証: P1 の途中で途切れた PR を実際に `--from-pr` で再開する動作（別の cloud session からの再開を含む）は試していない。上の表は SKILL.md を読んだ確認である。
- 未検証: skill の発火。frontmatter の description は変えていないため、発火の条件は変わらない。変えた本文が読まれることは、新しい session（cloud session）またはローカルの Claude Code の再起動の後に、skill を起動して確かめる。この session の本フローは変更前に読み込んだ skill で進めているが、変更は `--from-pr` の入口だけで、本フローは Issue の入口から始めたため影響しない。
- 既知の限界（P5 の評価）: 本 skill の P1 が採番と note への記録の間で途切れた場合、`number-working-branch-note` が触らなかった行（残された事項）は採番の commit の差分に現れない。終了時の報告では、報告が手元に無い旨でしか示せない。単独の `run-issue-task` で作った PR と見分ける印が無いため、0019 の G3 の trade-off として受け入れた。

## セッションログ

- 2026-09-25: 着手。`drive-issue-to-reviewed-pr` の P1 として開始した。SKILL.md、0019、index を更新し、上の検証を行った。
- 2026-09-25（P1）: PR #88 を draft で作成し、`number-working-branch-note` で note を採番した（commit `01de740`）。完了として書き換えたタスク行は、note の「PR を作成し、note を採番する」の 1 件（PR description には該当なし）。触らなかった stale 表現・タスク行は 0 件。#82 は索引外のため `progress.md` は変えていない。reviewer の指定は、PR の作成者と同じ account のため GitHub が受け付けなかった（assignee は設定済み）。
- 2026-09-25（P2・P3）: review cycle `claude-code-42a9dd1-20260925050328`、head `42a9dd1`。指摘 4 件（must 0 / ask 2 / imo 2）。4 件とも採用とした。
- 2026-09-25（P4）: 4 件に対応した（commit `0eed0b0`）。残りの手順を PR の head branch で行い push 後に確かめ直す規定、`run-issue-task` を単独で使った PR の引き上げた項目の扱い（G2 から G3 へ）、起点 Issue を判別できない PR の扱い、「停止とエスカレーション」の行を足した。0019 とこの note を揃えた。
- 2026-09-25（P5・P6）: head `0eed0b0` を同じ subagent が再確認し、4 件とも resolve 可、新規の指摘は 0 件（完了要約 1 本）。1 周で収束したため、フローを終えた。
