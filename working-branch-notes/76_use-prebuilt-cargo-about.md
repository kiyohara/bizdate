# 作業ブランチメモ

- ブランチ: `use-prebuilt-cargo-about`
- PR: #76
- 最終更新: 2026-09-24

## 目的

Issue #65。Release workflow の build job で cargo-about 0.9.2 をソースからビルドするのをやめ、upstream の prebuilt を repository に固定した sha256 と照合して入れる。あわせて、PR の run で Release workflow の `custom-ci` が直接の CI と同じ `platform` job を回す重複をやめる。tag push では `ci.yml` の全 job を従来どおり回す。

## 現在の状況

- 依存の #64（PR #67）は merge 済み。着手時点で open PR は無い。
- ローカル（macOS host の Docker Desktop）で作業している。`drive-issue-to-reviewed-pr` の P1 として進めている。
- PR #76 を作成し、note を採番した。PR CI（head `dbcf7b9` と `192844f`）で CI と Release workflow はすべて success となり、`custom-ci` の `platform` は skipped になった。所要時間を PR #67 の初回 push と比べた（「検証」の「PR CI」）。
- review cycle `claude-code-77382cf-20260924000454`（head `77382cf`）で指摘 1 件（imo 1）を受け、採用して対応した（P4、`214120e`）。再確認（P5、head `214120e`）で thread は resolve 可と判定され、1 周で収束した。残るのは人間による thread の resolve と merge の判断である。
- ユーザーの補足（2026-09-24）: 並列実行の試行中で、#68 / #70 / #63 を cloud session が同時に進めている。直列の確認は今回に限り例外として承認済みで、他の session のブランチと PR には触れない。`progress.md` は DIST-03b の行だけを更新する。decision log は 0022 への追記を優先し、新しいログが要る場合は 0026 を使う。review metadata の `Model` には実行環境で確認した識別子を書く。

## 調査結果

- cargo-about 0.9.2 の release（2026-08-18 公開）の asset は `aarch64-apple-darwin`、`aarch64-unknown-linux-musl`、`x86_64-unknown-linux-musl`、`aarch64-pc-windows-msvc`、`x86_64-pc-windows-msvc` の `.tar.gz` と、それぞれの `.sha256`。`x86_64-apple-darwin` は無い。
- 使う 3 asset を取得し、ローカルで計算した sha256、release の `.sha256`、GitHub API の asset の `digest` の 3 つが一致することを確かめた。`.sha256` は 64 byte の hex だけで、file 名と改行を含まない。
- archive の中身は `cargo-about-0.9.2-<target>/` の下に `cargo-about`、`README.md`、`LICENSE-APACHE`、`LICENSE-MIT`。

## 決定事項

- `install-cargo-about.sh` は `--root <dir>` だけを受け付け、`<dir>/bin/cargo-about` に置く。省略時は `cargo install` の既定と同じ `${CARGO_HOME:-$HOME/.cargo}`。Dockerfile は従来どおり `--root /usr/local` で呼ぶ。crate を取得しなくなったため、Dockerfile の registry の掃除（`rm -rf /usr/local/cargo/registry`）は外した。
- asset は `uname -sm` で選ぶ（`verify-release-archive.sh` と同じ対応）。`Darwin arm64` / `Linux x86_64` / `Linux aarch64` 以外は exit 1。sha256 は script に固定し、release の `.sha256` は使わない。照合は `sha256sum` を優先し、無ければ `shasum -a 256` を使う。取得は Dockerfile の dist と同じ `curl --proto '=https' --tlsv1.2 -fsSL`。
- `ci.yml` の `platform` に `if: ${{ !(github.event_name == 'pull_request' && inputs.plan) }}` を置く。省く場合（呼び出し側の event が `pull_request` で、dist から `plan` が渡された run）だけを列挙する。`lint` は条件を付けず、`custom-ci` を success で終わらせる。
- `.github/build-setup.yml` はコメントだけを変え、step の名前と中身は変えない。YAML のコメントは `release.yml` へ出ないため、`release.yml` の作り直しは要らない（`dist generate --check` で確かめた）。
- decision log は 0022 への追記とする（ユーザーの補足と Issue のとおり）。0022 の主題（release workflow の構成）のうち、cargo-about の導入方法と PR での CI の二重実行だけを変える更新であり、他の決定は有効なままである。
- review 対応で決めたこと: tag push での `platform` の実動確認を、初回公開の手順と確認項目を定める #40 の公開後チェックへ引き継ぐ（review cycle `claude-code-77382cf-20260924000454` の `[imo]` を採用）。`platform` が省かれても `custom-ci` は `lint` だけで success になり、`host` は公開へ進むため、確認しないと失敗として表に出ない。0022 の追記の「影響」に確認の方法と引き継ぎ先を書き、#40 の本文に 1 項目を足す。#65 のスコープ外の「リリース手順（#40）」には当たらないと判断した。手順そのものは書かず、#67 が #40 の本文を同期したのと同じく、前提の変化を引き継ぐだけであるため。

## 次にやること

- [x] `install-cargo-about.sh` を prebuilt 方式に改め、Dockerfile の呼び出しをそろえる
- [x] `ci.yml` の `platform` に、PR の `custom-ci` でだけ省く条件を置く
- [x] decision log（0022 追記、index）を更新する
- [x] `distribution.md` / guideline / `build-setup.yml` と `ci.yml` のコメント / Copilot 指示を改める
- [x] Compose で検証する（`release-tools` の作り直し、`dist generate --check`、表記の生成と照合、ソースビルドとの byte 比較、stub での失敗経路）
- [x] `progress.md` の DIST-03b の行を更新する（採番後に PR 欄へ #76 を反映した）
- [x] PR を作成し、note を採番する
- [x] PR CI の結果と所要時間を記録し、PR #67 の実測と比べる

## 後続 Issue の同期

Issue 本文を MCP で更新し、公開 API で読み戻して、意図した本文と byte 単位で一致することを確かめた。PR の差分には含まれない。

- #40: 「#36 の決定（同期）」の公開後チェックに、tag push の Release workflow で `custom-ci / test / build (<target>)` の 3 job が skipped でなく success で走ったことを足した。#65 で加えた項目であることと、確かめる理由（PR の run では省くため tag push でだけ確かめられ、省かれても `custom-ci` は success になって公開を止めない）も書いた。review 対応（P4）で行った。
- MCP の読み取り結果は `<...>` を除くなど本文を整形して返すため、更新前の本文は公開 API から取得した原文を使った（#40 の原文は `v<version>` を 2 箇所含む）。

## 検証

### Compose（ローカル。Docker Desktop、container は Linux aarch64、rustc 1.98）

| 項目 | 結果 |
|---|---|
| `docker compose build release-tools` | OK。cargo-about の step は 0.6 秒（`aarch64-unknown-linux-musl` を選び、sha256 一致、`cargo-about 0.9.2`）。image の cargo-about は静的リンク（`statically linked`、`not a dynamic executable`） |
| `dist generate --check && dist plan` | OK。`release.yml` の作り直しは不要。成果物は 3 archive（各 `bizdate` / `LICENSE` / `README.md` / `THIRD-PARTY-LICENSES.md`）、各 `.sha256`、`sha256.sum` |
| `THIRD-PARTY-LICENSES.md` の生成と `check-third-party-licenses.sh` | OK。`ok: 50 crates for aarch64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu` |
| prebuilt とソースビルドの出力の比較 | 同じ container で `cargo install cargo-about --version 0.9.2 --locked --features cli --root /tmp/about-src`（36 秒）を実行し、同じ引数で生成した。2 つの出力は 93,111 byte、sha256 `85cfb7f1…d22f` で一致し、`cmp` も差分なし |
| `check-action-pins.sh` | OK（`uses:` 27 件） |
| `sh -n install-cargo-about.sh` | OK |

`install-cargo-about.sh` の経路（dev service の container で、`uname` / `curl` を PATH 先頭の stub に置き換えて実行）:

| 入力 | 結果 |
|---|---|
| `uname -sm` が `Darwin x86_64` / `Linux riscv64` / `FreeBSD amd64` | exit 1。`no prebuilt cargo-about 0.9.2 is pinned for '<platform>'`。何も置かない |
| `curl` の stub が改変した file を書く（Linux aarch64） | exit 1。`sha256 of cargo-about-0.9.2-aarch64-unknown-linux-musl.tar.gz is <実値>, not the pinned af51…e5a8`。何も置かない |
| 同上で `uname -sm` が `Darwin arm64` | exit 1。取得先が `cargo-about-0.9.2-aarch64-apple-darwin.tar.gz` であり、固定値 `ae72…c3e` と照合して止まる（asset の選択の確認） |
| `curl` の stub が exit 22 | exit 22。何も置かない |
| `--root` の値なし / 未知の option | exit 2 と usage |
| stub なし、`--root <tmp>` | exit 0。`<tmp>/bin/cargo-about` に置かれ、`cargo-about 0.9.2` |
| stub なし、`--root` 省略、`CARGO_HOME=<tmp>` | exit 0。`<tmp>/bin/cargo-about` に置かれ、image の `/usr/local/cargo/bin` には書かない |
| PATH を `sha256sum` を含まない dir（`uname` / `mktemp` / `curl` / `cut` / `shasum` / `tar` / `gzip` / `mkdir` / `install` / `rm` の symlink だけ）に限る | exit 0。`shasum -a 256` で照合して成功（fallback の確認） |

- `git diff --check`: clean。
- 実物の macOS（`Darwin arm64`）での取得と実行は Compose では確かめられない。PR CI の `build-local-artifacts (aarch64-apple-darwin)`（`macos-15`）で確かめる。

### PR CI（PR #76）

所要時間は、PR #67 の note と同じく run の `run_started_at` から `updated_at` までで数えた。job と step の時間は job の API の時刻（秒単位）から数えた。

| 項目 | PR #67 の初回 push（head `ee5d285`） | 本 PR の初回 push（head `dbcf7b9`） |
|---|---|---|
| Release workflow | 3 分 37 秒（run 35873618982） | 1 分 44 秒（run 35936143855） |
| build `aarch64-apple-darwin`（`macos-15`） | 2 分 40 秒（うち cargo-about 109 秒） | 58 秒（うち cargo-about 1 秒） |
| build `aarch64-unknown-linux-gnu`（`ubuntu-22.04-arm`） | 2 分 17 秒（うち 93 秒） | 48 秒（うち 1 秒未満） |
| build `x86_64-unknown-linux-gnu`（`ubuntu-22.04`） | 2 分 48 秒（うち 109 秒） | 48 秒（うち 1 秒未満） |
| `custom-ci` の job | `lint` と `platform` 3 job | `lint`（19 秒）だけ。`platform` は skipped |
| push 1 回あたりの macOS job | 4 本（CI 1、Release workflow 3） | 3 本（CI 1、Release workflow 2） |
| 直接の CI（参考。本 PR は job を変えていない） | 52 秒（run 35873618462） | 31 秒（run 35936143642） |

- 採番の push（head `192844f`）でも同じ結果だった。Release workflow は 1 分 58 秒（run 35936207846）、build job は 1 分 3 秒 / 48 秒 / 53 秒、直接の CI は 36 秒（run 35936207687）。
- 両 head とも、CI（`fmt / clippy` と 3 target の `test / build`）と Release workflow（`plan`、3 target の build、`custom-ci / fmt / clippy`、`build-global-artifacts`、`custom-release-verify` の `release tag` と 3 target の `archive`）が success。`custom-ci / test / build (${{ matrix.target }})` は skipped で、matrix を展開する前に省かれた。`host` と `announce` は skipped。
- `custom-ci` の呼び出し側の job は check run に現れない。`custom-ci` を `needs` に持つ `build-global-artifacts` と `custom-release-verify` が走って success になったこと、run が success で終わったことから、`custom-ci` は success で終わったと判断した。
- 直接の CI では、3 target の `platform` がすべて回った（`inputs.plan` が空の run では省かれない）。
- `build-local-artifacts (aarch64-apple-darwin)` の log: `install-cargo-about: ok: sha256 ae72…c3e cargo-about-0.9.2-aarch64-apple-darwin.tar.gz` と `cargo-about 0.9.2` を出し、step は約 0.5 秒だった。実物の macOS（`macos-15`、macOS 15.7）で asset の選択、照合、実行が通った。
- 3 つの build job で、表記の生成と照合が通った（`ok: 50 crates for aarch64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu`）。archive の検証（`release-verify`）も 3 target で通った。
- 検証した archive はその run の workflow artifact であり、公開された Release asset ではない。

### 条件式の review（tag push の経路）

`platform` の `if: ${{ !(github.event_name == 'pull_request' && inputs.plan) }}` を run の種類ごとに読んだ。呼ばれた workflow の `github` context は呼び出し側のもので、`release-verify.yml` の `github.event_name == 'pull_request'` の分岐も同じ前提で PR #62 / #67 の run を通っている。

| run | `github.event_name` | `inputs.plan` | `platform` |
|---|---|---|---|
| PR の直接の CI | `pull_request` | 無し（空） | 回る |
| main への push の CI | `push` | 無し | 回る |
| Release workflow の `custom-ci`（PR） | `pull_request` | dist の plan（空でない JSON） | 省く |
| Release workflow の `custom-ci`（tag push） | `push` | dist の plan | 回る（`&&` の左辺が偽） |

- tag push で省かれるのは、event が `pull_request` になる場合だけであり、tag push の event は `push` である。後から trigger を足しても、`pull_request` でなければ回る。
- tag push の実動は PR では走らないため未検証（初回の公開で確かめる）。

## リスク・ブロッカー

- tag push で `custom-ci` の `platform` が回ることは、条件式の review だけで確かめている。実動は未検証で、初回公開の公開後チェックとして #40 へ引き継ぐ。
- prebuilt とソースビルドの出力の byte 比較は Linux aarch64（`aarch64-unknown-linux-musl`）でだけ行った。`x86_64-unknown-linux-musl` と `aarch64-apple-darwin` の prebuilt の出力は、PR CI の各 build job で `check-third-party-licenses.sh` を通すことで確かめる（byte 比較ではない）。
- `progress.md` の「次にやること」に「`run-issue-task` で #65 へ進む」の行が残る。ユーザーの指示（DIST-03b の行だけを更新する）に従って据え置いた。merge 後の整理（`maintain-progress` など）に残す。

## セッションログ

- 2026-09-24: Issue #65 を読み、依存（#64 / PR #67 merge 済み）と open PR 無しを確認。ブランチ `use-prebuilt-cargo-about` で着手。cargo-about 0.9.2 の asset と sha256 を確かめた。
- 2026-09-24: `install-cargo-about.sh`、Dockerfile、`ci.yml`、`build-setup.yml` のコメント、`distribution.md`、development-command guideline、Copilot 指示、decision log（0022 追記と index）、`progress.md` の DIST-03b を改めた。Compose で `release-tools` を作り直し、`dist generate --check` / `dist plan`、表記の生成と照合、ソースビルドとの byte 比較、stub での失敗経路を確かめた。
- 2026-09-24: PR #76 を作成した（head `dbcf7b9`）。`number-working-branch-note` で note を PR #76 で採番した（`192844f`）。PR description の note 参照を 1 箇所置換し、公開 API の読み戻しで意図した本文と byte 単位で一致することを確かめた。P1 の引き上げ項目: 完了として書き換えたタスク行は note の 1 件（「PR を作成し、note を採番する」）で、PR description には該当なし。触らなかった行は note の 1 件（「`progress.md` の DIST-03b の行を更新する（PR 欄は採番後）」。完了要素と未完要素（PR 欄の反映）が混在する行で、採番 skill は `progress.md` を更新しない）で、この後の P1 で PR 欄を反映し括弧書きを改めた。PR description の「PR CI の結果と所要時間は、完了後にこの節へ追記する」は定型外で採番 skill は触らず、P1 で CI の結果に置き換えた。採番 skill は停止せず完走した。
- 2026-09-24: PR CI（head `dbcf7b9` と `192844f`）の結果と所要時間を記録し、PR #67 の初回 push と比べた。0022 の追記の「影響」に実測を足した。`progress.md` の DIST-03b の PR 欄に #76 を反映した。
- 2026-09-24: P2 の review（cycle `claude-code-77382cf-20260924000454`、head `77382cf`）を subagent に委譲した。review 1 本と inline 1 件が投稿され、読み戻しで確かめた。指摘は imo 1。must / ask / nits / fyi は 0。install script、`platform` の条件、0022 の追記、文書の整合、Issue #65 の完了条件はいずれも妥当と判定された。review の `Model` は `claude-opus-5-5`（subagent の system prompt で確認）。
- 2026-09-24: P3 で指摘を確かめた。#40 の本文の公開前チェック・公開後チェックに、tag push の run で `custom-ci` の `platform` が回ったことを見る項目は無かった。`platform` が省かれても `custom-ci` は success になり、`host` の条件を満たすことも確かめた。採用と判断した。P4 で 0022 の追記の「影響」に確認の方法と引き継ぎ先を書き、#40 の本文の公開後チェックに 1 項目を足した（「後続 Issue の同期」）。文書だけの変更のため、Rust と Compose の検証は再実行していない。
- 2026-09-24: P4 で thread に処置を返信した（採用し修正した。`Model` は `claude-opus-5-5`。session の metadata と system prompt で確認）。読み戻しで返信が 1 件だけであることを確かめた。`214120e` の check runs 17 件はすべて完了し、期待どおりだった。
- 2026-09-24: P5 の再確認（P2 と同じ subagent、head `214120e`）で、thread は resolve 可と判定された（resolve 可マーカー付きの返信 1 件と完了要約 1 本を読み戻しで確かめた）。未対応 0、新規指摘 0。#40 の本文も公開 API の原文で確かめられた。P6 で追加対応は不要と判断した。
- 2026-09-24: P6 の後、PR の `mergeable_state` が `dirty` になった。main に #72 / #74 / #75 が入り、`doc/design/decision-log/index.md` が衝突したため、ユーザーの補足に従って main を PR head に merge した（`a36491a`。rebase と force push はしない）。この PR の 0022 の行と、main が足した 0023 の行を両方残した。`progress.md` は自動で merge され、main との差は DIST-03b の行だけである。
