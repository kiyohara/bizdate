# 作業ブランチメモ

- ブランチ: `claude/hopeful-hawking-qhljka`（cloud session が決めたブランチ。Issue の推奨名 `drop-intel-mac-target` へは切り替えない）
- PR: #67
- 最終更新: 2026-09-23

## 目的

Issue #64。配布対象から Intel Mac（`x86_64-apple-darwin`）を外し、`aarch64-apple-darwin` / `aarch64-unknown-linux-gnu` / `x86_64-unknown-linux-gnu` の 3 target にする。PR ごとの CI の待ち時間と macOS の同時実行枠を減らすためである。対象変更の決定はユーザーが行った（2026-09-23）。本ブランチはその記録、設定、CI、文書、後続 Issue 本文の同期を行う。

## 現在の状況

- 依存の #38（PR #62）は merge 済み。open PR は無く、直列消化の前提を満たす。
- cloud session（Claude Code on the web）で作業している。GitHub 操作は組み込み tool、開発コマンドは Compose 経由。`drive-issue-to-reviewed-pr` の P1 として進めている。
- 実装と文書更新を終え、Compose（cloud session）で検証した。PR #67 を作成した。head `ee5d285` の PR CI（CI と Release workflow）はすべて成功し、`release-tools` を要する検証をその結果で代えた。所要時間と macOS job 数を PR #62 の最終 push と比べた。
- 後続 Issue（#39 / #40 / #63）の本文を 3 target の前提へ同期した。

## 調査結果

- `.github/workflows/release.yml` に target 固有の記述は無い。build の matrix と runner は `plan` job の `dist plan` が実行時に決める（`artifacts_matrix`）。`dist-workspace.toml` の `targets` と `github-custom-runners` から `x86_64-apple-darwin` を外しても、`dist generate` の出力は変わらない見込みである。
- `THIRD-PARTY-LICENSES.md` の crate 集合: Compose（cloud session、dev service）で `cargo tree --locked -e normal,build --target <t>`（bizdate を除く）を 4 target それぞれに実行した。4 target とも同じ 50 crate で、3 target の和集合も 4 target の和集合と同一だった。`x86_64-apple-darwin` だけが持つ依存は無く、表記の crate 集合は変わらない。
- `.github/scripts/` の Intel Mac 由来の分岐:
  - `verify-release-archive.sh`: target と `uname -sm` の対応表の `x86_64-apple-darwin` 行。
  - `platform-check.sh`: `codesign -dv` の失敗を許す `|| true`（x86_64 は linker が署名しないため）と、最低 macOS version の読み取りで `LC_VERSION_MIN_MACOSX` を見る分岐（deployment target 10.12 の x86_64 向け。Apple Silicon の deployment target は 11.0 以上で、linker は `LC_BUILD_VERSION` に書く）。

## 決定事項

- decision log は 0016 への追記とする。0016 の主題（初回配布の契約）のうち対象 target だけを変える更新であり、tzdb、TLS、archive、version / tag、署名などの決定は有効なままである。ログ全体を `superseded` にすると有効な決定まで上書き扱いになるため、0015 の 2026-09-23 追記と同じく既存ログへ追記し、index の 0016 の行を改める。
- 0022 は変更しない。0022 は対象 target を 0016 と `distribution.md` に委ねており（背景）、本文の「4 target」は当時の記録である。「後から見直す条件」の cargo-about の件（`x86_64-apple-darwin` の prebuilt が無い前提）は、#65 が 0022 を更新して扱う。
- `platform-check.sh` の Intel Mac 由来の許容を外す。署名が無い場合は失敗にし、最低 macOS version は `LC_BUILD_VERSION` だけから読む。Apple Silicon は署名の無い binary を実行せず、native link で linker が ad-hoc 署名を付けるためである（`distribution.md`）。
- cloud session では `release-tools`（dist / cargo-about）を使えない（`doc/guidelines/cloud-session-guidelines.md`）。`dist generate`、`dist generate --check` / `dist plan`、`THIRD-PARTY-LICENSES.md` の生成と照合は Compose では実行せず、PR CI の Release workflow の結果で確かめる。`plan` job は実行のたびに `release.yml` と設定の一致を検査する。ローカルでの実行はユーザーに残す。

## 次にやること

- [x] decision log（0016 追記、index）を更新する
- [x] `distribution.md` / `concept.md` を 3 target に改める
- [x] `dist-workspace.toml` / `ci.yml` / `about.toml` / `.github/scripts/` を改める
- [x] guideline / Copilot 指示 / `progress.md` を改める
- [x] 残った参照を検索し、改めない箇所の理由を記録する
- [x] Compose で検証する
- [x] PR を作成し、note を採番する
- [x] 後続 Issue（#39 / #40 / #63）の本文を 3 target の前提へ同期する
- [x] PR CI の結果と所要時間を記録し、PR #62 の最終 push と比べる

## 残した参照

`x86_64-apple-darwin`、`macos-15-intel`、`Intel`、「4 target」「4 対象」「4 つの」などを repository 全体で検索した（過去の working branch note を除く）。改めなかった箇所と理由:

- decision log 0016 の本文（候補、検討内容、決定、理由、影響、見直し条件）、0022 の本文と見直し条件、0015 の 2026-09-23 追記: 当時の記録として書き換えない。0016 には追記で置き換えを明記した。0022 の見直し条件（cargo-about の prebuilt）は #65 で更新される。
- `progress.md` の現況の「続く #37 で配布対象 4 target の native CI …」と、DIST-02 の行の「4 target の native `platform` job」: #37 の完了時点の記録であり、当時の事実として残す。
- 0021 の「4 つの選択肢」: 無関係。
- 新たに書いた参照: `distribution.md` の対象外と案内、`dist-workspace.toml` のコメント、`.github/copilot-instructions.md`、0016 の追記。Intel Mac を外したことを明示するために置いた。

## 後続 Issue の同期

Issue 本文を MCP で更新し、公開 API で読み戻して、意図した本文と byte 単位で一致することを確かめた。PR の差分には含まれない。

- #39: 「#36 の決定（同期）」の Formula の分岐を 3 対象にし、macOS 向けの archive は Apple Silicon だけであることを足した。検証の asset 選択を 3 対象（macOS は Apple Silicon の archive だけを選ぶこと）にした。依存の節の「#64 で同期する」を、同期した旨に改めた。
- #40: README の案内の項目に「macOS は Apple Silicon のみ対応」と記載し、Intel Mac での手動ビルドを案内しないことを足した。「#36 の決定（同期）」に 3 target へ改めた旨を足し、公開前後のチェックを 3 対象にした。
- #63: Cargo の更新 PR のレビュー観点の「CI（4 target）」を「CI（3 target）」にした。

## 検証

### Compose（cloud session で実行。dev service、container は Linux x86_64、rustc 1.98）

| 項目 | 結果 |
|---|---|
| `cargo fmt --check` | OK |
| `cargo clippy --locked --all-targets -- -D warnings` | OK |
| `cargo test --locked` | unit 69 件、子プロセス再入 2 件、CLI E2E 12 件。すべて成功 |
| `cargo build --locked --release` と `platform-check.sh` | OK。29 項目。最低 glibc は `GLIBC_2.34`。`Cargo.lock` は変わらない |
| `check-action-pins.sh` | OK（`uses:` 27 件） |
| 依存の集合（`cargo tree --locked -e normal,build --target <t>`） | 4 target とも 50 crate で同一。3 target の和集合は 4 target の和集合と一致 |
| `dist-workspace.toml` / `about.toml` の parse（container の python3 `tomllib`） | dist の `targets`、`github-custom-runners` の key（`global` を除く）、about の `targets` が同じ 3 target。`check-third-party-licenses.sh` と同じ `sed` で 3 target を読める |
| `platform-check.sh` の macOS 分岐（合成入力） | script から抜き出した awk は `LC_BUILD_VERSION` の `minos 11.0` を読み、`LC_VERSION_MIN_MACOSX` だけの入力では空を返す（script は失敗する）。署名の判定は stub の `codesign` で、署名ありは exit 0 で記録を出し、署名なしは `FAIL` で exit 1 |

- `.github/workflows/ci.yml` を YAML として読み、`platform` の matrix が 3 target であることを確かめた。Compose の image に PyYAML が無いため、sandbox の python3 で読んだ参考値である。
- `git diff --check`: clean。

### 未実施（cloud session のため）

- `release-tools` を要する検証: `dist generate` / `dist generate --check` / `dist plan`、`THIRD-PARTY-LICENSES.md` の生成と `check-third-party-licenses.sh`、archive の作成と `verify-release-archive.sh`。cloud session では `release-tools` を使えない（`doc/guidelines/cloud-session-guidelines.md`）。PR CI の Release workflow の結果（`plan` の一致検査と成果物の一覧、各 build job での生成と照合、`release-verify`）で代える。ローカルの Compose での実行はユーザーに残す。
- `release.yml` の作り直し: 上記のため `dist generate` を実行していない。target ごとの記述が無く、PR CI の `plan` job の一致の検査が通った（下の「PR CI」）。
- macOS の実物での `platform-check.sh`: Compose では動かせない。PR CI の `macos-15` の job で確かめた（下の「PR CI」）。

### PR CI（PR #67、head `ee5d285`、2026-09-23）

所要時間は、公開 API の run の `run_started_at` から `updated_at` までで数えた。同じ数え方で PR #62 の最終 push も数え直した。

| 項目 | PR #62 の最終 push（head `3a505c3`） | 本 PR（head `ee5d285`） |
|---|---|---|
| Release workflow | 8 分 14 秒（[run 35852002755](https://github.com/kiyohara/bizdate/actions/runs/35852002755)） | 3 分 37 秒（[run 35873618982](https://github.com/kiyohara/bizdate/actions/runs/35873618982)） |
| CI | 1 分 8 秒（Issue の記載は 1 分 6 秒。[run 35852002437](https://github.com/kiyohara/bizdate/actions/runs/35852002437)） | 52 秒（[run 35873618462](https://github.com/kiyohara/bizdate/actions/runs/35873618462)） |
| push 1 回あたりの macOS job | 8 本（CI 2、Release workflow 6） | 4 本（CI 1、Release workflow 3） |
| 最も長い build job | `x86_64-apple-darwin`（`macos-15-intel`）6 分 59 秒 | `x86_64-unknown-linux-gnu`（`ubuntu-22.04`）2 分 48 秒 |

- CI: `fmt / clippy` と 3 target の `test / build` がすべて success。
- Release workflow: `plan`、3 target の `build-local-artifacts`、`custom-ci` の 4 job、`build-global-artifacts`、`custom-release-verify` の 4 job（`release tag` と 3 target の `archive`）がすべて success。`host` と `announce` は skipped で、Release は作られていない。
- `plan` job の `dist plan`: 成果物は 3 archive（各 `bizdate` / `LICENSE` / `README.md` / `THIRD-PARTY-LICENSES.md`）、各 `.sha256`、`sha256.sum`。build の matrix は `macos-15` / `ubuntu-22.04-arm` / `ubuntu-22.04` の 3 つ。`release.yml` と設定の一致の検査も通り、`release.yml` の作り直しが要らないことを確かめた。
- third-party 表記: 3 つの build job で生成と照合が通った（`ok: 50 crates for aarch64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu`）。4 target のときと同じ 50 crate である。
- `platform-check.sh` の厳格化: CI の `test / build (aarch64-apple-darwin)` と `custom-release-verify / archive (aarch64-apple-darwin)` の両方で通った。`codesign -dv` は `flags=0x20002(adhoc,linker-signed)` と `Signature=adhoc` を記録し、最低 macOS version は `LC_BUILD_VERSION` から `11.0` を読んだ。
- build job の cargo-about の `cargo install` は、`macos-15` 109 秒、`ubuntu-22.04` 109 秒、`ubuntu-22.04-arm` 93 秒だった（#65 の比較の基準）。

| target | runner | build | 検証 | 最低 glibc / macOS | 動的リンク先 | 署名 |
|---|---|---|---|---|---|---|
| `aarch64-apple-darwin` | `macos-15` | 2m40s | 0m12s | macOS 11.0 | `libSystem.B.dylib`、`libiconv.2.dylib` | ad-hoc（`linker-signed`） |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` | 2m17s | 0m11s | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0`） | `libc.so.6`、`libgcc_s.so.1` | - |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | 2m48s | 0m07s | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0`） | `libc.so.6`、`libgcc_s.so.1` | - |

- 3 target とも、checksum の照合、archive 名と構成の plan との一致、`README.md` / `LICENSE` の同一性、third-party 表記の照合、`platform-check.sh` の各項目が通った。最低 glibc と動的リンク先は PR #62 の値と同じである。
- 検証した archive はこの run の workflow artifact であり、公開された Release asset ではない。

## リスク・ブロッカー

- cloud session では `release-tools` を使えないため、Issue の検証のうち `dist generate --check` / `dist plan` と third-party 表記の生成・照合は Compose で実行していない。PR CI の結果（`plan` の一致の検査と成果物の一覧、各 build job での生成と照合）で代えた。ローカルの Compose での実行はユーザーに残す。

## セッションログ

- 2026-09-23: Issue #64 を読み、依存（#38 / PR #62 merge 済み）と open PR 無しを確認。cloud session のブランチ `claude/hopeful-hawking-qhljka` で着手。`release.yml` に target 固有の記述が無いこと、4 target の依存が同一であることを確かめた。
- 2026-09-23: 設定、CI、script、spec、decision log（0016 追記と index）、guideline、Copilot 指示、`progress.md` を改めた。Compose で fmt / clippy / test / release build / `platform-check.sh` / `check-action-pins.sh` を通し、macOS 分岐の変更を合成入力で確かめた。
- 2026-09-23: PR #67 を作成した。head `ee5d285` の CI（run 35873618462）と Release workflow（run 35873618982）がすべて success。所要時間、macOS job 数、archive の検証結果、cargo-about の install 時間を記録した。#39 / #40 / #63 の本文を同期し、読み戻しで一致を確かめた。
