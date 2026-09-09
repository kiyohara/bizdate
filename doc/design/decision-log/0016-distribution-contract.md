<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 初回配布の対象環境・成果物・公開運用

- 状態: decided
- 作成日: 2026-09-09
- 最終更新日: 2026-09-09
- 関連: `doc/design/distribution.md`, `doc/design/concept.md`, `doc/design/cli-interface.md`

## 背景

v1 の CLI 実装（Issue #7〜#15）が完了し、CI も安定した。一方 GitHub Releases は 0 件で、README は配布バイナリ未提供と明記している。配布手段の詳細は index の未決事項に残っていた。

[0008](0008-language-and-distribution.md) と [0010](0010-hosting-and-ci-platform.md) で、Rust、macOS / Linux の amd64 / arm64、GitHub Releases、`dist`、既存 tap `kiyohara/homebrew-tap` の共用までは決まっている。決まっていないのは、target triple、最低実行環境、実行時前提、成果物の形、version と tag、同梱要件、公開の運用条件である。後続の #37〜#40 はこれらが決まらないと実装に入れない。

配布は一度公開すると取り消しが効きにくい。実装前に、一次資料と実測で前提を確認しておく必要がある。

## 候補

対象環境について。

- A: `*-unknown-linux-gnu` の 2 target + macOS 2 target（計 4）
- B: Linux を musl 静的リンクにする
- C: Linux で gnu と musl の両方を配る（計 6）

third-party ライセンス表記について。

- D: 自前の `LICENSE`（MIT）だけを同梱する
- E: `cargo-about` で生成した表記を同梱する
- F: `cargo-bundle-licenses` で生成した表記を同梱する

初回 version について。

- G: `0.1.0`
- H: `1.0.0`

macOS 署名について。

- I: 署名・notarization を行わない
- J: Apple Developer Program に加入し、`dist` の `macos-sign` で署名する（notarization は別途自作する）

## 検討内容

### 対象環境と libc

Compose の dev container で実測したところ、`*-unknown-linux-gnu` の release build は `libc.so.6` と `libgcc_s.so.1` に動的リンクしていた。pure Rust 方針は C ライブラリを新たに引き込まないことを意味するが、libc への動的リンクは残る。「pure Rust だから完全静的」という前提は成り立たない。

そこで musl（候補 B / C）を検討した。`rustup target add aarch64-unknown-linux-musl` は成功したが、ビルドは `failed to find tool "aarch64-linux-musl-gcc"` で失敗した。TLS に使う `ring` が C コードを含み、musl 向けの C toolchain を要求するためである。musl を選ぶと、4 target 分の CI に musl cross toolchain の準備が加わる。#37 / #38 の実装コストが上がる。

一方、gnu を選んだ場合の最低 glibc は実測で制御できる。`objdump -T` で参照 GLIBC symbol version を数えたところ、最大は `GLIBC_2.34` だった。ビルド環境の glibc は 2.41 であり、最低 glibc がビルド環境の値と一致するわけではないことも確認した。参考として Ubuntu 22.04 の glibc は 2.35、24.04 は 2.39 である。

ビルド runner を古い側（`ubuntu-22.04` 系）に固定すれば、runner image の更新で最低 glibc が黙って上がる事故を抑えられる。実測を CI に置けば、上がったこと自体は検知できる。

macOS は native ビルドが要件になる。Apple Silicon は ad-hoc であっても署名済みバイナリしか実行せず、macOS runner 上で link すればリンカが自動で ad-hoc 署名を付ける。他 OS からのクロスビルドではこれが付かない。

runner の公式一覧を 2026-09-09 に再確認したところ、`macos-15` / `macos-26`（arm64）、`macos-15-intel` / `macos-26-intel`（Intel）、`ubuntu-22.04` / `ubuntu-24.04` と各 `-arm` 版があり、public リポジトリでは無料である。4 対象すべてを native runner で賄える。ただし macOS Intel runner は 2027 年秋の macOS 15 image 引退で提供終了が予告されており、`x86_64-apple-darwin` は rustc の Tier 2 へ降格済みである。Intel macOS のサポートは有限である。

### 実行時前提

`jiff` は Unix では tzdb を bundle せず、`TZDIR` か既定ディレクトリのシステム tzdb を読む。実測では、`/usr/share/zoneinfo` と `/etc/localtime` を消すと `first` が exit code `2` で失敗した。`--version` は成功した。配布バイナリは外部データに依存する。

失敗の因果は採用経路で分かれる。`src/date.rs` の `resolve_timezone_with` は `--timezone` があればその IANA 名を直接解決して return し、local timezone は `--timezone` と `BIZDATE_TZ` の両方が無い場合だけ参照する。実測でも、`Asia/Tokyo` の zoneinfo だけを戻して `/etc/localtime`・`TZ`・`BIZDATE_TZ` を欠いたままにすると `--timezone Asia/Tokyo` は成功した（`--date` の有無によらない）。tzdb 全体を消したときの `--timezone Asia/Tokyo` の失敗は、local timezone の先行解決ではなく指定 IANA 名のデータ不在による。

必要なのは採用するタイムゾーンのデータであって、local timezone が設定されていること自体ではない。local timezone を採用する経路（`--timezone` も `BIZDATE_TZ` も無い場合）だけは、`--date` を与えても解決を省略しない。CI と利用者案内で local timezone の設定まで必須化しないよう、この区別を spec の表に落とす。

TLS trust は逆だった。`/etc/ssl/certs` と `/usr/share/ca-certificates` を削除しても `fetch-holidays` は成功した。`webpki-roots` が Mozilla root store をバイナリへ埋め込むためである。システム CA store に依存しない代わりに、root 証明書の更新には再リリースが要る。

### third-party ライセンス表記

依存 58 crate のライセンスを `cargo metadata` で確認し、registry 上の実ファイルも読んだ。自前の MIT だけでは足りない条件が実在する。

- `webpki-roots` は CDLA-Permissive-2.0 で、同梱 `LICENSE` は agreement の全文だった。埋め込む root store は「データ」であり、共有時に agreement 本文を添えることが要件である。
- `encoding_rs` は `LICENSE-WHATWG` を持ち、binary 形式での再配布時に copyright notice を添付物へ再掲する条項がある。
- `ring` は `LICENSE` / `LICENSE-BoringSSL` / `LICENSE-other-bits` / `third_party/fiat` に分かれ、ファイル単位で ISC と Apache-2.0 が混在する。
- `unicode-ident` は Unicode-3.0 を含む。

候補 D は成り立たない。E と F はどちらも同種の道具である。`cargo-about` 0.9.2 を dev container に入れて実際に生成させたところ、上記 4 件を含む一覧を出力できた。`targets` 指定で対象 4 triple に絞る機能もある。実地で動くことを確認できた側を採る。`cargo install` で bin を得るには `--features cli` が要ることも確認した。

生成物を commit すると、依存を更新したのに表記が古いまま配られる事故が起きうる。release build のたびに生成する方が安全である。

Homebrew 経由でも保持されるかは、`dist` 0.32.0 の Formula template で確認した。`README.*` / `LICENSE` / `LICENSE.*` / `CHANGELOG.*` は doc として扱われ、それ以外の同梱ファイルは `pkgshare` に入る。`THIRD-PARTY-LICENSES.md` は後者に該当し、配置は変わるが失われない。

### version と tag

「v1」は本リポジトリでの機能範囲の呼称であり、semver の major を約束するものではない。`1.0.0`（候補 H）は公開契約の安定を意味する慣習が強い。CLI の option と exit code は spec で固めたが、配布とインストールの導線は初回であり、実利用のフィードバックで変える余地を残したい。内部 lib の入口も互換性保証の対象外である（[0008](0008-language-and-distribution.md)）。`Cargo.toml` の現在値も `0.1.0` であり、初回から飛ばす理由がない。

### `publish = false` と配布対象

`dist` は既定で `publish = false` の package を配布対象から外す。package 単位の `dist = true` がこれを上書きする。`Cargo.toml` に書く場合は `[package.metadata.dist]` に置く。crates.io へ公開しない方針を変えずに bin だけを配れる。

### 署名

`dist` 0.32.0 のソースを確認したところ、macOS の署名には対応している。build 設定に `macos-sign` があり、`sign/macos.rs` が一時 keychain へ証明書を取り込んで `/usr/bin/codesign` を実行する。生成される release workflow も、有効時に `CODESIGN_IDENTITY` / `CODESIGN_CERTIFICATE` / `CODESIGN_CERTIFICATE_PASSWORD`（任意で `CODESIGN_OPTIONS`）を secret から渡す。一方 notarization には対応せず、同モジュールの doc comment に将来対応する旨だけが書かれている。

したがって候補 J の障壁は機能の有無ではない。Apple Developer Program の年額費用、証明書と secret の運用（失効・更新を含む）、notarization を `dist` の外に自作する必要が残ることである。

一方、Gatekeeper は `com.apple.quarantine` 属性が付いたファイルにだけ働く。Homebrew と `curl` はこの属性を付けず、ブラウザでの直接ダウンロードだけが付ける。主導線を Homebrew と `curl` に置けば、未署名の影響は限定される。

### Compose と CI / release build の関係

`doc/guidelines/development-command-guidelines.md` は、記録・報告する検証は Compose 経由で行うと定めている。この原則は変えない。

ただし配布では、macOS native ビルドと 4 target の実行確認が Compose の外に出る。Compose は Linux コンテナであり、macOS バイナリを作れないし動かせない。原則を「ローカル検証は Compose が正」と読み替え、CI / release workflow 上のビルドと実行確認は Compose の対象外だと guideline に明示する必要がある。

### 公開の運用条件

`dist` の release workflow は tag push を trigger とし、plan / build / host / publish / announce の順に進む。GitHub Release が作られる段階は `github-release` 設定で決まり、既定の `auto` は `host` に解決される。既定のままなら Release は host 段階で公開され、announce まで draft に留まるわけではない。

この既定を変えない。`github-release = "announce"` にすると、生成した Homebrew installer が公開前の URL を参照する短い窓ができ、その間の install が失敗する。公式文書も既定からの変更を勧めていない。

運用上の帰結として、publish 段階の Homebrew 更新が失敗して announce に到達しなくても、GitHub Release は公開済みになり得る。失敗時に「Release は未公開」と決めつけず、Release の実状を確認してから復旧を判断する。

tap への push には `HOMEBREW_TAP_TOKEN` という secret（`repo` scope の PAT）を source リポジトリ側に置く必要がある。secrets の設定はユーザーが行う。agent は secrets を操作しない（`doc/guidelines/github-cli-guidelines.md`）。

Homebrew は単一 version しか保持しない。同じ version を作り直すと Formula が指す成果物と checksum がずれる。復旧は「同じ version の再公開」ではなく「version を上げてやり直す」に倒すのが安全である。

## 決定

配布仕様の正本を `doc/design/distribution.md` に置く。主要な決定は次のとおりで、詳細は同 spec を正とする。

- 対象は 4 target。Linux は gnu 版のみとし、musl 版は配らない（候補 A）。すべて native runner でビルドする。
- ビルド runner は `macos-15` / `macos-15-intel` / `ubuntu-22.04` / `ubuntu-22.04-arm` とする。Linux 側は最低 glibc を上げないため古い側に固定する。
- 最低 glibc は `objdump -T` の参照 GLIBC symbol version で実測し、release ごとに確認する。実測は #37 の CI に置く。
- tzdb はバイナリに bundle せず、システム tzdb を前提とする。採用するタイムゾーンのデータが読めない場合に `first` / `last` が失敗することを、採用経路（`--timezone` / `BIZDATE_TZ` / local）ごとに spec の表へ書く。local timezone の設定を一律に必須要件としない。
- TLS trust は `webpki-roots` の埋め込みを維持する。root 更新には再リリースが要ることを記録する。
- archive は `.tar.gz`、checksum は `sha256` とする。
- `THIRD-PARTY-LICENSES.md` を `cargo-about` 0.9.2 で生成し、archive に同梱する（候補 E）。commit はせず release build のたびに生成する。Homebrew 経由では `pkgshare` に入る。
- 初回配布の version は `0.1.0`、tag は `v0.1.0` とする（候補 G）。
- macOS の署名と notarization は行わない（候補 I）。`dist` の `macos-sign` は使えるが、費用と運用負担に見合わないと判断した。利用者向けの主導線を Homebrew と `curl` に置く。
- `Cargo.toml` の `publish = false` を維持し、package 単位の `dist = true` で bin を配布対象にする。
- installer は Homebrew のみとする。shell installer は v1 では提供しない。
- `CHANGELOG.md` は導入しない。Release 本文は `dist` の生成物一覧に人間が要約を加える。

公開の運用については次を決める。作業手順そのものの正本は #40 が整える `doc/guidelines/` のリリース guideline とし、本ログと配布 spec を手順の正本にはしない。

- 公開 trigger は `v<version>` tag の push とする。PR の merge は公開ではない。
- release workflow は `contents: write` を要する。tap 更新のため `HOMEBREW_TAP_TOKEN` を source リポジトリの secret に置く。secrets の設定と tag の push はユーザーが行う。
- 公開前に、CI が green であること、`Cargo.toml` の version と tag が一致すること、`dist plan` が想定どおりの成果物を列挙すること、third-party 表記が生成できることを確認する。
- 公開後に、4 target の archive と checksum が揃っていること、Homebrew での install と upgrade が通ること、`bizdate --version` が tag と一致することを確認する。
- `github-release` は既定（`auto` → `host`）のままとする。Release は host 段階で公開されるため、publish 段階の Homebrew 更新が失敗して announce に到達しなくても Release は公開済みになり得る。復旧手順では Release の実状を確認してから判断し、未公開と決めつけない。
- 失敗時は同じ version を作り直さず、version を上げてやり直す。Homebrew Formula が単一 version しか保持しないためである。

`doc/guidelines/development-command-guidelines.md` に、Compose がローカル検証の正であること、CI / release workflow 上の macOS native ビルドと対象環境での実行確認はその例外であることを追記する。原則そのものは変えない。

## 理由

musl を採らないのは、`ring` の C コードが musl 向け toolchain を要求することを実測で確認したためである。得られる可搬性に対して、4 target 分の CI に載せる手当が重い。gnu でも最低 glibc は実測で制御でき、runner を固定すれば黙って上がる事故を抑えられる。

third-party 表記を入れるのは、`webpki-roots` の CDLA-Permissive-2.0 と `encoding_rs` の WHATWG 条項が、バイナリ配布に対して現に条件を課すためである。自前の MIT だけでは満たせない。

初回を `0.1.0` にするのは、配布導線が初回で、まだ安定を約束する段階ではないためである。

macOS を署名しないのは、`dist` の署名機能は使えるものの、Apple Developer Program の費用と証明書・secret の運用負担が要り、notarization は結局自作になる一方で、主導線では Gatekeeper が働かないためである。

## 影響

- `doc/design/distribution.md` を新設し、`concept.md` / `cli-interface.md` / `doc/design/README.md` / `AGENTS.md` から参照する。
- decision log index の未決事項「配布手段の詳細」を解消し、本ログを主要方針へ加える。
- #37 は 4 target の native ビルドと実行確認、最低 glibc の実測、tzdb 前提の確認を CI に載せる。
- #38 は `dist` 0.32.0 系の導入、`dist = true`、`.tar.gz`、`sha256`、`THIRD-PARTY-LICENSES.md` の生成と同梱を実装する。
- #39 は tap の `Formula/` への出力と、`pkgshare` に third-party 表記が残ることの確認を扱う。
- #40 はリリース手順の guideline、`HOMEBREW_TAP_TOKEN` の設定案内、README のインストール案内を整える。tzdb 前提と macOS の取得方法も案内に含める。
- `.github/copilot-instructions.md` の「配布手段は未決」という前提を更新する。
- README のインストール案内は、実際に Release を公開してから #40 で切り替える。公開前に手順だけ書くと事実と食い違う。

## 後から見直す条件

- 古い glibc の環境で動かないという要望が実際に出た場合。musl 版の追加、または古い glibc でのビルド環境の用意を再検討する。
- macOS Intel runner が使えなくなった場合。`x86_64-apple-darwin` は Tier 2 であり、2027 年秋に提供終了が予告されている。cross-build のみになった時点でサポート範囲を再判断する。
- Homebrew を使わない Linux 利用者の導線が必要になった場合。shell installer の追加を検討する。
- 配布バイナリの改竄検知を強めたい場合。署名付き checksum や sigstore を再検討する。
- macOS で Gatekeeper の確認が実際に利用者の障害になった場合。`dist` の `macos-sign` は既に使えるため、費用と運用負担を引き受けるかの判断になる。
- `dist` が notarization に対応した場合。
- 公開契約が安定し、`1.0.0` を名乗る段階になった場合。
