# 作業ブランチメモ

- ブランチ: `claude/issue-86-kfbe3o`
- PR: #89
- 最終更新: 2026-09-25

## 目的

Issue #86。dist 0.32.0 が作る checksum file（各 archive の `.sha256` と `sha256.sum`）は末尾に空行を 1 行含み、macOS の `shasum -a 256 -c` が `WARNING: 1 line is improperly formatted` を出す。次の version から空行の無い checksum file を公開し、公開前の検証（`custom-release-verify`）に再発を止める検査を足す。採った方法を decision log に残し、README の注記を外す時期と担当を決めて記録する。

## 現在の状況

- cloud session（Claude Code on the web）で、`drive-issue-to-reviewed-pr` で進めた。#82 と並行して進めた（ユーザーが一度限りの試行として承認。直列の確認は今回に限り外す。#82 のブランチと PR には触れない。decision log を新規に作る場合は 0027 を使う）。
- 依存: #84 は PR #87 の merge（2026-09-25）で閉じている。
- 実装と手元の検証を終えた。方法はユーザーが「公開前に整える」に決めた（2026-09-25）。
- review cycle `claude-code-763668e-20260925053303` は 2 周目で収束した。残るのは、人間による 2 thread の resolve、ready for review への切り替え、merge である。
- main は #82 の PR #88 の merge で進んだ。この branch との衝突は無く、main は merge していない。

## 決定事項

- dist の最新版を確かめた（2026-09-25）。crates.io の cargo-dist の最新は 0.32.0 のままで、上流の main の `write_checksum_file` も改行を足したままである。version を上げて直す方法は採れない。
- dist 0.32.0 の release workflow で、custom job が checksum file を整えられる位置を読んだ。
  - `global-artifacts-jobs`（`custom-release-verify`）は `build-global-artifacts` と並行して走る。`sha256.sum` はまだ無く、needs で待てない。
  - `host-jobs` は `host` と並行して走り、`host` が Release に上げる artifact を置き換えると競合する。
  - `github-release = "announce"` は Release の作成を publish job の後へ移し、tap の Formula が Release より先に書かれる。
  - `extra-artifacts` の build は `sha256.sum` の生成より前に走る。
  - 残る方法は、`custom-release-verify` の中で `build-global-artifacts` の artifact を待ち、末尾の空行を取り除いた checksum file で workflow artifact を置き換えてから `host` へ渡す形である。PR の run（`pr-run-mode = "upload"`）で毎回通る。待つために run の artifact の一覧を読む `actions: read` が要る。
- 方法の選択をユーザーに示し、ユーザーが推奨の「公開前に整える」を選んだ（2026-09-25）。
- 推奨の方法で実装した。`release-verify` に 3 job（`wait for build artifacts`、`normalize checksums (<artifact>)`、`checksum files`）と 3 script（`wait-for-build-artifacts.sh`、`normalize-checksum-files.sh`、`check-checksum-files.sh`）を足し、`release-verify` の権限に `actions: read` を足した。置き換えは、build の artifact を取得する `archive`、`homebrew-formula`、`homebrew` の後に置く。
- re-run で前の attempt の `artifacts-build-global` を拾わないよう、最新の `artifacts-plan-dist-manifest` より後に作られたものを待つ。同じ名前の artifact が複数あると、取得と置き換えは ID が最大のものを選び、ID は作成順とは限らない。待ち合わせの後は `host` の取得まで `artifacts-*` を上げる job が無く、`checksum files` が `host` と同じ集合を検査するため、置き換えが別の attempt の artifact を選んで整え損ねても `host` の前で止まる。例外として、`build-global-artifacts` が artifact を上げた後に失敗した run は再実行せず、version を上げる（release-guidelines の「復旧」）。
- prerelease では `homebrew-formula` と `homebrew` が skipped になり、`if` の無い後の job も skipped になる。追加した 3 job のすべてに状態関数を含む条件を置き、prerelease でも走らせる。
- README の注記は、空行の無い checksum file を初めて公開する version の公開後の PR で外す。その version の公開後確認で、公開した checksum file が `check-checksum-files.sh` を通った後とする。担当はその version の公開担当者。release-guidelines の「公開後の PR」と、公開後確認 Issue の雛形に項目を置いた。
- decision log は 0022 に追記した（0027 は使わない）。

## 次にやること

- [x] release-verify に、checksum file を整える job と検査を足す
- [x] `dist-workspace.toml` の権限を変え、`release.yml` を作り直す
- [x] 検査が空行を含む checksum file で失敗し、整えた file で通ることを Compose で確かめる
- [x] decision log 0022 に追記し、index と関連文書を揃える
- [x] README の注記を外す時期と担当を記録する
- [x] PR を作成し、note を採番する
- [x] PR の run で、追加した job が通ることを確かめる（`checksum files` の記録を読む）
- [x] review cycle を回す（2 周で収束）

## 検証

cloud session で実行した。script は Compose の dev service（Debian 13、x86_64。`sha256sum` は GNU coreutils 9.7、`shasum` 6.04、Python 3.13.5）で実行した。

| 項目 | 結果 |
|---|---|
| `v0.1.0` の公開済み checksum file | 9 件の asset を取得した。`.sha256` 3 件と `sha256.sum` は、いずれも改行 2 つで終わる。`shasum -a 256 -c sha256.sum` は `WARNING: 1 line is improperly formatted`（exit 0）。coreutils 9.7 の `sha256sum -c --strict` は警告を出さない（exit 0） |
| 検査が空行で失敗すること | 公開済みの file に対し、`check-checksum-files.sh` は `bizdate-aarch64-apple-darwin.tar.gz.sha256 has an empty line (line 2)` で失敗した（exit 1） |
| 整えた file が通ること | `normalize-checksum-files.sh` は 4 件を `normalized` とし、各 file は最後の 1 byte だけが減った（`.sha256` は 103→102 など、`sha256.sum` は 316→315）。`check-checksum-files.sh` は 4 件とも通り、`sha256sum -c --strict` と `shasum -a 256 -c --strict` は警告を出さない |
| 冪等性 | 2 回目の `normalize-checksum-files.sh` は 4 件とも `unchanged` |
| 検査が止める崩れ | 12 件すべてで失敗した（exit 1）: 空行 2 行（1 行だけ取り除いた後）、途中の空行、CRLF、末尾の改行なし、text mode（空白 2 つ）、大文字の 16 進、別の archive を記した `.sha256`、`sha256.sum` の行の欠落、行の重複、digest の不一致、checksum file の欠落、名前に path。checksum file の無い dir では `normalize-checksum-files.sh` が失敗した |
| 待ち合わせ（`wait-for-build-artifacts.sh`） | `gh` を差し替えた 6 件がいずれも期待どおり: 最初から上がっている、1 回待って上がる、re-run で前の attempt の `artifacts-build-global` しか無い（上限で失敗）、re-run 後に新しいものが上がる、一覧の取得が失敗し続ける（上限で失敗）、1 回失敗した後に取得できる。出力は `names=` の 1 行で、build の artifact 4 件の名前の JSON 配列 |
| `dist generate --check` | OK。dist 0.32.0 の公開 binary を Dockerfile の `release-tools` の stage に固定した sha256 と照合し、dev service に read-only で mount して実行した（PR #79 と同じ方法）。`release.yml` の差分は、`custom-release-verify` の権限の `"actions": "read"` の 1 行 |
| `check-action-pins.sh` | OK（44 件） |
| `sh -n` と実行権限 | 追加した 3 script で OK |
| YAML の parse | `release-verify.yml`、`release.yml` で OK |
| `git diff --check` | OK |
| PR の run | head `fa31805` の Release workflow と CI が success（`host` は PR のため skipped）。`wait for build artifacts` は plan の後に作られた `artifacts-build-global` を待たずに見つけ、build の artifact 4 件の名前を出した。`normalize checksums (<artifact>)` の 4 job は、それぞれ checksum file を `normalized` とし、元の artifact を消して上げ直した。`checksum files`（`ubuntu-22.04`）は、`host` と同じ形で取得した 5 件の artifact（置き換えた 4 件は上げ直したときと同じ digest）で 4 つの checksum file を検査し、`sha256sum -c --strict` と `shasum -a 256 -c --strict` が警告なしで通った |
| prerelease の経路 | `homebrew-formula` の条件を一時的に反転した commit（head `b113fc4`）の PR の run で、`homebrew formula` と `homebrew` が skipped、`wait for build artifacts` が success のとき、`normalize checksums (<artifact>)` と `checksum files` が skipped になり、run は success で終わった（修正前の再現）。修正後（head `19f1482`）は同じ条件で、`normalize checksums (<artifact>)` の 4 job が checksum file を `normalized` として元の artifact を置き換え、`checksum files` が 4 つの checksum file で `sha256sum -c --strict` と `shasum -a 256 -c --strict` を警告なしで通した。条件の反転は次の commit で戻した |
| coreutils 8.x の `sha256sum` | Docker の `ubuntu:22.04`（coreutils 8.32）で、末尾に空行がある checksum file に対し、`sha256sum -c` が `WARNING: 1 line is improperly formatted` を出して exit 0、`--strict` を付けると exit 1 になった。整えた file は、PR の run の `checksum files`（`ubuntu-22.04`）で `--strict` でも警告なしで通った |
| 公開した asset | この PR では確かめられない。次の version の公開後確認で `check-checksum-files.sh` に通す |
| Rust の検証（`cargo fmt` / `clippy` / `test`） | 省略。Rust のコードと Cargo の設定を変えていないため。PR CI では回る |
| 情報統制 | note、script、文書に、認証情報、個人情報、ローカルの絶対 path を書いていない |

## リスク・ブロッカー

- 別の job が上げた artifact の `overwrite` での置き換えと、`actions: read` での artifact の一覧の取得は、PR の run（head `fa31805`）で動くことを確かめた。置き換えた artifact を `host` が Release に上げる経路は、次の version の公開で初めて通る。
- `build-global-artifacts` が失敗した場合、`host` は走らない。artifact を上げる前の失敗では `wait for build artifacts` が上限（900 秒）まで待ってから失敗し、上げた後の失敗では待ち合わせを抜ける（その run は再実行しない。次の行）。
- `build-global-artifacts` が artifact を上げた後に失敗した run で、失敗した job だけを再実行すると、上げ直した `sha256.sum` が置き換えと検査を通らずに `host` に渡りうる。workflow では防げないため、release-guidelines の「復旧」で、その run では再実行せず version を上げるとした。

## セッションログ

- 2026-09-25: Issue と依存を確認。dist の最新版と workflow の差し込み位置を調べ、方法の選択をユーザーに示した。
- 2026-09-25: 推奨の方法で実装し、Compose で検証した。decision log 0022 に追記し、仕様、手順書、公開後確認 Issue の雛形、Copilot 用の指示を揃えた。
- 2026-09-25（P1）: PR #89 を draft で作成し、`number-working-branch-note` で note を採番した。完了として書き換えたタスク行は、note の「PR を作成し、note を採番する」の 1 件（PR description には該当なし）。触らなかった stale 表現・タスク行は 0 件。
- 2026-09-25: PR の run（head `fa31805`）が success。追加した job の記録を読み、置き換えと検査が動いたことを確かめた。coreutils 8.32 の空行の扱いを Docker の `ubuntu:22.04` で確かめた。
- 2026-09-25（P2）: review cycle `claude-code-763668e-20260925053303`（head `763668e`）で指摘 2 件（must 1、ask 1）。must は prerelease で置き換えと検査が skipped になる件、ask は re-run で前の attempt の artifact を選びうる件。
- 2026-09-25（P3/P4）: 2 件とも採用した。must は `homebrew-formula` の条件を一時的に反転した commit（`b113fc4`）で再現し、後の 2 job に状態関数を含む条件を置いた。ask は、取得と置き換えが選ぶ artifact の前提を改め、防げない場合の扱いを release-guidelines の「復旧」に置いた。
- 2026-09-25（P4）: 修正後の run（head `19f1482`）で、prerelease 相当でも 3 job が走って通ることを確かめ、条件の反転を戻した。
- 2026-09-25: ユーザーが方法の選択で「公開前に整える」を選んだ。
- 2026-09-25（P5）: 同じ cycle の verify-comments（head `02515d5`）で、2 件とも確認済み（resolve 可）。新規指摘は [nits] 1 件（decision log 0022 の「影響」の文書の列挙と、900 秒の待ちの条件）。
- 2026-09-25（P6/P4 の 2 周目）: [nits] を採用し、0022 の「影響」、note、PR description の同じ記述を直した。
- 2026-09-25（P5/P6 の 2 周目）: verify-comments（head `b04bc6e`）で [nits] の処置を確認済みとし、新規指摘は 0 件。cycle は 2 周目で収束した。
