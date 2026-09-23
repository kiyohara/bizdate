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

## リスク・ブロッカー

- 新しい cloud session で手作業の導入なしに `gh --version` が通ることは、merge と stub の貼り直しの後でしか確かめられない（未検証）。

## セッションログ

- 2026-09-23T23:42Z 着手（`drive-issue-to-reviewed-pr` #70）。
- 2026-09-23T23:43Z 検証用に session 内で `apt-get install -y --no-install-recommends gh` を実行した。今回は権限判定で拒否されず、約 3 秒で `gh 2.45.0` が入った（`apt-get update` なし）。
