# 作業ブランチメモ

- ブランチ: `claude/issue-82-g09n08`
- PR:
- 最終更新: 2026-09-25

## 目的

Issue #82。`drive-issue-to-reviewed-pr` を `--from-pr` で P2 から始めるとき、先に P1 の完了条件を確かめ、満たしていなければ P1 の残りの手順から進めるようにする。P1 の途中（PR の作成後、採番の前など）で session が途切れて `--from-pr` で再開すると、採番、`progress.md` の更新、`run-issue-task` の報告から引き上げた項目の記録が飛ばされるため。

## 現在の状況

- cloud session（Claude Code on the web）で、`drive-issue-to-reviewed-pr` の P1 として進めている。#82 は依存なし、`progress.md` の索引外の単発 Issue である。
- #86 と並列に進めるトライアルの 1 つ（ユーザーの承認による一回限りの例外）。直列消化の確認は今回に限り省く。#86 の branch と PR には触らない。decision log は 0019 への追記とし、新規が要る場合は 0028 を使う（0027 は #86 に予約）。衝突は main の merge で解き、rebase と force push をしない。
- SKILL.md の「入力と入口」に「`--from-pr` で P2 から始めるとき」を足し、「終了時の報告」に 1 文足した。0019 に追記し、index の 0019 行に 1 文足した。

## 決定事項

- 確認は P2 から始める場合に限る。未収束の review cycle から P4 で再開する場合は対象外とした（Issue の範囲。本 skill の流れでは review cycle は P1 を終えた後の P2 で作られる）。
- 起点 Issue を持たない PR（索引登録、進捗整理、リリース準備）は `run-issue-task` で作らないため、確かめずに P2 から始める。既存の「追加の review 観点」が `--from-pr` でこれらの PR を扱うことを想定しており、書かないと「対象 Issue を決められない」として止まり得るため、明記した。
- P1 の完了条件は push までを含むため、PR の head の内容で確かめる。PR が open であることは既存の入口の確認で済んでいる。
- 各条件の確かめる手段と残りの手順は、表で `run-issue-task` の手順 6 / 7 / 9、`number-working-branch-note`、「note の探し方」、本 skill の「working branch note」への参照として書いた。手順の中身は複製していない。
- 採番を前の session で行い、採番の報告が手元に無い場合は、報告が無いことと採番の commit を note に残し、未解決事項として報告して進める。報告は推測で埋めない。ユーザーに確認しても失われた報告は戻らず、進め方も変わらないため止めない。`run-issue-task` の 0 件と「呼ばなかった」を分ける規定と同じく、報告が落ちたことを区別できるようにする。`run-issue-task` は Issue のスコープ外のため変えず、本 skill 側に書いた。
- 一意に読めない例として、番号付き note と `draft_` の note が両方ある（`number-working-branch-note` の Step 3 も止まる）、どちらも無い、記録どうしが食い違う、を挙げた。
- 残りの手順が止まった場合（`number-working-branch-note` の停止など）は P2 に進まず、止まった理由と未反映の変更を報告する。
- frontmatter の description は変えない。`--from-pr` の用途（既存 PR を指定して review 以降だけを回す）は変わらず、P1 の残りはその前提を整える手順であるため。
- decision log は新規を作らず 0019 に追記した。

## 次にやること

- [x] SKILL.md の「入力と入口」に確認の規定を足す
- [x] 0019 に追記し、index の要約を揃える
- [x] Issue の「検証」を行い、結果を残す
- [ ] PR を作成し、note を採番する
- [ ] review cycle を回す

## 検証

cloud session で実行した。

### 状態ごとの開始フェーズ（SKILL.md を読んで確かめた）

変更後の SKILL.md の「入力と入口」を、各状態を想定して読んだ。いずれも PR は open で、`Closes #<番号>` を持つ Issue 駆動の PR とする（最後の 2 行を除く）。上の 4 行が Issue の指定した状態、それ以降は境界の確認のために足した状態である。実装した本人が読んだ結果であり、P2 の review で fresh context の reviewer に改めて確かめてもらう。

| 状態 | 開始フェーズ | P2 の前に行う手順 |
|---|---|---|
| note が採番前、review cycle が無い | P1 の残り → P2 | 「note の探し方」で `draft_` だけが見つかる → 採番（`run-issue-task` の手順 9、`number-working-branch-note`）。索引にある Issue なら手順 7 の PR 欄も確かめる。採番の報告を「working branch note」に従って `セッションログ` に残し、push する |
| 採番済み、索引にある Issue の `progress.md` が未更新、review cycle が無い | P1 の残り → P2 | `run-issue-task` の手順 7 で索引の行を更新し、push する |
| 採番済み、引き上げた項目が note に無い、review cycle が無い | P1 の残り → P2 | 採番の報告が手元にあれば、「working branch note」に従って残す。無ければ（前の session で採番した場合など）、報告が無いことと採番の commit を残す。push し、終了時に未解決事項として報告する |
| P1 の完了条件をすべて満たし、review cycle が無い | P2（従来どおり） | なし |
| 番号付き note と `draft_` の note が両方ある | 開始しない | ユーザーに確認する（一意に読めない） |
| note がどちらも無い | 開始しない | ユーザーに確認する（一意に読めない） |
| note の `検証` に Issue の検証の結果が無い | P1 の残り → P2 | `run-issue-task` の手順 6 を行い、push する |
| 残りの手順が止まる（採番で、PR の head branch と現在の branch が一致しないなど） | P2 に進まない | 止まった理由と未反映の変更をユーザーへ報告する |
| `Closes` が無い PR（索引登録、進捗整理、リリース準備） | P2（従来どおり） | なし（P1 の完了条件を当てはめない） |
| 未収束の review cycle がある | P4（従来どおり） | なし（確認は P2 から始める場合に限る） |

### その他

| 項目 | 結果 |
|---|---|
| repo 相対 path の存在 | 追加した行の path（この note を含め 13 件）はすべて存在する |
| 参照先の節名の存在 | 追加した行の「」の参照（「note の探し方」「フェーズ」「head SHA と CI」「working branch note」「入力と入口」「終了時の報告」「`--from-pr` で P2 から始めるとき」「Issue 番号がある場合」「被委譲 skill の報告の引き上げ」「反復上限の置き場」）は、参照先に同名の見出しがある。「呼ばなかった」は `run-issue-task` の本文の語の引用である |
| `run-issue-task` の手順番号 | 「Issue 番号がある場合」の手順 6（検証）、7（`progress.md`）、9（note の rename）が表の参照と一致する |
| 文体 | 追加した行は常体。ですます調の文末は無い（`doc/guidelines/document-style-guidelines.md` の開発者向け） |
| `git diff --check` | OK |
| note の情報統制 | `doc/guidelines/working-branch-notes-security.md` の「編集時の確認」の観点で確認し、該当なし |
| Rust の検証（`cargo fmt` / `clippy` / `test`） | 省略。Rust のコードと Cargo の設定を変えていないため |

## リスク・ブロッカー

- 未検証: P1 の途中で途切れた PR を実際に `--from-pr` で再開する動作は試していない。上の表は SKILL.md を読んだ確認である。
- 未検証: skill の発火。frontmatter の description は変えていないため、発火の条件は変わらない。変えた本文が読まれることは、新しい session（cloud session）またはローカルの Claude Code の再起動の後に、skill を起動して確かめる。この session の本フローは変更前に読み込んだ skill で進めているが、変更は `--from-pr` の入口だけで、本フローは Issue の入口から始めたため影響しない。

## セッションログ

- 2026-09-25: 着手。`drive-issue-to-reviewed-pr` の P1 として開始した。SKILL.md、0019、index を更新し、上の検証を行った。
