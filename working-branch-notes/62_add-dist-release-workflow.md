# 作業ブランチメモ

- ブランチ: `add-dist-release-workflow`
- PR: #62
- 最終更新: 2026-09-23

## 目的

Issue #38。`dist` 0.32.0 で配布成果物（4 target の `.tar.gz` と `.sha256`、`THIRD-PARTY-LICENSES.md` の同梱）を作る設定と GitHub Releases 向けの release workflow を導入する。PR では plan / build / archive 検証だけを行い、公開は `v<version>` tag の push でのみ動かす。あわせて 0015 の見直し条件に従い Dependabot の扱いを再判断する。

## 現在の状況

- 依存の #37（PR #61）は merge 済み。レビュー待ちの自分の PR は無く、直列消化の前提を満たす。
- ローカル環境（Docker Desktop、arm64）で作業している。
- 実装と文書更新を終え、Compose で検証した。PR #62 を作成し、note を採番した。head `5544cd5` の PR CI（CI と Release workflow）がすべて成功し、4 target の archive 検証の結果を記録した。次は review cycle（P2）。

## 調査結果

### dist 0.32.0（tag `v0.32.0` のソースと、Compose での実行）

- `dist init` は `dist-workspace.toml` を作り、`[profile.dist]`（`release` + `lto = "thin"`）を足す。`publish = false` のままでは `This workspace doesn't have anything for dist to Release!` になり、`[package.metadata.dist] dist = true` で配布対象に入る。
- `dist plan` / `dist build` / `dist host` は実行のたびに `dist generate --check` 相当の検査を行い、`release.yml` が設定と食い違えば止まる。`cargo-dist-version` と実行中の dist の version が違うと `dist generate` も止まる。
- release workflow のテンプレート: `plan-jobs` は `plan` と並行して走り、`build-local-artifacts` がそれを待つ。`host` の条件は `always()` のうえで build 系と `local-artifacts-jobs` / `global-artifacts-jobs` の結果（success か skipped）を見るが、`plan-jobs` の結果は見ない。`plan-jobs` が失敗すると build 系が skipped になり `host` の条件が成立する。`dist host` は GitHub hosting では成果物の有無を確かめない（`host.rs`）ため、archive の無い Release が作られうる。
- `local-artifacts-jobs` は `build-local-artifacts` と並行、`global-artifacts-jobs` は `build-local-artifacts` の後に走り、どちらの結果も `host` の条件に入る。custom job には `with: plan` が渡る（`plan-jobs` を除く）。
- `dist build` は `cargo build --profile dist --target <t>` を `--locked` なしで実行する。`MACOSX_DEPLOYMENT_TARGET` は設定しない。
- root の `permissions` は `contents: write` 固定。変えられるのは custom job の権限だけ。build job には `GH_TOKEN` が渡る。
- `github-action-commits` の値は `uses:` の行末へそのまま出る。`"<SHA> # <tag>"` と書くと末尾 tag コメントになる。`github-build-setup` の値は引用符付きで出るため、tag コメントを含めると不正な参照になる（生成して確認）。
- 既定の runner は `aarch64-apple-darwin` だけ `macos-14`。他の 3 target と global は配布仕様の runner と同じ。`macos-15-intel` と `ubuntu-22.04-arm` は dist の既知 runner 表にある。
- 既定で `source.tar.gz` と `sha256.sum` を作る。`sha256.sum` は checksum を有効にしている限り外せない。
- tag: `dist plan --tag=v0.1.0` は成功、`--tag=v0.2.0` は exit 255（`This workspace doesn't have anything for dist to Release!`）、`--tag=0.1.0` と `--tag=bizdate-v0.1.0` は成功した。

### cargo-about 0.9.2

- 未許可ライセンスがあると error を数えて非 0 で終了する。`--fail` はライセンス式を読めない crate も error にする。
- online と `--offline` の生成結果は同一だった（ライセンスファイルを欠く crate は無い）。
- prebuilt は `aarch64-apple-darwin` / `aarch64-unknown-linux-musl` / `x86_64-unknown-linux-musl` / Windows のみで、`x86_64-apple-darwin` は無い。`cargo install --locked --features cli` は Compose（14 CPU）で 35 秒。
- 生成した表の crate（50 件）は、4 target それぞれの `cargo tree --locked -e normal,build --target <t>` の和集合から bizdate 自身を除いたものと一致した。`--target all` では 73 件になり、target の絞り込みが効いている。
- handlebars の `{{...}}` は HTML escape される（`&quot;` が出た）。書式は `{{{...}}}` に揃えた。

### 実行環境

- cloud session の hook は `compose config --images | head -n 1` の image を `compose build dev` で build し、base image は Dockerfile の最初の `FROM` から組む。`release-tools` service に profile を付けると `config --images` に出ず、`AS dev` を足しても最初の `FROM` の解析は変わらないことを確かめた。
- `rust:1.98-trixie` の dev image には `python3` がある（`jq` は無い）。

## 決定事項

構成の採否理由は `doc/design/decision-log/0022-release-workflow.md`、Dependabot の再判断は 0015 の 2026-09-23 追記に置いた。要点:

- dist の設定は `dist-workspace.toml`、`dist = true` だけ `Cargo.toml` の `[package.metadata.dist]`。`[profile.dist]` は `release` を継ぐだけ（LTO を足さない）。
- 公開を止める検査は `plan-jobs` に置かない。CI の再実行は `local-artifacts-jobs`（`ci.yml` に `workflow_call` を追加）、tag の形と archive の検証は `global-artifacts-jobs`（`release-verify.yml`）。
- third-party 表記は各 build job で生成し、`check-third-party-licenses.sh` で依存と照合してから `dist build` する（`.github/build-setup.yml`）。
- build の直前に toolchain を 1.98 にし、`cargo fetch --locked` で lock の一致を確かめる。
- `pr-run-mode = "upload"`、`cache-builds = false`、`source-tarball = false`。
- action は `github-action-commits` で SHA 固定。CI の `lint` job に `check-action-pins.sh` を追加。
- ローカルは `Dockerfile` の `release-tools` stage と Compose の `release-tools` service（profile 付き）。dist は公開 binary を checksum 照合で、cargo-about は build job と同じ `install-cargo-about.sh` で入れる。
- Dependabot: Cargo の version updates、Dependabot alerts、security updates をいずれも採用。実装は別 Issue（Issue の起票は計画担当の役割のため本 PR では起票しない）。alerts と security updates の有効化はユーザーが行う。`release.yml` は Dependabot の対象に残し、差分が出たら `github-action-commits` を揃える。

## 次にやること

- [x] `dist` / `cargo-about` をローカルで実行できる環境を用意する
- [x] `dist` の設定と release workflow を生成する
- [x] third-party 表記の生成設定と、archive の検証 job を作る
- [x] 0015 の再判断を記録する
- [x] guideline / spec / Copilot 指示 / progress.md を更新する
- [x] Compose で検証し、結果を記録する
- [x] PR を作成し、note を採番する
- [x] PR CI（CI と Release workflow）の結果を記録する

## 検証

### Compose（ローカル、Docker Desktop の Linux arm64 container。dev image は rustc 1.98.0）

| 項目 | 結果 |
|---|---|
| `cargo fmt --check` | OK |
| `cargo clippy --locked --all-targets -- -D warnings` | OK |
| `cargo test --locked` | unit 69 件、子プロセス再入 2 件、CLI E2E 12 件、doc-test 0 件。すべて成功 |
| `cargo build --locked --release` | OK。`Cargo.lock` は変わらない |
| `docker compose build release-tools` | dist 0.32.0（公開 checksum と照合）、cargo-about 0.9.2（`install-cargo-about.sh`）が入る |
| `dist generate --check` / `dist plan` | OK。4 archive（bin + `LICENSE` / `README.md` / `THIRD-PARTY-LICENSES.md`）、各 `.sha256`、`sha256.sum` を列挙 |
| third-party 表記の生成と照合 | OK。50 crate が 4 target の依存の和集合と一致。`webpki-roots` の CDLA-Permissive-2.0、`encoding_rs` の BSD-3-Clause、`ring` の ISC / Apache-2.0 の本文あり |
| `aarch64-unknown-linux-gnu` の archive の作成と `verify-release-archive.sh` | OK。checksum、構成（plan と一致）、`README.md` / `LICENSE` の同一性、third-party 表記の照合、`platform-check.sh` の 29 項目、最低 glibc `GLIBC_2.34`（GCC symbol は `GCC_4.2.0` が最大）、動的リンク先 `libc.so.6` / `libgcc_s.so.1` |
| guideline に書いたコマンド | 書いた形のまま実行して成功 |

副作用のない異常系の確認（いずれも期待どおり exit 1、または dist が止まる）:

- `check-release-tag.sh`: `v0.1.0` だけ成功。`v0.2.0`、`0.1.0`、`bizdate-v0.1.0`、`v0.1.0-rc.1` は失敗。
- `check-action-pins.sh`: 一時的に置いた `actions/checkout@v7` の workflow で失敗（確認後に削除）。現状の 27 件の `uses:` は成功。
- `check-third-party-licenses.sh`: 表の行の欠落、CDLA 本文の欠落、`subtle` の本文の欠落、空ファイル、`ring` の ISC 本文の付け替え、`encoding_rs` の BSD-3-Clause 本文の付け替えで失敗。
- `verify-release-archive.sh`: checksum の改ざん、runner と target の不一致、plan との構成不一致、`README.md` の差し替え（checksum は作り直し）で失敗。

### workflow

- `ci.yml` / `release.yml` / `release-verify.yml` / `build-setup.yml` / `dependabot.yml` を Ruby の YAML parser で読み、job の `needs` と `uses:` の参照先を確認した。`release-verify.yml` の `run:` に `: ` を含む plain scalar があり parse に失敗したため、block scalar に直した。
- `release.yml` の `uses:` はすべて 40 桁 SHA（tag コメントは YAML の comment として落ちる）。
- `host` の条件に `custom-ci` と `custom-release-verify` の結果が入り、build 系は `plan` だけを待つことを生成物で確認した。
- `git diff --check`: clean。
- `.github/copilot-instructions.md` の同期要否を `agent-configuration-management.md` に従って確認し、CI と release workflow の記述、`release.yml` の手編集の扱いを更新した。
- note の情報統制: 秘密情報、個人情報、認証情報付き URL、ローカル絶対 path は含まない。

### PR CI（PR #62、head `5544cd5`、2026-09-23）

- CI（[run 35842593579](https://github.com/kiyohara/bizdate/actions/runs/35842593579)）: 5 job すべて success。`fmt / clippy` の job で action の SHA 固定の検査も通った。
- Release workflow（[run 35842593797](https://github.com/kiyohara/bizdate/actions/runs/35842593797)）: `plan`（`release.yml` と設定の一致の検査を含む）、`build-local-artifacts` 4 job、`custom-ci` 5 job、`build-global-artifacts`、`custom-release-verify` 5 job がすべて success。`host` と `announce` は skipped で、PR では Release を作らないことを確かめた。
- 検証した archive はこの run の workflow artifact であり、公開された Release asset ではない。

| target | runner | build | 検証 | 最低 glibc | 動的リンク先 | 署名 |
|---|---|---|---|---|---|---|
| `aarch64-apple-darwin` | `macos-15` | 3m05s | 0m13s | 該当なし | `libSystem.B.dylib`、`libiconv.2.dylib` | ad-hoc（`linker-signed`） |
| `x86_64-apple-darwin` | `macos-15-intel` | 7m32s | 0m31s | 該当なし | `libSystem.B.dylib`、`libiconv.2.dylib` | 未署名 |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` | 2m17s | 0m08s | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0`） | `libc.so.6`、`libgcc_s.so.1` | - |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | 1m48s | 0m07s | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0`） | `libc.so.6`、`libgcc_s.so.1` | - |

- 4 target とも、checksum の照合、archive 名と構成の plan との一致（`bizdate` / `LICENSE` / `README.md` / `THIRD-PARTY-LICENSES.md`）、`README.md` / `LICENSE` の同一性、third-party 表記の照合（50 crate）、`platform-check.sh` の 29 項目が通った。`release tag` job は PR のため tag の検査を行わず、公開しない旨を出して成功した。
- 最低 glibc、動的リンク先、署名の種別は、#37 の CI が `--release` のバイナリで実測した値（PR #61 の note）と同じだった。
- build job の所要時間の大半は cargo-about の `cargo install` だった（`macos-15-intel` 333 秒、`macos-15` 126 秒、`ubuntu-22.04-arm` 92 秒、`ubuntu-22.04` 68 秒）。生成は 3〜11 秒、照合は 2 秒以内、`dist build` は 20〜68 秒。

### 未検証

- tag push での公開経路（`host` / `announce`）。tag の push と Release の作成はスコープ外で、公開は #40 の手順に従う。
- cloud session での `release-tools` の build（GitHub の release asset への到達を含む）。

## リスク・ブロッカー

- dist が生成する job は `contents: write` を継ぎ、build job に `GH_TOKEN` が渡る。dist の制約として 0022 に記録した。
- PR ごとに Release workflow が 4 target の build と検証、CI の再実行を行うため、PR の CI 時間が増える。head `5544cd5` では Release workflow が約 9 分（うち `macos-15-intel` の cargo-about の install が約 5.5 分）、CI 単体は約 1.5 分だった。
- `Dockerfile` と `compose.yaml` の変更により、cloud session の environment cache は drift 警告が出る。stub の貼り直しはユーザーの操作。
- Dependabot の Cargo 運用の実装 Issue は未起票。

## セッションログ

- 2026-09-23: Issue #38 を読み、依存（#37 / PR #61 merge 済み）と open PR 無しを確認。ブランチ `add-dist-release-workflow` で着手。
- 2026-09-23: dist 0.32.0 と cargo-about 0.9.2 のソースを読み、Compose に `release-tools` を用意して `dist init` / `generate` / `plan` / `build` を試した。当初 `plan-jobs` に置いた CI と third-party 表記の生成が `host` の条件を素通りすると分かり、`local-artifacts-jobs` / `global-artifacts-jobs` と build job 内での生成へ組み替えた。
- 2026-09-23: 0022 を新設し、0015 に再判断を追記。guideline / spec / Copilot 指示 / progress.md を更新し、Compose で検証した。
- 2026-09-23: 最初の commit が 1Password の承認待ち（`failed to fill whole buffer`）で失敗し、guideline に従って中断した。ユーザーの指示で再実行して commit（`150b782`）。PR #62 を作成し、`number-working-branch-note` で note を採番（`5544cd5`）。`progress.md` の DIST-03 に PR 番号を反映。P1 の引き上げ項目: 完了として書き換えたタスク行は note の 1 件（「PR を作成し、note を採番する」）、PR description には該当なし。触らなかった stale 表現は 1 件（「現在の状況」の「PR の作成と、PR CI の結果の記録が残っている」。定型外の prose で完了要素と未完要素が混在するため採番 skill は触らず、この更新で書き換えた）。採番 skill は停止せず完走した。PR 本文は MCP の読み出しで `<...>` と引用符が sanitize されるため、置換は公開 API で読んだ本文に対して行い、読み戻しで一致を確かめた。
- 2026-09-23: head `5544cd5` の CI（run 35842593579）と Release workflow（run 35842593797）がすべて success。PR では `host` / `announce` が skipped。4 target の archive の最低 glibc、動的リンク先、署名、build の所要時間を検証欄に記録した（sha256 は情報統制の観点で note に書かない）。
