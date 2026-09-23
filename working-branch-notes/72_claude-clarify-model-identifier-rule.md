# 作業ブランチメモ

- ブランチ: `claude/clarify-model-identifier-rule`
- PR: #72
- 最終更新: 2026-09-23

## 目的

#68: model の識別子を記載してよい範囲と、review の canonical metadata の `Model` の扱いを明文化する。

## 現在の状況

- 正本、skill、入口 shim、decision log 0023、index、`progress.md`（OPS-01 の行だけ）を更新した。PR 作成前。

## 決定事項

- 禁止は PR title だけとし、tool 名と並べて model の識別子を加えた。PR description、PR と review のコメント、commit message、code と文書の本文は制限せず、書き手の裁量とする（0017 の考え方を広げた）。
- code と文書の本文は制限しない。困った事例が無く、先回りの禁止になるため（候補 B を採らない）。
- review metadata の `Model` は必須のキーなので、実行環境の既定の指示を理由に `unknown` にしない。任意の記載は harness の指示に従って省いてよいので、衝突は `Model` だけに限られる。
- 正本の置き場所: 記載範囲は `doc/guidelines/pull-request-guidelines.md`、`Model` のキー定義は `.agents/skills/review-pull-request/SKILL.md`。skill から guideline を参照する。
- decision log は 0017 への追記ではなく新しい 0023 にした。review metadata の扱いが 0017 の主題（PR の tool 名）の外にあるため。0017 は変更しない。
- 同期: `.claude/rules/` と `.cursor/rules/` の `pull-request-guidelines` は title の禁止内容を要約しているため同期した。`AGENTS.md`、`CLAUDE.md`、`.github/copilot-instructions.md`、`doc/guidelines/git-operation-guidelines.md` は変更しない（理由は 0023 の「影響」）。
- 本 note と repository の文書には model の識別子の実値を書かない。新しいルールでは書いてよいが、本 session の harness の既定の指示に従って省いた。review metadata の `Model` には、ユーザーの指示どおり session 情報で確認した値を書く。

## 次にやること

- PR を作成し、note を採番する。（完了）
- `drive-issue-to-reviewed-pr` の P2 以降（review を subagent へ委譲）。

## 検証

- `git diff --check`: 問題なし。
- 記述の整合: 正本（`pull-request-guidelines.md`）、skill（`review-pull-request` の `Model` の項）、入口 shim 2 件、0023、index の行を読み比べ、禁止の範囲（PR title のみ）、`Model` の扱い、参照先の節名が一致することを確認した。旧節名「Tool 名の扱い」への参照は 0017 の本文（当時の記録）にだけ残り、据え置く。
- decision log と index: 0023 の「決定」と index の 0023 行の結論が一致することを確認した。
- note の情報統制: `working-branch-notes-security.md` の確認項目（token、secret、cookie、session に続く実値、長いランダム文字列、認証情報付き URL）を grep で確認し、該当なし。
- cargo の検証: 文書だけの変更のため実施しない。

## リスク・ブロッカー

- 無い。#69 は本 Issue の決定に合わせて `Model` の確認手段を定める。

## セッションログ

- 2026-09-23T23:42Z: 着手。`drive-issue-to-reviewed-pr` の P1 として開始した。ユーザーが承認した並列実行の試行の一つ（#70 と #63 は別の cloud session、#65 はユーザーがローカル）であり、直列確認は今回に限り例外として承認済み。#68 は依存なし。
- 2026-09-23T23:45Z: PR #72 を作成した（着手から約 3 分）。`number-working-branch-note` で note を採番した。完了として書き換えたタスク行は note の「PR を作成し、note を採番する。」の 1 件（PR description は 0 件）。触らなかった stale 表現・タスク行は 0 件（「P2 以降」の行は未完のタスクであり stale ではない）。停止なし。`progress.md` の OPS-01 の PR 欄に #72 を入れた。ユーザー確認で止まった箇所は無い。権限判定や proxy で拒否された操作は無い（`github-op-integrated` MCP は session 開始時に接続できず、組み込みの GitHub tool を使った。cloud session の既定どおり）。
