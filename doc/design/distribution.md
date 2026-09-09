# 配布仕様

この文書は、`bizdate` の配布物の正本である。対象環境、成果物、命名、checksum、version / tag 規則、同梱要件を扱う。

想定読者は、配布を実装・運用する開発者と AI agent である。

公開の作業手順（誰がいつ何を実行するか、公開前後に何を確認するか）はこの文書では扱わない。手順の正本は `doc/guidelines/` のリリース guideline とする。決定経緯は [`decision-log/0016-distribution-contract.md`](decision-log/0016-distribution-contract.md) を参照する。

## 配布手段

- 配布の起点は GitHub Releases とする。
- 成果物の生成と公開は `dist` に委ねる。
- Homebrew 経由の導線は、既存の個人 tap `kiyohara/homebrew-tap` に Formula を置いて提供する。
- 上記以外の環境は、Releases の archive を直接取得して展開する。shell installer などの追加 installer は v1 では提供しない。

## 対象環境

### target triple

次の 4 対象を配布対象とする。いずれも対象 architecture の runner 上で native ビルドし、クロスビルドは使わない。

| target triple | OS / architecture | libc | 最低実行環境 | ビルド runner |
|---|---|---|---|---|
| `aarch64-apple-darwin` | macOS / arm64 | - | macOS 11.0 | `macos-15` |
| `x86_64-apple-darwin` | macOS / x86_64 | - | macOS 10.12 | `macos-15-intel` |
| `aarch64-unknown-linux-gnu` | Linux / arm64 | glibc | 「libc 条件」に従う | `ubuntu-22.04-arm` |
| `x86_64-unknown-linux-gnu` | Linux / x86_64 | glibc | 同上 | `ubuntu-22.04` |

最低 macOS version は rustc の既定 deployment target である。`MACOSX_DEPLOYMENT_TARGET` は設定せず既定のままとする。

native ビルドを要件とするのは、Apple Silicon が ad-hoc であっても署名済みバイナリしか実行しないためである。macOS runner 上で link すればリンカが ad-hoc 署名を付けるが、他 OS からのクロスビルドでは付かない。

### libc 条件

Linux は glibc 版（`*-unknown-linux-gnu`）だけを配る。musl 版は v1 では配らない。

- 配布バイナリは完全静的ではない。`libc.so.6` と `libgcc_s.so.1` に動的リンクする。
- 実行に必要な最低 glibc は、rustc が示す target 下限（glibc 2.17）ではなく、**バイナリが実際に参照する GLIBC symbol version の最大値**で決まる。ビルド環境の glibc version と同じ値になるとは限らない。
- 最低 glibc は release ごとに実測する。確認方法は次のとおりとし、出力の最大値を最低 glibc とする。

```sh
objdump -T <binary> | grep -o 'GLIBC_[0-9.]*' | sort -u -V | tail -1
```

- 実測は #37 で整備する CI に置き、4 対象すべてについて記録する。参考値として、2026-09-09 時点の `aarch64-unknown-linux-gnu` release build では `GLIBC_2.34` および `GCC_4.2.0` が最大だった。
- 最低 glibc が上がると、それまで動いていた環境が黙って動かなくなる。実測値が上がった場合は release note に明記する。

### 実行時前提

配布バイナリが実行環境へ要求するものを次に示す。「pure Rust に寄せる」方針（[0008](decision-log/0008-language-and-distribution.md)）は C ライブラリを新たに引き込まないことを意味するのであって、完全静的や外部データ不要を意味しない。

| 前提 | 要否 | 内容 |
|---|---|---|
| IANA time zone database | 必須 | `TZDIR`、無ければ `/usr/share/zoneinfo` などの既定ディレクトリを読む |
| システム CA store | 不要 | TLS trust anchor はバイナリに埋め込む |
| 動的ライブラリ | Linux のみ | `libc.so.6`、`libgcc_s.so.1` |
| ネットワーク | `fetch-holidays` のみ | `first` / `last` はネットワークを使わない |
| 祝日データ | `first` / `last` に必須 | 保存先と有効期間は [`business-day.md`](business-day.md) を正本とする |

#### time zone database

`first` / `last` は tzdb が無いと exit code `2` で失敗する。`--date` と `--timezone` を両方与えた場合も失敗する。local timezone の解決を先に行うためである。`--version` と `--help` は tzdb が無くても成功する。

tzdb をバイナリへ bundle する選択は取らない。システム側の更新で祝日以外の時刻解釈が最新に保たれる利点を優先する。tzdb を持たない最小コンテナ（`scratch`、tzdata を入れていない distroless / Alpine など）では、tzdb を別途導入する必要がある。この前提は利用者向けの案内へ反映する。

#### TLS trust

`fetch-holidays` の TLS trust anchor は `webpki-roots` が持つ Mozilla root store をバイナリへ埋め込む。システムの CA store を参照しない。

- システムの CA store が無い環境でも取得できる。
- root 証明書の追加・失効を取り込むには再ビルドと再リリースが要る。企業内 proxy の独自 CA も、この方式では信頼できない。

## 成果物

### archive

- 形式は `.tar.gz` とする。`dist` の既定は `.tar.xz` だが、展開ツールの可用性を優先して変更する。
- 命名は `dist` の既定に従い `bizdate-<target triple>.tar.gz` とする。実際の生成名は `dist plan` の出力と突き合わせて確認する。
- tarball の中身は、archive 名から拡張子を除いた名前のディレクトリ 1 つに入る。展開時に `--strip-components=1` を使うと平坦に取り出せる。
- archive の root に実行ファイル `bizdate` を置く。

### checksum

- `dist` の既定に従い `sha256` とする。archive ごとに `.sha256` ファイルを Release に添付する。
- `sha256sum -c` で検証できる形式とする。
- 署名付き checksum、GPG 署名、sigstore は v1 では導入しない。

### 同梱物

archive には次を含める。

| ファイル | 由来 | 目的 |
|---|---|---|
| `bizdate` | ビルド成果物 | 実行ファイル |
| `README.md` | `dist` の auto-includes | 利用方法 |
| `LICENSE` | `dist` の auto-includes | 本体の MIT License |
| `THIRD-PARTY-LICENSES.md` | `dist` の `include` に明示 | 依存クレートのライセンス表記 |

Homebrew Formula 経由で install した場合の配置は次のとおりで、いずれも保持される。

- `README.md` と `LICENSE` は Homebrew が doc として扱う。
- `THIRD-PARTY-LICENSES.md` はそれ以外のファイルとして `pkgshare`（`$(brew --prefix)/share/bizdate/`）へ入る。

### third-party ライセンス表記

本体の `LICENSE`（MIT）だけでは、依存クレートの再配布条件を満たさない。バイナリを配る以上、次のような条件が現に付く。

| 依存 | ライセンス | 満たすべき点 |
|---|---|---|
| `webpki-roots` | CDLA-Permissive-2.0 | 埋め込む root store は「データ」であり、agreement 本文を配布物に添える |
| `encoding_rs` | (Apache-2.0 OR MIT) AND BSD-3-Clause | WHATWG 由来部分に、binary 形式での再配布時に copyright notice を添付物へ再掲する条項がある |
| `ring` | Apache-2.0 AND ISC | ファイル単位で ISC と Apache-2.0 に分かれ、一方の選択では足りない |
| その他 | MIT / Apache-2.0 / ISC / BSD-3-Clause / Unlicense / BSL-1.0 / Unicode-3.0 | 各ライセンスの表示条件に従う |

要件は次のとおりとする。

- `THIRD-PARTY-LICENSES.md` を生成し、archive に同梱する。crate 名、version、ライセンス識別子、ライセンス本文を含める。
- 生成には `cargo-about` を使う。採用 version は 0.9.2 とする。`cargo install` する場合は `--features cli` が要る。
- 対象は 4 target すべてを合わせた集合とする。platform ごとに別ファイルへ分けない。
- 生成物はリポジトリに commit せず、release build のたびに生成する。依存の追加・更新で内容が古くなる事故を避けるためである。
- 検証は、生成が成功し、かつ `cargo-about` が未許可ライセンスを検出せずに終了することをもって行う。許可するライセンス識別子は生成設定に列挙し、依存追加時に見直す。

## version と tag

### version

- 初回配布の version は `0.1.0` とする。
- 「v1」は機能範囲の呼称であり、semver の major version を指さない。`1.0.0` は公開契約が安定したことを意味する慣習が強いため、初回配布では名乗らない。
- `Cargo.toml` の `version` を正とする。

### tag

- tag 名は `v<version>` とする（例: `v0.1.0`）。
- tag の version 部分は `Cargo.toml` の `version` と一致させる。
- tag の push が release workflow の trigger になる。

### stable と prerelease

- 初回配布は stable とする。
- prerelease が必要になった場合は `v<version>-<prerelease>` 形式とする（例: `v0.2.0-rc.1`）。
- Homebrew Formula は単一 version しか保持しない。prerelease で stable の Formula を上書きしない設定を維持する。

### リリースノート

- `CHANGELOG.md` は v1 では導入しない。
- Release 本文は、`dist` が生成する成果物一覧に、変更点の要約を人間が加える形とする。
- 最低 glibc の実測値が上がった場合と、実行時前提が変わった場合は必ず記載する。

## 署名

- macOS の署名と notarization は行わない。
- `dist` 0.32.0 は macOS の署名・notarization に対応しない。Apple Developer Program の費用も要る。
- 利用上の影響は、macOS の Gatekeeper が `com.apple.quarantine` 属性の付いたファイルにだけ働くことで限定される。Homebrew と `curl` はこの属性を付けない。ブラウザで archive を直接ダウンロードした場合は付くため、Gatekeeper の確認が出る。
- 利用者向けの案内では、macOS では Homebrew か `curl` での取得を主導線とする。
- Windows の署名は、Windows を対象外とするため扱わない。

## Homebrew

- tap は `kiyohara/homebrew-tap` を slapex と共用する（[0010](decision-log/0010-hosting-and-ci-platform.md)）。
- `dist` は Formula を tap の `Formula/` 配下へ書く。既存の `Casks/slapex.rb` とはディレクトリが分かれるため共存できる。Cask を Formula へ移す作業は行わない。
- `dist` は Cask を生成しない。`bizdate` は CLI であり Formula が適切である。
- Formula は 4 対象すべてを 1 ファイルで扱い、`OS` と `Hardware::CPU` で分岐して該当 archive を取得する。

## 対象外

次は配布仕様の対象外とする。

- Windows 向けの配布（[0008](decision-log/0008-language-and-distribution.md)）
- crates.io への公開。`Cargo.toml` の `publish = false` は維持する
- 公開ライブラリクレートとしての提供
- Linux の musl 版、パッケージマネージャ（apt / dnf / AUR など）向けのパッケージ
- コンテナイメージの配布
- 署名付き checksum、GPG 署名、sigstore
- 自動更新（self-update）
