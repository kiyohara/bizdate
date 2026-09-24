# 作業ブランチメモ

- ブランチ: `claude/clarify-model-identifier-rule`
- PR: #72
- 最終更新: 2026-09-23

## 目的

#68: model の識別子を記載してよい範囲と、review の canonical metadata の `Model` の扱いを明文化する。

## 現在の状況

- PR #72 を作成し、review cycle `claude-code-594a957-20260923235616` を完了した（指摘 1 件、採用、resolve 可マーカー付き）。thread の resolve と merge はユーザーが行う。

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
- `drive-issue-to-reviewed-pr` の P2 以降（review を subagent へ委譲）。（完了）
- ユーザーが inline thread を resolve し、PR を merge する。

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
- 2026-09-23T23:55Z: PR CI が green になった（最初の check run の開始 23:45:50Z から最後の完了 23:54:09Z まで約 8 分）。
- 2026-09-23T23:56Z〜23:58Z: P2。subagent が review cycle `claude-code-594a957-20260923235616` を作り、head `594a957` を review した。指摘 1 件（`[imo]` 1）。拒否された操作は無い。inline comment に environment が footer を二重に付けた（#69 のスコープ）。
- P3: `[imo]` 1 件を採用した。guideline の「PR title 以外では制限しない」は tool 名にも掛かり、0017（commit message は無規定、code / 文書は言及なし）からの明文化を 0023 が記録していなかった。0023 の「決定」と「影響」、index の行を直した。
- 2026-09-23T23:58Z: P4。修正を 60a7b46 で push し、inline thread に処置（採用し修正した）を返信した。read-back で返信 1 件のみを確認した。
- 2026-09-24T00:03Z: 修正後の head `0701b70` の CI が green（23:58:57Z から 00:02:40Z まで約 4 分）。
- 2026-09-24T00:04Z: P5。P2 と同じ subagent が head `0701b70` を再確認し、thread に resolve 可マーカー付きで返信し、完了要約を投稿した。追加の指摘は 0 件。拒否された操作は無い。
- P6: 追加対応は不要と判断し、review cycle を完了した（着手から約 22 分）。ユーザー確認で止まった箇所は無い。
- 評価実験の所見: 投稿の末尾に environment が attribution footer を自動で付けることがあり、canonical metadata が末尾にならない投稿や、`---` が二重になる投稿が出た（#69 のスコープ）。`github-op-integrated` MCP は接続できず、組み込みの GitHub tool で代替した。
- 2026-09-24T00:30Z: ユーザーの依頼で別 Agent（Codex）がクロスレビューした（review cycle `codex-fcf618d-20260924002407`、`[must]` 1 件）。指摘: `Model` の規則が、上位の指示がコメントへの記載まで実際に禁じる場合も上書きするように読める。採用し、guideline、skill、0023、index の行を「コメントが指示の対象外と確認できる場合は `unknown` にしない。コメントまで禁じる、または確認できない場合は従って `unknown` とし理由を残す」に揃えた。resume 時の SessionStart hook で Docker daemon が起動しなかったが、文書だけの変更で cargo の検証は元から対象外のため影響は無い。
