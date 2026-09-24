# 作業ブランチメモ

- ブランチ: `claude/project-thread-hr2qqs`
- PR: 未作成
- 最終更新: 2026-09-24

## 目的

#69: cloud session でも review の canonical metadata（`Agent` / `Model` / `Review cycle` / `Reviewed head` / `Mode`）を規定どおりに書けるようにする。footer が付く環境での位置、`Model` の確認手段、投稿前の確認と誤りの訂正を `review-pull-request` skill に定め、委譲 brief、guideline、decision log を揃える。

## 現在の状況

- 依存の #68（PR #72）と #70（PR #74）はどちらも merge 済み（`progress.md` の OPS-01 / OPS-02 が done）。
- cloud session で作業する。ブランチは session が決めたものを使う。

## 決定事項

- 位置は「投稿の末尾。実行環境が署名行（footer）を付ける場合はその直前」とし、parse は位置ではなく 5 つのキーの並びで行う。1 投稿に 1 組とする。
- 5 行と footer の間は空行を 1 行置く。`---` の直前の行は Markdown で見出し（setext heading）になるためである。
- `Model` の確認手段: cloud session の本体は `get_session`（`session_id` 省略）の `external_metadata.last_served_model`（無ければ `session_context.model`）。subagent は自身の system prompt が示す識別子を使う。`get_session` が示すのは session の model であり、subagent 自身の model とは限らないためである。orchestrator は値を指示しない。
- 誤りは新しい投稿で訂正せず、その投稿を編集する。conversation comment は MCP（cloud session の `update_issue_comment`）、inline comment / 返信 / 提出済み review の本文は `gh api`（REST）。`gh` が無ければ人間に UI での編集を依頼する。
- decision log は 0024 を新設し、0023 に確認手段の決定先を追記する。

## 次にやること

- skill、guideline、decision log の改訂。
- 検証、PR 作成、review cycle（本 PR 自身の cycle で新規定の読み戻しを確かめる）。

## 検証

すべて cloud session で行った。

- footer が付かない環境（ローカルの Claude Code、Codex、Cursor）: 規定は「末尾。footer が付く場合はその直前」であり、footer が無ければこれまでと同じ位置になることを文面で確かめた。`Model` の確認手段の表は、これらを「上記以外」の行で扱う。
- skill、guideline、decision log の整合: `Model` の記載範囲（0023、`pull-request-guidelines.md`）と確認手段（0024、skill）が食い違わないこと、`gh api` での編集の範囲が `github-mcp-guidelines.md` の「`gh` で補う範囲」と一致することを読み合わせた。
- `git diff --check`: 問題なし。note の情報統制: 禁則の値なし（session ID などは書いていない）。
- cargo の検証: 文書と skill だけの変更のため実施しない。
- 新規定の実地確認（Issue の検証 1）: 本 PR の review cycle で行う（未実施）。

## リスク・ブロッカー

## セッションログ

- 2026-09-24T04:10Z 着手（`drive-issue-to-reviewed-pr` #69）。前の session（同じ Issue で起動）は着手前に中断されており、引き継ぐブランチや PR は無かった。
- subagent から見える model の情報を確認した。subagent の system prompt に exact model ID の行があり、`get_session` も呼べた（どちらも本 session の model と同じ値）。
