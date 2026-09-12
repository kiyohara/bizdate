# 作業ブランチメモ

- ブランチ: refine-pr-and-note-numbering-rules
- PR: 未作成
- 最終更新: 2026-09-12

## 目的

類似構成の別プロジェクトで入った AI agent 向け rule / skill の改善 2 件を、bizdate の構成に合わせて取り入れる（Issue #43）。

1. PR の tool 名記載禁止を title のみに限定し、description と PR コメントの制限を外す。
2. `number-working-branch-note` skill に「本 skill の実行で完了するタスク行」の検出を追加する。

## 現在の状況

- `doc/guidelines/pull-request-guidelines.md` の「基本方針」「Tool 名の扱い」を改訂した。
- 入口 shim（`.claude/rules/`、`.cursor/rules/`）の要約行を正本に同期した。
- `.agents/skills/number-working-branch-note/SKILL.md` の Step 5 / Step 10 / 「stale 表現の定型置換」前書き / 「やらないこと」を改訂した。
- decision log 0017 を追加し、index を更新した。番号は origin/main で 0016 が配布契約に使われていたため 0017 とした。

## 決定事項

- 禁止は PR title のみとする。title はレビュアーが変更内容を把握するためのものであり、ガードレールはここだけで足りる。description と PR コメントは制限せず、`Co-Authored-By` や `🤖 Generated with ...` のような trailer を明示的に許可する。
- 旧ルールの例外条項（変更対象の実ファイルパスとして `.cursor/rules/` などは記載してよい）は削除する。禁止対象が title の tool 名・prefix に限定された結果、ファイルパスが抵触しないことは文面から自明である。
- bizdate の入口 shim は asahimaru と異なり禁止内容の要約行を持つため、同期が必要と判断した。行は増やさない。
- 採番 skill の完了タスク行は、機械置換の対象にせず候補提示とユーザー合意を経て書き換える。bizdate の skill は「定型のみ機械置換」という設計なので、判断を要する完了タスク行はその外側の手順として分けた。
- 書き換え形式は note の既存記法に合わせる（checkbox なら `- [x]`、そうでなければ削除または `（完了）` 付記）。どちらかに強制すると差分が不必要に大きくなる。
- tool 名記載範囲の変更は方針の上書きに当たるため、decision log に記録する（`decision-log-guidelines.md` の「記録が必要な場面」）。採番 skill の検出範囲拡張は skill 手順の改善であり、decision log の対象としない。
- 既存 note に残る完了タスク行（#1 / #2 / #3 / #4 / #16 / #33 / #35）は本 PR で修正しない。`working-branch-notes-handling.md` のメンテコスト判断に従い据え置く。

## 次にやること

- レビュー指摘への対応。merge はユーザーが行う。

## 検証

- `git grep "tool 名"` で PR description を禁止対象とする記述が残っていないことを確認する。MCP server の tool 名を指す別文脈は対象外。
- SKILL.md 内で、機械置換してよい範囲とユーザー合意が必要な範囲が矛盾なく読めることを確認する。
- `git diff --check` を実行する。
- 文書のみの変更であり、Rust の実装には影響しない。

## リスク・ブロッカー

- 検出対象を広げた分、採番 skill 実行時に提示する置換候補が増える。誤検出はユーザーが却下する運用で吸収する。
- description の体裁は裁量任せになるため、書き方のばらつきは許容する。実運用で問題が出た時点で改めて検討する。

## セッションログ

- 2026-09-12: 参照元 PR 2 件の差分を確認し、bizdate 側の該当ファイルと突き合わせて取り込み範囲を決めた。Issue #43 を作成し、本ブランチで実装した。
