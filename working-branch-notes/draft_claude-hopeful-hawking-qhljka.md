# 作業ブランチメモ

- ブランチ: `claude/hopeful-hawking-qhljka`（cloud session が決めたブランチ。Issue の推奨名 `drop-intel-mac-target` へは切り替えない）
- PR: （採番前）
- 最終更新: 2026-09-23

## 目的

Issue #64。配布対象から Intel Mac（`x86_64-apple-darwin`）を外し、`aarch64-apple-darwin` / `aarch64-unknown-linux-gnu` / `x86_64-unknown-linux-gnu` の 3 target にする。PR ごとの CI の待ち時間と macOS の同時実行枠を減らすためである。対象変更の決定はユーザーが行った（2026-09-23）。本ブランチはその記録、設定、CI、文書、後続 Issue 本文の同期を行う。

## 現在の状況

- 依存の #38（PR #62）は merge 済み。open PR は無く、直列消化の前提を満たす。
- cloud session（Claude Code on the web）で作業している。GitHub 操作は組み込み tool、開発コマンドは Compose 経由。`drive-issue-to-reviewed-pr` の P1 として進めている。
- 実装と文書更新を終え、Compose（cloud session）で検証した。`release-tools` を要する検証は PR CI の結果で代える。PR の作成が残っている。

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
- [ ] PR を作成し、note を採番する
- [ ] 後続 Issue（#39 / #40 / #63）の本文を 3 target の前提へ同期する
- [ ] PR CI の結果と所要時間を記録し、PR #62 の最終 push と比べる

## 残した参照

`x86_64-apple-darwin`、`macos-15-intel`、`Intel`、「4 target」「4 対象」「4 つの」などを repository 全体で検索した（過去の working branch note を除く）。改めなかった箇所と理由:

- decision log 0016 の本文（候補、検討内容、決定、理由、影響、見直し条件）、0022 の本文と見直し条件、0015 の 2026-09-23 追記: 当時の記録として書き換えない。0016 には追記で置き換えを明記した。0022 の見直し条件（cargo-about の prebuilt）は #65 で更新される。
- `progress.md` の現況の「続く #37 で配布対象 4 target の native CI …」と、DIST-02 の行の「4 target の native `platform` job」: #37 の完了時点の記録であり、当時の事実として残す。
- 0021 の「4 つの選択肢」: 無関係。
- 新たに書いた参照: `distribution.md` の対象外と案内、`dist-workspace.toml` のコメント、`.github/copilot-instructions.md`、0016 の追記。Intel Mac を外したことを明示するために置いた。

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
- `release.yml` の作り直し: 上記のため `dist generate` を実行していない。target ごとの記述が無いため差分は出ない見込みで、PR CI の `plan` job で一致を確かめる。
- macOS の実物での `platform-check.sh`: PR CI の `macos-15` の job（CI の `platform` と `release-verify`）で確かめる。

## リスク・ブロッカー

- cloud session では `release-tools` を使えないため、Issue の検証のうち `dist generate --check` / `dist plan` と third-party 表記の生成・照合は Compose で実行できない。PR CI の結果で代える。ローカルの Compose での実行は未実施として残る。

## セッションログ

- 2026-09-23: Issue #64 を読み、依存（#38 / PR #62 merge 済み）と open PR 無しを確認。cloud session のブランチ `claude/hopeful-hawking-qhljka` で着手。`release.yml` に target 固有の記述が無いこと、4 target の依存が同一であることを確かめた。
- 2026-09-23: 設定、CI、script、spec、decision log（0016 追記と index）、guideline、Copilot 指示、`progress.md` を改めた。Compose で fmt / clippy / test / release build / `platform-check.sh` / `check-action-pins.sh` を通し、macOS 分岐の変更を合成入力で確かめた。
