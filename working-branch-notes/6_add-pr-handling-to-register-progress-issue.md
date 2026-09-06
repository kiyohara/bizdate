# 作業ブランチメモ

- ブランチ: `add-pr-handling-to-register-progress-issue`
- PR: #6
- 最終更新: 2026-09-06

## 目的

Issue #5 の対応。`register-progress-issue` skill に、`progress.md` 編集後の commit / PR の扱いを追記する。

skill は編集手順までしか定めておらず、`main` が保護されている以上必ず PR が要るのに、独立 PR にするのか・起点 Issue を作るのか・`Closes` を付けるのかが読み取れなかった。実装タスクの索引登録を行う直前にこの抜けが判明した。

## 現在の状況

PR #6 を作成し、note を採番済み。review comment 7 件すべてへ対応した。開発ループ正本側への例外反映は、ユーザー判断により本 PR で実施した。

## 決定事項

- 追記内容は slapex の [PR #212](https://github.com/kiyohara/slapex/pull/212) の実運用に合わせる。同 PR は「起点 Issue は作らず、`Closes` 対象は無い」と明記し、専用ブランチの独立 PR として出している。
- 手順の末尾に新しい step を足すのではなく、**step 6 として PR 化を挿入し、既存の step 6・7 を 7・8 へ繰り下げる**。確認は PR 作成後に行うのが自然なため。
- 「参照する正本」に `git-operation-guidelines.md` / `pull-request-guidelines.md` / `working-branch-notes-handling.md` の 3 本を追加する。現状いずれも挙がっておらず、追記する手順がこれらを参照するため。review 対応で `working-branch-notes-security.md` を加えて計 4 本とした。
- 既存の step 6（Issue へ確認コメントを残す）は、`progress.md` 編集より前に行う判断もあり得るが、現行の順序を変えない。本 Issue のスコープは「commit / PR の扱いの追記」であり、既存手順の並べ替えは含まない。
- 索引登録の例外（起点 Issue を作らず `Closes` を付けない）を `doc/guidelines/development-loop.md` にも反映する。skill にしか例外が無いと、正本だけを追う agent が到達できないため。ユーザー判断で follow-up Issue ではなく本 PR で実施した。Issue #5 の スコープ外 は `development-loop.md` の **skill 表** の変更だけを除いており、skill 表は触っていない。

## review 対応（2026-09-06）

Codex と Claude Code の 2 cycle から inline comment 7 件。対応は次のとおり。

| 指摘 | 対応 |
|---|---|
| `[must]` 参照する正本に `working-branch-notes-security.md` が無い | 採用。1 行追加した |
| `[imo]` ブランチ / note の作成が `progress.md` 編集より後 | 採用。step 5 の冒頭に「編集前に step 6 の専用ブランチと note を作る」を置いた |
| `[imo]` 変更が生じなかった場合のガードが無い | 採用。step 6 の先頭に「変更が無ければ commit と PR を作らず終了する」を追加し、「終了報告」の PR URL も条件付きにした |
| `[imo]` 「やらないこと」の対象が広すぎる | 採用。「索引登録による `progress.md` の変更を〜」へ限定した |
| `[fyi]` note の stale 表現 | 採用。note を編集する機会に合わせて修正した |
| `[must]` / `[ask]` 索引登録の例外が開発ループ正本から辿れない | 採用。ユーザー判断により本 PR で `development-loop.md` の基本方針・資材表・標準フロー・参照順の 4 箇所へ反映した |

## 次にやること

- ユーザーの review と merge 判断を待つ。

## 検証

Issue #5 の検証項目 4 件をすべて実施した。

| 項目 | 結果 |
|---|---|
| `pull-request-guidelines.md`（merge は agent が行わない）との整合 | 一致。正本 L13 の「PR の merge は AI agent が行わない」に対し、skill の step 6 と「やらないこと」の 2 箇所で同趣旨を明記した |
| `number-working-branch-note` の commit / push step との表現整合 | 一致。両 skill とも「参照する正本」に `git-operation-guidelines.md` と `pull-request-guidelines.md` を挙げ、本文から同じ形で参照している |
| 既存 step 番号と「やらないこと」「終了報告」の整合 | step は 1〜8 で連続。既存の 6・7 を 7・8 へ繰り下げた。「やらないこと」に 2 項目、「終了報告」に PR URL を追加し、既存記述との矛盾は無い |
| 参照する正本の存在 | review 対応後は 9 件すべて存在（`AGENTS.md` / `development-loop.md` / `git-operation-guidelines.md` / `github-mcp-guidelines.md` / `issue-driven-task-execution.md` / `pull-request-guidelines.md` / `working-branch-notes-handling.md` / `working-branch-notes-security.md` / `progress.md`） |

`progress.md` は更新していない。Issue #5 は索引に載らない単発 Issue であり、`doc/guidelines/issue-driven-task-execution.md` step 7 の「表に載らない単発 Issue は `progress.md` を更新しなくてよい」に従った。

## リスク・ブロッカー

- なし。

## セッションログ

- 2026-09-06: Issue #5 に着手。main から分岐し、本 note を作成した。
- 2026-09-06: skill に step 6（PR 化）を挿入し、既存 6・7 を繰り下げ。「参照する正本」に 3 本、「やらないこと」に 2 項目、「終了報告」に PR URL を追加した。検証 4 項目を実施し、すべて通過した。
- 2026-09-06: PR #6 を作成し、note を採番した。
- 2026-09-06: review comment 7 件へ対応。skill 記述に閉じる 5 件を反映し、note の stale 表現を修正した。
- 2026-09-06: 開発ループ正本への例外反映についてユーザー判断を得て、`doc/guidelines/development-loop.md` の 4 箇所を修正した。
