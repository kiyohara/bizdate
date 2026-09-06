# 作業ブランチメモ

- ブランチ: `add-pr-handling-to-register-progress-issue`
- PR: #6
- 最終更新: 2026-09-06

## 目的

Issue #5 の対応。`register-progress-issue` skill に、`progress.md` 編集後の commit / PR の扱いを追記する。

skill は編集手順までしか定めておらず、`main` が保護されている以上必ず PR が要るのに、独立 PR にするのか・起点 Issue を作るのか・`Closes` を付けるのかが読み取れなかった。実装タスクの索引登録を行う直前にこの抜けが判明した。

## 現在の状況

skill の編集と検証を完了した。PR 作成待ち。

## 決定事項

- 追記内容は slapex の [PR #212](https://github.com/kiyohara/slapex/pull/212) の実運用に合わせる。同 PR は「起点 Issue は作らず、`Closes` 対象は無い」と明記し、専用ブランチの独立 PR として出している。
- 手順の末尾に新しい step を足すのではなく、**step 6 として PR 化を挿入し、既存の「終了時に確認する」を step 7 へ繰り下げる**。確認は PR 作成後に行うのが自然なため。
- 「参照する正本」に `git-operation-guidelines.md` と `pull-request-guidelines.md` を追加する。現状どちらも挙がっておらず、追記する手順がこの 2 つを参照するため。
- 既存の step 6（Issue へ確認コメントを残す）は、`progress.md` 編集より前に行う判断もあり得るが、現行の順序を変えない。本 Issue のスコープは「commit / PR の扱いの追記」であり、既存手順の並べ替えは含まない。

## 次にやること

- skill を編集する。
- 検証を実施する。
- PR を作成し、note を採番する。

## 検証

Issue #5 の検証項目 4 件をすべて実施した。

| 項目 | 結果 |
|---|---|
| `pull-request-guidelines.md`（merge は agent が行わない）との整合 | 一致。正本 L13 の「PR の merge は AI agent が行わない」に対し、skill の step 6 と「やらないこと」の 2 箇所で同趣旨を明記した |
| `number-working-branch-note` の commit / push step との表現整合 | 一致。両 skill とも「参照する正本」に `git-operation-guidelines.md` と `pull-request-guidelines.md` を挙げ、本文から同じ形で参照している |
| 既存 step 番号と「やらないこと」「終了報告」の整合 | step は 1〜8 で連続。既存の 6・7 を 7・8 へ繰り下げた。「やらないこと」に 2 項目、「終了報告」に PR URL を追加し、既存記述との矛盾は無い |
| 参照する正本の存在 | 8 件すべて存在（`AGENTS.md` / `development-loop.md` / `git-operation-guidelines.md` / `github-mcp-guidelines.md` / `issue-driven-task-execution.md` / `pull-request-guidelines.md` / `working-branch-notes-handling.md` / `progress.md`） |

`progress.md` は更新していない。Issue #5 は索引に載らない単発 Issue であり、skill の step 7（旧 step 7）の「単発 Issue が索引に無い場合は無理に登録しない」に従った。

## リスク・ブロッカー

- なし。

## セッションログ

- 2026-09-06: Issue #5 に着手。main から分岐し、本 note を作成した。
- 2026-09-06: skill に step 6（PR 化）を挿入し、既存 6・7 を繰り下げ。「参照する正本」に 3 本、「やらないこと」に 2 項目、「終了報告」に PR URL を追加した。検証 4 項目を実施し、すべて通過した。
