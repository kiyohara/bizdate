# 作業ブランチメモ

- ブランチ: `claude/project-thread-0js1d6`
- PR: 未作成
- 最終更新: 2026-09-24

## 目的

Issue #80。tap（`kiyohara/homebrew-tap`）への書き込みに使う Actions secret の名前を、同じ tap へ書く slapex と同じ `HOMEBREW_TAP_GITHUB_TOKEN` に揃える。

## 現在の状況

- cloud session（Claude Code on the web）で、`drive-issue-to-reviewed-pr` の P1 として進めている。
- 発端: ユーザーから登録した secret の確認を頼まれ、workflow が読む名前（`HOMEBREW_TAP_TOKEN`）とユーザーが挙げた名前（`HOMEBREW_TAP_GITHUB_TOKEN`）が違うと分かった。続けて、slapex と名前を揃えたいとの指示があり、#80 を起こした。
- slapex の参照: 2026-09-24 に `kiyohara/slapex` を read-only の shallow clone で確認した（HEAD `08e4ba1`）。`.goreleaser.yaml` と `.github/workflows/release.yml` が `HOMEBREW_TAP_GITHUB_TOKEN` を参照する。
- cloud session からは secret の一覧を読めない（Actions secrets の API を proxy が 403 で拒否する）。ユーザーが実際に登録した名前は未確認。

## 決定事項

- 名前だけを変え、推奨する token の種類（tap だけに絞った fine-grained PAT）と登録の手順は変えない。
- decision log は新規ログを作らず、0025 に追記する。名前は 0025 の主題の一部であり、0025 の他の決定は有効なままのため。0016 の本文と merge 済みの note（42、79）は当時の記録として変えない。
- `publish-homebrew.yml` の冒頭コメントに、dist の builtin の job が読む `HOMEBREW_TAP_TOKEN` ではないことを残す。dist の文書と見比べた読み手が迷わないようにするため。
- #80 は単発 Issue として扱い、`progress.md` の索引へは登録しない（`run-issue-task` の「単発 Issue が索引に無い場合は、無理に `progress.md` へ登録しない」）。#40 より前に行うことは #80 の本文に書いた。

## 次にやること

- [x] publish job、`dist-workspace.toml` のコメント、`distribution.md` を新しい名前に揃える
- [x] 0025 に追記し、index を揃える
- [ ] PR を作成し、note を採番する
- [ ] #40 の本文の secret の記述を同期する
- [ ] review cycle を回す

## 検証

| 項目 | 結果 |
|---|---|
| 旧名 `HOMEBREW_TAP_TOKEN` の検索（`rg`） | 残るのは、0025 の追記より前の本文と追記内の旧名への言及、`publish-homebrew.yml` 冒頭の注記、0016 の本文、note 42 / 79 だけ |
| workflow の YAML parse | OK。`formula` job の 2 か所（未設定の確認と tap の checkout）が `secrets.HOMEBREW_TAP_GITHUB_TOKEN` を参照する。dev container に PyYAML が無いため、host の python3 で parse した |
| `check-action-pins.sh`（Compose の dev service） | OK（`uses:` 38 件） |
| `git diff --check` | OK |
| Rust の検証（`cargo fmt` / `clippy` / `test`） | 省略。Rust のコードと Cargo の設定を変えていないため。PR CI では回る |
| publish job の実動 | 未検証。tag push でしか走らないため、#40 の初回公開で確かめる |

## リスク・ブロッカー

- ユーザーが既に登録した secret の名前は、cloud session から確認できない。`HOMEBREW_TAP_TOKEN` で登録していた場合は、`HOMEBREW_TAP_GITHUB_TOKEN` で登録し直す必要がある（ユーザーが行う）。
- 新しい名前での publish job の実動は、tag push まで確かめられない。

## セッションログ

- 2026-09-24: ユーザーの確認依頼を受けて #80 を起こし、着手した。publish job、spec、0025 と index を新しい名前に揃え、上の検証を通した。
