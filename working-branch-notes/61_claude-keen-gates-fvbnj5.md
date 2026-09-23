# 作業ブランチメモ

- ブランチ: `claude/keen-gates-fvbnj5`（cloud session が決めたブランチ。Issue の推奨名 `add-platform-validation` へは切り替えない）
- PR: #61
- 最終更新: 2026-09-22

## 目的

Issue #37。配布対象 4 target（`aarch64-apple-darwin` / `x86_64-apple-darwin` / `aarch64-unknown-linux-gnu` / `x86_64-unknown-linux-gnu`）で CLI を native にビルド・テスト・起動確認する CI を整備し、最低 glibc と動的リンク先を実測して記録できるようにする。

## 現在の状況

- 依存の #36（PR #42）は merge 済み。open PR は無く、直列消化の前提を満たす。
- cloud session（Claude Code on the web）で作業している。GitHub 操作は組み込み tool、開発コマンドは Compose 経由。
- 実装と文書更新を終え、Compose で CI と同じコマンド列を通し、PR #61 を作成した。CI は 4 target とも success（検証欄に記録）。review cycle `claude-code-b0339ef-20260922122430` は指摘 3 件への対応と再確認を終え、3 thread とも resolve 可。残るのは人間による thread の resolve と merge 判断。

## 調査結果

### main の branch protection / ruleset（read-only、2026-09-22）

- ruleset `main protection`（id 22376200、enforcement `active`、対象 `~DEFAULT_BRANCH`）の rule は `pull_request`（必要 approve 数 0）/ `deletion` / `non_fast_forward` の 3 つ。**`required_status_checks` の rule は無い。** `GET /repos/kiyohara/bizdate/rules/branches/main` と `GET /repos/kiyohara/bizdate/rulesets/22376200` で確認した（HTTP 200）。
- classic branch protection は `GET /repos/kiyohara/bizdate/branches/main` の `protection` に `enabled: false`、`required_status_checks.enforcement_level: off`、`contexts: []` と出た。詳細 endpoint（`.../branches/main/protection`）は HTTP 403（`Resource not accessible by integration`）で読めなかったが、上記の要約表示で required status checks が無いことは確認できている。
- 帰結: job 名を `fmt / clippy / test / build` から `fmt / clippy` と `test / build (<target>)` × 4 へ分割しても、required status checks の更新は不要。settings は変更していない。

### runner 一覧（2026-09-22）

- docs.github.com は cloud session の egress proxy に遮断され（HTTP 403）、Issue に書かれた公式一覧 URL は開けなかった。代わりに公式一覧の元になる `actions/runner-images` の README（<https://raw.githubusercontent.com/actions/runner-images/main/README.md>）を取得して確認した。image 状態を示す badge の JSON（gist）も遮断されたため、deprecated 表示は README の本文で判断した。
- 採用 runner と architecture。すべて対象 architecture 上の native 実行で、emulation と cross-build は使わない。

| target | runner | architecture | README 上の状態 |
|---|---|---|---|
| `aarch64-apple-darwin` | `macos-15` | arm64 | 掲載あり、deprecated 表示なし |
| `x86_64-apple-darwin` | `macos-15-intel` | x64 | 掲載あり、deprecated 表示なし |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` | arm64 | 掲載あり、deprecated 表示なし |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | x64 | 掲載あり、deprecated 表示なし |

- 一覧には `ubuntu-26.04` / `macos-26` が既に載り、`macos-14` は deprecated。support policy は「GA image は最大 2 つ、最新 OS が GA になった時点で最古の deprecation を始める」であり、`ubuntu-22.04` は近く deprecation が始まり得る。`macos-15-intel` は 2027 年秋の提供終了が予告済み（#36 の調査）。CI の run が成功したことをもって、確認日時点で 4 label が利用可能であったことを実証する。

### jiff のタイムゾーン解決（`jiff` 0.2.35 のソースを Compose の registry で確認）

- `TZDIR` が指すディレクトリを開けた場合はそれだけを使い、既定ディレクトリへは fall back しない。1 zone だけ置いた `TZDIR` を渡せば、runner の tzdb を壊さずに「指定 zone のデータが無いと `unknown IANA time zone`」を再現できる。
- `TZ` が IANA 名としても TZif ファイルとしても解決できない場合、`TimeZone::try_system()` はエラーを返す（`/etc/localtime` へは fall back しない）。`TZ=Invalid/Zone` で「local timezone を解決できないと `cannot resolve local time zone`」を再現できる。

## 決定事項

- workflow は `lint`（`fmt / clippy`、`ubuntu-latest` で 1 回）と `platform`（4 target の matrix、`fail-fast: false`）の 2 job に分ける。fmt / clippy を matrix 展開しない。
- `platform` job は、runner の host triple が対象 target と一致することを先に確かめ（emulation / cross-build で通る構成を弾く）、`cargo test --locked --target`（unit + CLI E2E）、`cargo build --locked --release --target`、release バイナリに対する CLI E2E（`--release --test cli_e2e`。取得・更新・exit code を配布 profile で通す）、`.github/scripts/platform-check.sh` による起動確認と記録の順に回す。
- `platform-check.sh` は POSIX sh で書き、Compose でも同じ引数で実行できるようにする。確認するのは help / version、fixture を使った first / last / `--quiet` / `--day-off` / exit 0/1/2、タイムゾーンの採用経路（`--timezone` / `BIZDATE_TZ` / local、データ有無ごとの成否）、データ欠落。記録するのは Linux の最低 glibc（`objdump -T` の参照 GLIBC symbol version の最大値）と動的リンク先（`ldd`）、macOS の動的リンク先（`otool -L`）と署名種別（`codesign -dv`）。記録は step summary に出す。
- MSRV は workflow の `env.RUST_TOOLCHAIN` に 1 つだけ置き、2 つの job から参照する。第三者 action は既存の 3 つ（SHA 固定）のまま増やさない。`permissions` は `contents: read` のまま。
- Cargo.lock は `--locked` で固定する。unit / E2E に OS 依存の skip は入れない。

## 次にやること

- [x] CI workflow と `platform-check.sh` を実装する
- [x] guideline / Copilot 指示 / spec の前方参照 / progress.md を更新する
- [x] Compose で検証し、結果を記録する
- [x] PR を作成し、note を採番する
- [x] CI の 4 matrix job の結果（OS / architecture、テスト件数、head SHA、run URL、最低 glibc、動的リンク先）を記録する

## 検証

### Compose（cloud session で実行。dev container は `x86_64-unknown-linux-gnu`、rustc 1.98.1、Debian trixie / glibc 2.41）

CI の `platform` job と同じ順序・同じ option で実行した。

| 項目 | 結果 |
|---|---|
| host triple の確認（`rustc -vV` の `host` = target） | 一致 |
| `cargo fmt --check` | OK |
| `cargo clippy --locked --all-targets -- -D warnings` | OK |
| `cargo test --locked --target x86_64-unknown-linux-gnu` | unit 69 件、子プロセス再入 2 件、CLI E2E 12 件、doc-test 0 件。すべて成功 |
| `cargo build --locked --release --target x86_64-unknown-linux-gnu` | OK |
| `cargo test --locked --release --target x86_64-unknown-linux-gnu --test cli_e2e` | 12 件成功。release 成果物を共有し、追加ビルドは 1 秒未満 |
| `.github/scripts/platform-check.sh target/x86_64-unknown-linux-gnu/release/bizdate` | 29 項目すべて ok。最低 glibc `GLIBC_2.34`（ビルド環境の glibc 2.41 とは一致しない）、GCC symbol の最大は `GCC_4.2.0`、動的リンク先は `libc.so.6` と `libgcc_s.so.1`（ほかに `ld-linux-x86-64.so.2` と `linux-vdso.so.1`） |
| `--target` を付けない `cargo test --locked` | 同じ件数で成功（`--target` の有無で結果が変わらないことの確認） |

`platform-check.sh` の項目: help / version 5、fixture を使った first / last / `--quiet` / `--day-off` 8、exit 2（不正な日付、未知の option、データ欠落）4、system tzdb でのタイムゾーン採用経路 7（`--timezone` / `BIZDATE_TZ` / local / `TZ`、不正値 3 経路）、`Asia/Tokyo` だけを持つ `TZDIR` での採用経路 5（指定 zone があれば成功、無ければ `unknown IANA time zone`、`--version` は成功）。

### workflow と文書

- `.github/workflows/ci.yml` を PyYAML で parse し、job が `lint` / `platform` の 2 つ、matrix が 4 entry であることを確認した。
- 使用 action の入力名（`dtolnay/rust-toolchain` の `targets`、`Swatinem/rust-cache` の `key`）を、pin した SHA の `action.yml` で確認した。第三者 action は既存 3 つのまま、SHA も変えていない。
- `.github/copilot-instructions.md` の同期要否を `agent-configuration-management.md` に従って確認し、CI の記述と「配布 4 target の CI が無い」前提の 2 行を更新した。
- `doc/guidelines/development-command-guidelines.md` の「Compose の外に出るもの」を実装した job 名に揃え、MSRV 4 箇所の `ci.yml` 側の値名を `RUST_TOOLCHAIN` へ更新した。`doc/design/distribution.md` の「#37 で整備する CI」という前方参照を実体へ置き換えた。
- `git diff --check`: clean。
- note の情報統制: 秘密情報、個人情報、認証情報付き URL、ローカル絶対 path は含まない。

### CI（PR #61、head `82b2060`、[run 35725908817](https://github.com/kiyohara/bizdate/actions/runs/35725908817)、2026-09-22）

5 job すべて success。`fmt / clippy`（`ubuntu-latest`）は 0 分 17 秒（[job](https://github.com/kiyohara/bizdate/actions/runs/35725908817/job/106739640724)）。`platform` の 4 job はいずれも host triple の確認が通り（native 実行）、`cargo test --locked --target`（unit 69 件、子プロセス再入 2 件、CLI E2E 12 件、doc-test 0 件）、`cargo build --locked --release --target`、release バイナリでの CLI E2E 12 件、`platform-check.sh` の 29 項目がすべて成功した。macOS 2 job は cache 無し（`No cache found`）、Linux 2 job は直前の run（head `490cc5e`、push で cancel）が保存した cache を復元しており、所要時間の差はこれによる。

| target | runner | 実測した host / uname | 最低 glibc | 動的リンク先 | 署名 | 所要時間 |
|---|---|---|---|---|---|---|
| `aarch64-apple-darwin` | `macos-15`（image 20260907.0337.1、macOS 15.7.9） | host `aarch64-apple-darwin`、`Darwin arm64` | 該当なし | `libSystem.B.dylib`、`libiconv.2.dylib` | ad-hoc（`flags=0x20002(adhoc,linker-signed)`、`Signature=adhoc`） | 1 分 21 秒、[job](https://github.com/kiyohara/bizdate/actions/runs/35725908817/job/106739640837) |
| `x86_64-apple-darwin` | `macos-15-intel`（image 20260824.0482.1、macOS 15.7.9） | host `x86_64-apple-darwin`、`Darwin x86_64` | 該当なし | `libSystem.B.dylib`、`libiconv.2.dylib` | 未署名（`code object is not signed at all`） | 2 分 40 秒、[job](https://github.com/kiyohara/bizdate/actions/runs/35725908817/job/106739640344) |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm`（image 20260920.137.1、Ubuntu 22.04.5、glibc 2.35） | host `aarch64-unknown-linux-gnu`、`Linux aarch64` | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0` が最大） | `libc.so.6`、`libgcc_s.so.1`（ほかに `ld-linux-aarch64.so.1`、`linux-vdso.so.1`） | - | 0 分 25 秒、[job](https://github.com/kiyohara/bizdate/actions/runs/35725908817/job/106739640723) |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04`（image 20260907.292.1、Ubuntu 22.04.5、glibc 2.35） | host `x86_64-unknown-linux-gnu`、`Linux x86_64` | `GLIBC_2.34`（GCC symbol は `GCC_4.2.0` が最大） | `libc.so.6`、`libgcc_s.so.1`（ほかに `ld-linux-x86-64.so.2`、`linux-vdso.so.1`） | - | 0 分 26 秒、[job](https://github.com/kiyohara/bizdate/actions/runs/35725908817/job/106739640727) |

- 最低 glibc は Linux 2 対象とも `GLIBC_2.34` で、#36 の参考値（Compose の aarch64、glibc 2.41 環境）と一致した。runner の glibc 2.35 ともビルド環境の値とも一致しないことを実測で確認した。
- Apple Silicon の release バイナリには linker の ad-hoc 署名が付く（`linker-signed`）。Intel 側は未署名で、Intel Mac は未署名バイナリを実行できるため配布上の問題は無いが、事実として記録する。
- タイムゾーン採用経路の確認は 4 対象すべてで同じ結果になった（`TZ` 未設定の local 解決も runner の `/etc/localtime` で成功）。
- build だけで終わった対象は無い。4 対象とも test と起動確認まで通っており、未検証の対象は無い。
- `platform-check.sh` の Linux の記録に `ldd --version | head` 由来の `printf: write error: Broken pipe` が混ざった（Ubuntu の `ldd` は shell script で、`head` が pipe を先に閉じる）。動作には影響しないが、`sed -n '1p'` に直して Compose で再確認した。この修正後の run は CI が再実行する。

## リスク・ブロッカー

- 3 thread の resolve と PR #61 の merge は人間の作業として残る。
- `ubuntu-22.04` は support policy 上、`ubuntu-26.04` の GA に伴って deprecation が始まり得る。Linux runner を古い側に固定する方針（0016）は維持するが、label が使えなくなった時点で `ubuntu-24.04` へ移し、最低 glibc の実測値が変わらないことを確認する必要がある。
- `macos-15-intel` は 2027 年秋に提供終了が予告されている（0016 の見直し条件）。
- docs.github.com の公式 runner 一覧は cloud session から開けない。ローカル環境での再確認手段として URL を残す。

## セッションログ

- 2026-09-22: Issue #37 を読み、依存（#36 / PR #42 merge 済み）と open PR 無しを確認。cloud session のブランチ `claude/keen-gates-fvbnj5` で着手。main の ruleset / branch protection を read-only で確認（required status checks なし）。runner 一覧は `actions/runner-images` README で確認（docs.github.com は遮断）。jiff の `TZDIR` / `TZ` の挙動をソースで確認し、tzdb 前提の再現方法を決めた。
- 2026-09-22: CI workflow（`lint` + `platform` matrix）、`platform-check.sh`、guideline / Copilot 指示 / `distribution.md` の前方参照 / `progress.md` を更新。Compose で CI と同じコマンド列を通し、検証欄に記録。
- 2026-09-22: PR #61 を作成し、`number-working-branch-note` で note を採番（commit `c8f9cc0`）。`progress.md` の DIST-02 に PR 番号を反映（`490cc5e`）。P1 の引き上げ項目: 完了として書き換えたタスク行 1 件（note の「PR を作成し、note を採番する」。PR description には該当なし）。触らなかった stale 表現 1 件（「現在の状況」の「PR 作成と CI の実行結果の記録が残っている」は定型外の prose のため採番 skill では触らず、この更新で書き換えた）。採番 skill は停止せず完走した。
- 2026-09-22: head `82b2060` の CI（run 35725908817）が 5 job とも success。4 target の実測（host / uname、テスト件数、最低 glibc、動的リンク先、署名、所要時間）を検証欄に記録。Linux の記録に混ざった `ldd` の SIGPIPE 診断を `sed -n '1p'` で解消し、Compose で再確認。
- 2026-09-22: P2 review（cycle `claude-code-b0339ef-20260922122430`、head `b0339ef`）で指摘 3 件（imo 1 / nits 2、must 0）。P3 で 3 件とも Compose で実物確認し採用: (1) 記録ブロックが `{ } | tee` の pipeline 内にあり `objdump` などの失敗が exit 0 に飲まれる → 一時ファイルへ書き切ってから出力し、整形の前段は変数に受けて失敗を代入時に検出する形へ変更（`objdump` が失敗する stub では exit 3、`ldd` 不在の stub では exit 127 と、tool の終了コードで非 0 に止まることを確認）。(2) `expect_error "date"` が `bizdate: ` prefix に常に一致して無効 → `invalid calendar date` へ変更。(3) 開発コマンド guideline の「上記」が Compose の実行例を指すように読める → 「表の CI 側の実行」と明示。P4 で修正を push し、各 thread へ処置を返信。
- 2026-09-22: P5 再確認（同じ subagent、head `b0708a5`、CI 5 job success）で 3 thread とも resolve 可、未対応 0、新規 inline 指摘 0。完了要約内の `[fyi]` 1 件（note の stub の exit code 表記）はこの更新で直した。P6 で追加対応不要と判断し、1 周で収束。
