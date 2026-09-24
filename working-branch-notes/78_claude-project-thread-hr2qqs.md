# 作業ブランチメモ

- ブランチ: `claude/project-thread-hr2qqs`
- PR: #78
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

- skill、guideline、decision log の改訂。（完了）
- 検証、PR 作成、review cycle（本 PR 自身の cycle で新規定の読み戻しを確かめる）。
- merge 後、ユーザーが inline thread を resolve する。

## 検証

すべて cloud session で行った。

- footer が付かない環境（ローカルの Claude Code、Codex、Cursor）: 規定は「末尾。footer が付く場合はその直前」であり、footer が無ければこれまでと同じ位置になることを文面で確かめた。`Model` の確認手段の表は、これらを「上記以外」の行で扱う。
- skill、guideline、decision log の整合: `Model` の記載範囲（0023、`pull-request-guidelines.md`）と確認手段（0024、skill）が食い違わないこと、`gh api` での編集の範囲が `github-mcp-guidelines.md` の「`gh` で補う範囲」と一致することを読み合わせた。
- `git diff --check`: 問題なし。note の情報統制: 禁則の値なし（session ID などは書いていない）。
- cargo の検証: 文書と skill だけの変更のため実施しない。
- 新規定の実地確認（Issue の検証 1）: 本 PR の review cycle で確認した。review body（subagent、footer は server が付与）、address-comments の返信 7 件（session 本体、footer は自分で記載）、verify-comments の完了要約（subagent）のいずれも、5 行が末尾（footer の直前、空行 1 行を挟む）にあり、`Model` に確認した識別子 `claude-opus-5-5` が入ることを read-back で確かめた。

## リスク・ブロッカー

## セッションログ

- 2026-09-24T04:10Z 着手（`drive-issue-to-reviewed-pr` #69）。前の session（同じ Issue で起動）は着手前に中断されており、引き継ぐブランチや PR は無かった。
- subagent から見える model の情報を確認した。subagent の system prompt に exact model ID の行があり、`get_session` も呼べた（どちらも本 session の model と同じ値）。
- 2026-09-24T04:40Z PR #78 を draft で作成（head `9391c2f`）。レビュアーの指定は、PR の作成者と同じ account のため GitHub に拒否された（assignee は設定済み）。
- `number-working-branch-note` の手順で note を採番した（`draft_` → `78_`）。完了として書き換えたタスク行: 「次にやること」の「skill、guideline、decision log の改訂。」の 1 件（note 本文）。PR description は 0 件。触らなかった行: 「次にやること」の「検証、PR 作成、review cycle（…）」（完了要素と未完の review cycle が混在する複合行）。停止は無い。
- `progress.md` の OPS-03 を done にし、PR 欄を #78 に更新した（別 commit）。
- 2026-09-24T04:15Z PR CI（head `061bf5d`）が success 14 / skipped 3。
- 2026-09-24T04:17Z P2 を subagent に委譲。review cycle `claude-code-061bf5d-20260924041749`、指摘 7 件（imo 5 / nits 2）。04:21Z に完了。
- P2 の metadata の read-back（subagent の報告）: 5 行は review body の最後に連続し、subagent は footer を書かず、server が空行 1 行を挟んで footer を付けた。`Model` は subagent の system prompt の exact model ID（`claude-opus-5-5`）。新規定を満たした。
- P3: 7 件すべて採用。footer を書かない agent の書き方と確かめ方、`Model` は exact model ID、`get_session` は投稿の直前、`Agent` 行で確認できない誤りは人間に依頼、subagent は note を書かず報告を返す、編集の routing を `github-mcp-guidelines.md` の「操作別の第一選択」に追加、`cloud-session-guidelines.md` の参照先に「`Model` の確認手段」を追加。0024 も揃えた。
- 2026-09-24T04:22Z P4: 修正を push（head `da30cce`）し、7 thread へ処置を返信（MCP）。返信の metadata は session 本体として投稿直前の `get_session` の `last_served_model` を使い、footer は自分で書いた（空行 1 行を挟む）。read-back で 5 行と footer の関係を確かめた。
- 2026-09-24T04:25Z P5 を同じ subagent に委譲。7 thread すべて resolve 可。返信 7 件と完了要約の metadata が新規定を満たすことを read-back で確認（Issue の検証 1 を満たした）。追加は [nits] 1 件（「`gh` が使えない場合は編集しない」が、`gh` なしで編集できる cloud session の conversation comment と食い違う）と [fyi] 1 件（PR description が da30cce を反映していない）。
- P6: 2 件とも採用。nits は対象を `gh` を使う編集に絞って直し、PR description の「主な変更」を更新した。2 周目の P4 → P5 で確認する。
