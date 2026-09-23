# 作業ブランチメモ

- ブランチ: `claude/enable-gh-in-cloud-session`
- PR: #74
- 最終更新: 2026-09-23

## 目的

#70: cloud session で `gh` を使えるようにし、GitHub MCP tool に無い操作を `gh api`（REST）で補う。`--provision` に `gh` の導入を足し、規約（`github-mcp-guidelines.md` / `github-cli-guidelines.md` / `cloud-session-guidelines.md`）と decision log 0018 を改める。

## 現在の状況

- 依存は無い（#70 の「依存」: なし）。
- 本 session はユーザーが承認した並列試行の 1 つ（#68、#63 は別の cloud session、#65 はユーザーがローカル）。直列消化の例外はユーザー承認済み。

## 決定事項

- decision log は新しく作らず、0018 への追記で扱う。組み込み GitHub tool を第一選択とする決定は変わらず、`gh` は MCP tool に無い操作の補完に限るため、方針の置き換えではなく拡張と判断した。
- `gh` の導入は `--provision` だけで行う。hook では導入せず、有無を 1 行で表示する。hook での導入は session 開始の遅延と、session ごとの挙動の差を生むため。
- network の許可リストへの host の追加は agent が決めない。判断材料を PR に整理し、ユーザーの判断を仰ぐ。decision log には推奨（追加しない）と、判断が PR review に委ねられていることを書く。

## 次にやること

- script と文書の変更、検証、PR 作成、review cycle。

## 検証

すべて cloud session で実行した。

- script: `bash -n` が通る。shellcheck は sandbox にも dev image にも無く未実施（CI も shell の lint を持たない）。
- `--doctor`: `gh` の行が出る。state の digest（旧）と repo の digest（新）がずれ、exit 1 と貼り直し用の stub を出す。変更で digest が変わり、hook が貼り直しを促すことを確認した。
- hook（`CLAUDE_CODE_REMOTE=true`）: `gh: 2.45.0 あり` と drift の警告を出し、exit 0。`CLAUDE_CODE_REMOTE` 未設定では無出力で exit 0。
- `--provision`: `gh` を外した状態で実行し、`gh` を導入して exit 0（約 4 秒）。2 回目は `gh: あり` で exit 0（冪等）。`apt-get` が失敗する fake を PATH に置いた状態でも、失敗を表示して exit 0 で完走した。確認後、state file は元の内容へ戻した。
- setup script の文脈（agent proxy が無い）での `apt-get` の到達は、この session では再現できず未検証。
- `gh api` の read: repository、PR、review comment の取得が通った。`gh auth status` は GraphQL の 403 で「token が無効」と誤表示した（#70 の実測どおり）。
- `git diff --check`: 問題なし。note の情報統制: 禁則の値なし。
- review 対応後（P4）: hook は `gh` の有無を daemon の処理より先に表示し exit 0。`gh` を外した状態の `--provision` で導入して exit 0、2 回目も exit 0。`CLAUDE_CODE_REMOTE` 未設定では無出力。state file は元に戻した。

## リスク・ブロッカー

- 新しい cloud session で手作業の導入なしに `gh --version` が通ることは、merge と stub の貼り直しの後でしか確かめられない（未検証）。

## セッションログ

- 2026-09-23T23:42Z 着手（`drive-issue-to-reviewed-pr` #70）。
- 2026-09-23T23:43Z 検証用に session 内で `apt-get install -y --no-install-recommends gh` を実行した。今回は権限判定で拒否されず、約 3 秒で `gh 2.45.0` が入った（`apt-get update` なし）。
- 2026-09-23T23:47Z PR #74 を作成（head `0d94607`）。PR 作成までの所要は着手から約 5 分。ユーザー確認で止まった箇所は無い。権限判定や proxy で拒否された操作は無い。
- 2026-09-23T23:48Z `number-working-branch-note` の手順で note を採番した（`draft_` → `74_`）。完了として書き換えたタスク行: 0 件。触らなかった行: 「次にやること」の「script と文書の変更、検証、PR 作成、review cycle。」（完了要素と未完の review cycle が混在する複合行）、PR description の「（採番後に rename します）」（定型置換の対象外）。後者は orchestrator が skill の外で削除した。停止は無い。
- `progress.md` の OPS-02 の PR 欄を #74 に更新した。
- 2026-09-23T23:55Z PR CI（head `8df1fc4`）が全件 success。最初の job の開始から最後の job の完了まで約 7 分（23:48〜23:55Z）。
- 2026-09-23T23:56Z P2 を subagent に委譲。review cycle `claude-code-8df1fc4-20260923235611`、指摘 7 件（must 2 / ask 1 / imo 3 / nits 1）。23:59Z に完了。
- P3: 7 件すべて採用。progress.md の行のセル崩れ（must）、`apt-get` の timeout 合計が setup script の 5 分上限に抵触し得る点（must、各 45 秒・最悪 135 秒に短縮）、host 追加の結論の記録（ask、「merge をもって採用」と明記）、github-cli の禁止操作の補足、cloud-session の表の残存表現、編集対象の判定基準、hook の `gh` 表示位置。
- 未収束（ユーザーへ報告）: 投稿の末尾に tool が footer を自動で付け、canonical metadata の 5 行が末尾にならない。本 PR では扱わず、#69 の判断事項として報告する。
