# 作業ブランチメモ

- ブランチ: `define-distribution-contract`
- PR: #42
- 最終更新: 2026-09-09

## 目的

Issue #36 の作業として、初回配布の対象環境・成果物・公開手順を具体化する。配布仕様の正本を `doc/design/` に置き、採否理由を decision log に記録する。後続の #37〜#40 が実装に入れる前提を確定させる。

## 現在の状況

仕様・decision log・関連文書の更新と、後続 Issue の同期まで完了した。PR 作成待ち。

## 調査結果

一次資料の確認日はいずれも 2026-09-09。

### 依存とライセンス

`cargo metadata --locked --filter-platform <target>` で解決した依存は、`x86_64-unknown-linux-gnu` と `aarch64-apple-darwin` で同一の 58 crate だった（platform 差分なし）。feature 解決前の集合であり、実際にリンクされる集合はこれ以下になる。

自前の MIT だけでは満たせない義務が実在する。

| crate | ライセンス | 確認した中身 |
|---|---|---|
| `webpki-roots` 1.0.9 | CDLA-Permissive-2.0 | 同梱 `LICENSE` は CDLA-Permissive-2.0 の全文。配布時は agreement 本文の同梱が要件 |
| `encoding_rs` 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | `LICENSE-WHATWG` に「binary 形式の再配布では copyright notice を documentation か添付物に再掲する」条項がある |
| `ring` 0.17.14 | Apache-2.0 AND ISC | `LICENSE` / `LICENSE-BoringSSL` / `LICENSE-other-bits` / `third_party/fiat` に分かれる |
| `unicode-ident` 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | `LICENSE-UNICODE` を同梱。proc-macro 側の依存 |
| その他 | MIT / Apache-2.0 / ISC / BSD-3-Clause / Unlicense / BSL-1.0 | `subtle` は BSD-3-Clause、`rustls-webpki` と `untrusted` は ISC、`ryu` は Apache-2.0 OR BSL-1.0 |

### 実行時前提（Compose の dev container で実測）

- **IANA tzdb は必須**。`/usr/share/zoneinfo` と `/etc/localtime` を消すと、`--date` と `--timezone` を両方与えても `first` が exit 2 で失敗する（`cannot resolve local time zone: failed to find system time zone`）。`--timezone Asia/Tokyo` だけを与えた場合も `unknown IANA time zone` になる。`--version` は成功する。jiff は Unix では既定で bundle せず `TZDIR` → 既定ディレクトリの順に探すため。
- **システム CA store は不要**。`/etc/ssl/certs` と `/usr/share/ca-certificates` を削除しても `fetch-holidays` は成功した。TLS trust は `webpki-roots` が bundle する Mozilla root store を使う。root 証明書の更新には再ビルドと再リリースが要る。
- **`*-linux-gnu` は静的リンクではない**。release build は `libc.so.6` と `libgcc_s.so.1` に動的リンクし、`file` は `dynamically linked` を示した。pure Rust 方針は C ライブラリを引き込まないことを意味するが、libc への動的リンクは残る。

### 対象環境の一次資料

- Rust platform support: `aarch64-apple-darwin` は Tier 1 / macOS 11.0+、`x86_64-apple-darwin` は **Tier 2 に降格済み** / macOS 10.12+、`*-unknown-linux-gnu` は Tier 1 / glibc 2.17+、`*-unknown-linux-musl` は Tier 2 / musl 1.2.5。<https://doc.rust-lang.org/rustc/platform-support.html>
- GitHub-hosted runner: `ubuntu-22.04` / `ubuntu-24.04` と `-arm` 版、`macos-15` / `macos-26`（arm64）、`macos-15-intel` / `macos-26-intel`（Intel）。public repo は無料。<https://docs.github.com/en/actions/reference/runners/github-hosted-runners>
- macOS Intel runner は 2027 年秋の macOS 15 image 引退で提供終了予定。
- dist の最新安定版は 0.32.0（2026-05-22）。Homebrew formula は tap の `Formula/` へ書き、`HOMEBREW_TAP_TOKEN` を source repo の secret として要求する。Cask 生成は非対応。macOS の署名・notarization も非対応。
- 既存 tap `kiyohara/homebrew-tap` は `Casks/slapex.rb` と `README.md` のみ。`Formula/` は未作成のため、ディレクトリが分かれて共存できる。
- macOS の Gatekeeper は `com.apple.quarantine` が付いたファイルにだけ働く。Homebrew と `curl` は付けないが、ブラウザでの直接ダウンロードは付ける。Apple Silicon は ad-hoc でも署名を要求するが、macOS 上で native link すればリンカが自動で付ける。

## 決定事項

配布仕様の正本を `doc/design/distribution.md` に新設し、採否理由を `doc/design/decision-log/0016-distribution-contract.md` に記録した。主な判断は次のとおり。

- 対象は 4 target。Linux は gnu のみとし musl は配らない。musl は `ring` の C コードが musl 向け C toolchain を要求し、実測でビルドが失敗した。4 target 分の CI に載せる手当が重い。
- 4 対象すべて native runner でビルドする。`macos-15` / `macos-15-intel` / `ubuntu-22.04` / `ubuntu-22.04-arm`。Linux 側を古い image に固定するのは最低 glibc を上げないため。
- 最低 glibc は `objdump -T` の参照 symbol version で実測する。ビルド環境の glibc とは一致しない。
- tzdb は bundle せずシステム依存とする。TLS trust は `webpki-roots` の埋め込みを維持する。
- archive は `.tar.gz`、checksum は `sha256`。`THIRD-PARTY-LICENSES.md` を `cargo-about` 0.9.2 で生成して同梱する。commit はしない。
- 初回 version は `0.1.0`、tag は `v0.1.0`。macOS の署名・notarization は行わない。
- `publish = false` を維持し、package 単位の `dist = true` で bin を配布対象にする。
- installer は Homebrew のみ。`CHANGELOG.md` は導入しない。

公開の運用条件（trigger、権限、secrets、公開前後チェック、復旧）は 0016 に記録し、作業手順の正本は #40 のリリース guideline とした。

`doc/guidelines/development-command-guidelines.md` に「Compose の外に出るもの」を追記した。原則（ローカル検証は Compose が正）は変えず、macOS native ビルドと 4 対象の実行確認、最低 glibc の実測、release 成果物の生成を CI 側の例外として明示した。

README の「配布用バイナリは提供していません」は変更していない。公開前に手順だけ書くと事実と食い違うため、実公開後に #40 の専用 PR で切り替える。

## 次にやること

- PR 採番後に note を rename する。
- レビュー後、#37 へ進む。

## 検証

- Compose で `cargo fmt --check`（OK）、`cargo clippy --locked --all-targets -- -D warnings`（OK）、`cargo test --locked`（unit 69 件・CLI E2E 12 件すべて成功）、`cargo build --locked`（OK）。文書のみの変更だが、調査中に release build と実行確認を行ったため、最終状態でも一通り実行した。ソースコードは変更していない。
- 実行時前提の実測（Compose の dev container、aarch64）。tzdb を消すと `first` が exit 2 で失敗し `--version` は成功すること、CA store を消しても `fetch-holidays` が成功すること、release build が `libc.so.6` と `libgcc_s.so.1` に動的リンクすることを確認した。
- 最低 glibc の実測方法を確認した。`objdump -T` の参照 GLIBC symbol version の最大は `GLIBC_2.34` で、ビルド環境の glibc 2.41 とは一致しなかった。
- musl ビルドを試し、`aarch64-linux-musl-gcc` が見つからず失敗することを確認した。musl を採らない根拠にした。
- `cargo-about` 0.9.2 を dev container に入れて実際に生成させ、`webpki-roots` の CDLA-Permissive-2.0、`encoding_rs` の BSD-3-Clause、`ring` の ISC / Apache-2.0、`unicode-ident` の Unicode-3.0 を含む一覧が出ることを確認した。`cargo install` には `--features cli` が要る。
- dist 0.32.0 の Formula template を読み、`THIRD-PARTY-LICENSES.md` が `pkgshare` に入ることを確認した。
- 更新した文書の相対リンクと backtick 表記の path をすべて解決確認した。切れは無い。
- `git diff --check` は clean。
- note の情報統制を確認した。秘密情報、個人情報、認証情報付き URL、ローカル絶対 path は含まない。
- Issue #37〜#40 の本文を更新し、read-back で節構成が保たれていることを確認した。

### 未検証

- 4 対象すべての実機ビルドと実行（#37）。macOS 側は Compose では確認できない。
- `dist` の実行、release archive の生成、実際の Release 公開、Homebrew の install / upgrade（#38 / #39 / #40）。
- 最低 glibc の 4 対象での実測。今回の値は Compose の aarch64 環境での参考値である。

## リスク・ブロッカー

- macOS Intel runner の提供終了予定（2027 年秋）と `x86_64-apple-darwin` の Tier 2 降格により、Intel macOS のサポートは有限である。
- 実際の Release 公開、Homebrew install / upgrade、4 対象すべての実機実行は本 Issue のスコープ外であり未検証のまま残る。

## セッションログ

- 2026-09-09: Issue #36 を読み、依存（PR #41 merge 済み）を確認して着手した。
- 2026-09-09: 依存ライセンス、実行時前提、対象環境、dist、tap の調査を実施した。実行時前提は dev container で実測した。
- 2026-09-09: `doc/design/distribution.md` と decision log 0016 を作成し、index・concept・cli-interface・design README・AGENTS・copilot 指示・開発コマンド guideline・progress を更新した。Issue #37〜#40 に「#36 の決定（同期）」節を追加した。
