# 開発コマンド実行ルール

この文書は、bizdate リポジトリで `cargo` などの開発コマンドを実行するときの共通正本である。AI agent と人間の両方がこのルールに従う。

toolchain と依存クレートの選定経緯は `doc/design/decision-log/0014-dependency-crates-and-toolchain.md` を参照する。

## 原則

- 開発コマンドは host OS 上で直接実行せず、Compose 経由で実行する。
- 実行環境は repo root の `compose.yaml` と `Dockerfile` を正とする。開発用 service 名は `dev` とする。
- host に Rust toolchain を入れることは禁止しない。ただし、検証結果として記録・報告する実行は Compose 経由のものとする。host の toolchain は MSRV や component 構成が揃っている保証が無い。

## Compose の外に出るもの

Compose は Linux コンテナ 1 種類の実行環境であり、配布対象すべてを賄えない。次は Compose の対象外とし、CI / release workflow 上で実行する。配布仕様の正本は `doc/design/distribution.md`、経緯は `doc/design/decision-log/0016-distribution-contract.md` を参照する。

| 実行 | 場所 | 理由 |
|---|---|---|
| macOS 向けの native ビルド | CI の `platform` job（`macos-15`） | Compose は Linux コンテナで、macOS バイナリを作れない。Apple Silicon は native link でリンカが付ける ad-hoc 署名を要求する |
| 配布対象 3 target での実行確認 | CI の `platform` job（3 つの native runner） | 対象 OS / architecture 上でしか実行成功を確認できない |
| 最低 glibc の実測 | CI の `platform` job（Linux runner） | 配布に使うバイナリそのものを測る必要がある |
| release 成果物の生成と公開 | release workflow の build job（3 つの native runner）と `host` job | `dist` が対象 runner 上で archive と checksum を作る。公開は `v<version>` tag の push でだけ行う |
| 配布 archive の検証 | release workflow の `release-verify` job（3 つの native runner） | 配る binary そのものを対象環境で起動して確かめる |
| Homebrew Formula での install と `brew test` | release workflow の `release-verify` job の `homebrew`（3 つの native runner） | Compose の container には Homebrew が無い。cloud session の sandbox では Homebrew の portable Ruby を取得できない |

CI（`.github/workflows/ci.yml`）は、`fmt` / `clippy` を Linux で 1 回だけ回す `lint` job と、配布対象 3 target をそれぞれの native runner で回す `platform` job から成る。`platform` job は runner の host triple が対象 target と一致することを確かめてから、`cargo test --locked`（unit / CLI E2E）、`cargo build --locked --release`、release バイナリに対する CLI E2E、`.github/scripts/platform-check.sh` による起動確認の順に進む。`platform-check.sh` が最低 glibc（Linux）、最低 macOS version（macOS）、動的リンク先を job の step summary に記録し、macOS では binary に署名があることも確かめる。同じ script は Compose でも実行でき、Linux コンテナ上の結果が得られる。

```sh
docker compose run --rm dev sh -c 'cargo build --locked --release && .github/scripts/platform-check.sh target/release/bizdate'
```

原則は変えない。ローカルで行う検証は Compose 経由を正とする。表の CI 側の実行（native ビルド、3 target の実行確認、最低 glibc の実測、release 成果物の生成と検証）の結果を報告するときは、Compose 経由の結果と区別し、どの runner で実行したかを書く。

## cloud session（Claude Code on the web）

Claude Code on the web の cloud session では Docker daemon が起動していないが、Compose 経由の原則は変えない。`.claude/settings.json` に登録した SessionStart hook が `.agents/scripts/cloud-session-setup.sh` で daemon を起動し、開発用 image を用意し、cloud 専用の override（`compose.cloud.yaml`）を `COMPOSE_FILE` で重ねる。開発コマンドの形は変わらず、検証結果は Compose 経由として扱う（報告には「cloud session で実行」と添える）。daemon が起動しない場合も sandbox の `cargo` で代替しない。詳細は `doc/guidelines/cloud-session-guidelines.md` を参照する。

## 基本形

```sh
docker compose run --rm dev cargo <subcommand>
```

日常の開発では次を使う。

```sh
docker compose run --rm dev cargo build
docker compose run --rm dev cargo test
docker compose run --rm dev cargo fmt --check
docker compose run --rm dev cargo clippy --all-targets -- -D warnings
```

検証結果として記録・報告する実行と CI では、`--locked` を付けた形を基本形とする。`Cargo.lock` を commit して再現性を担保する方針（`doc/design/decision-log/0014-dependency-crates-and-toolchain.md`）に対し、`--locked` が無いと lock と `Cargo.toml` がずれていても黙って解決し直して先へ進むためである。

```sh
docker compose run --rm dev cargo build --locked
docker compose run --rm dev cargo test --locked
docker compose run --rm dev cargo fmt --check
docker compose run --rm dev cargo clippy --locked --all-targets -- -D warnings
```

`cargo fmt` は `Cargo.lock` を解決しないため `--locked` を受け付けない（付けると exit code 2 になる）。

ビルドしたバイナリも container の中で実行する。`target/` は named volume に置いてあり、host からは見えない。

```sh
docker compose run --rm dev ./target/debug/bizdate --version
```

複数コマンドをまとめる場合は `sh -c` を使う。

```sh
docker compose run --rm dev sh -c 'cargo build && ./target/debug/bizdate --version'
```

## image の更新

`Dockerfile` を変更したときは image を作り直す。`release-tools` service を使う場合は、それも作り直す（「配布成果物の生成と確認」）。

```sh
docker compose build dev
```

## 配布成果物の生成と確認

配布成果物は `dist` が作る。設定は `dist-workspace.toml`、配布物の仕様は `doc/design/distribution.md`、release workflow の構成の採否理由は `doc/design/decision-log/0022-release-workflow.md` を参照する。

### ローカルの実行環境

`dist` と `cargo-about` は dev image に入れず、Compose の `release-tools` service（`Dockerfile` の `release-tools` stage）で実行する。dev image に 2 つの tool を足しただけで、toolchain と volume は dev と共有する。profile を付けているため、`docker compose run` で名指ししたときだけ使われる。初回と `Dockerfile` の変更後は image を作る。

```sh
docker compose build release-tools
```

cloud session（Claude Code on the web）では `release-tools` を使えない。`compose.cloud.yaml` の override（base image の mirror、proxy）を `dev` にだけ置いており、`release-tools` の build と外部からの取得が通らないためである。`release-tools` を要する作業（`dist generate` など）はローカルで行う。

`release.yml` を設定から作り直す。`release.yml` は手で直さない。dist の設定（`dist-workspace.toml`、`.github/build-setup.yml`、`Cargo.toml` の `[package.metadata.dist]` と `[profile.dist]`）を変えたら作り直し、設定と一緒に commit する。

```sh
docker compose run --rm release-tools dist generate
```

生成物が設定と一致すること、配布する成果物の一覧（archive、同梱物、checksum）を確かめる。release workflow の `plan` job も、実行のたびに同じ一致を検査する。

```sh
docker compose run --rm release-tools sh -c 'dist generate --check && dist plan'
```

third-party 表記（`THIRD-PARTY-LICENSES.md`）を生成し、配布対象の依存と対応していることを確かめる。生成物は commit しない（`.gitignore` 済み）。

```sh
docker compose run --rm release-tools sh -c 'cargo about generate --locked --fail --output-file THIRD-PARTY-LICENSES.md about.hbs && .github/scripts/check-third-party-licenses.sh THIRD-PARTY-LICENSES.md'
```

container と同じ target の archive を作り、release workflow の `release-verify` job と同じ検証を通す。`dist build` は `cargo build` に `--locked` を付けないため、先に `cargo fetch --locked` を実行する。

```sh
docker compose run --rm release-tools sh -c 'target=$(rustc -vV | sed -n "s/^host: //p") && cargo fetch --locked && cargo about generate --locked --fail --output-file THIRD-PARTY-LICENSES.md about.hbs && dist build --artifacts=local --target="$target" && dist plan --output-format=json > target/distrib/plan.json && .github/scripts/verify-release-archive.sh "$target" target/distrib target/distrib/plan.json'
```

tag の形と、第三者 action の SHA 固定を確かめる。どちらも dev service で動く。

```sh
docker compose run --rm dev .github/scripts/check-release-tag.sh v0.1.0
docker compose run --rm dev .github/scripts/check-action-pins.sh
```

Compose で確かめられるのは、Linux コンテナと同じ target の archive だけである。macOS と別 architecture の archive は release workflow の結果で確かめ、報告では runner を書く。

### release workflow

`.github/workflows/release.yml` は PR と tag の push で走る。PR では公開の直前までを通し、`host` 以降は走らない。

| job | 内容 | PR | tag の push |
|---|---|---|---|
| `plan` | `dist plan`（tag の push では `dist host --steps=create`）で成果物を決め、`release.yml` と設定の一致を検査する | 走る | 走る |
| `build-local-artifacts` | 3 target の native runner で `.github/build-setup.yml`（toolchain を CI に揃える、`cargo fetch --locked`、third-party 表記の生成と照合）を実行し、`dist build` で archive と checksum を作る | 走る | 走る |
| `custom-ci` | `.github/workflows/ci.yml` を呼び、同じ commit で CI を通す。PR では直接の CI と重複する `platform` を省く | `lint` だけ走る | 全 job が走る |
| `build-global-artifacts` | 全 archive の checksum をまとめた `sha256.sum` を作る | 走る | 走る |
| `custom-release-verify` | `.github/workflows/release-verify.yml`。tag が `v<Cargo.toml の version>` であることと、3 target の archive を検証する。dist が生成した Homebrew Formula を検査して test を足し、公開前の archive で install と `brew test` を行う | 走る | 走る |
| `host` | GitHub Release を作り、成果物を添付する | 走らない | 上の job がすべて成功したときだけ走る |
| `custom-publish-homebrew` | `.github/workflows/publish-homebrew.yml`。検査した Formula を tap の `Formula/bizdate.rb` へ書く（`doc/design/distribution.md` の「tap の更新」） | 走らない | `host` の成功後に走る。prerelease では skipped |
| `announce` | dist の後処理 | 走らない | `host` と `custom-publish-homebrew` の後に走る |

検証の結果（archive 名、sha256、構成、`platform-check.sh` の記録、Homebrew での install の記録）は `custom-release-verify` の各 job の step summary に残る。PR で確かめた archive はその run の workflow artifact であり、公開された Release asset ではない。

### tool と action の version を上げるとき

| 対象 | 箇所 | 注意 |
|---|---|---|
| dist | `dist-workspace.toml` の `cargo-dist-version`、`Dockerfile` の `release-tools` stage（version と checksum） | 上げたら `dist generate` で `release.yml` を作り直す。2 箇所がずれると `dist generate` が version の不一致で止まる |
| cargo-about | `.github/scripts/install-cargo-about.sh`（version と 3 つの asset の sha256） | upstream の prebuilt を入れる。上げたら 3 つの asset を取得して sha256 を計算し、同じ release の `.sha256` と一致することを確かめてから固定する。`Dockerfile` と release workflow の build job が同じ script で入れるため、`release-tools` の image も作り直す |
| `release.yml` の第三者 action | `dist-workspace.toml` の `github-action-commits` | 値は `"<SHA> # <tag>"` の形で書き、`dist generate` で作り直す。Dependabot の更新 PR が `release.yml` を変えた場合の扱いは `doc/guidelines/development-loop.md` を参照する |

## MSRV を上げるとき

MSRV の値は 5 箇所に現れる。上げるときは 5 つを同時に更新する。

| 箇所 | 値 | 役割 |
|---|---|---|
| `Cargo.toml` | `rust-version` | crate が要求する最小 Rust version |
| `Dockerfile` | `FROM ${BASE_REGISTRY}/rust:<MSRV>-trixie` | container の toolchain を実際に決める。`BASE_REGISTRY` は base image の取得元の差し替え口で、MSRV とは無関係 |
| `compose.yaml` | `image: bizdate-dev:<MSRV>`、`image: bizdate-release-tools:<MSRV>` | build した image に付ける local tag 名 |
| `.github/workflows/ci.yml` | `RUST_TOOLCHAIN: "<MSRV>"` | CI の toolchain を決める。workflow の `env` に 1 つだけ置き、`lint` と `platform` の両 job が参照する |
| `.github/build-setup.yml` | `rustup toolchain install <MSRV>` と `rustup default <MSRV>` | release workflow の build job が配布 archive をビルドする toolchain を決める。dist は toolchain を指定しないため、ここで CI と揃える |

`compose.yaml` の tag はビルド結果に影響しないが、ずれると tag が実態を偽る。`Dockerfile` の `FROM` だけ古いまま `rust-version` を上げると、build が MSRV エラーで落ちる。`.github/workflows/ci.yml` の `RUST_TOOLCHAIN` だけ古いまま上げると、CI が `Cargo.toml` の `rust-version` を満たせず落ちる。

`.github/build-setup.yml` だけ古いままにすると、CI が test した toolchain と異なる toolchain で配布 archive をビルドする。変えたら `dist generate` で `release.yml` を作り直す。

5 箇所を更新したうえで image を作り直す。`release-tools` の image も `dev` の上に作るため、あわせて作り直す。

```sh
docker compose build dev
docker compose build release-tools
```

cloud session の environment cache は `Dockerfile` の変更を自動では追わない。MSRV を上げたら `.agents/scripts/cloud-session-setup.sh --print-stub` の出力を environment に貼り直す（`doc/guidelines/cloud-session-guidelines.md`）。

## タイムゾーン

判定結果は「今日」の解釈に依存する（`doc/design/business-day.md`）。`compose.yaml` は host の `TZ` と `BIZDATE_TZ` を container へ引き継ぐ。

host 側で `TZ` が未設定の場合、container は UTC で動く。タイムゾーン依存の挙動を確認するときは明示的に与える。

```sh
TZ=Asia/Tokyo docker compose run --rm dev cargo test
```

## Linux host での所有権

container は root で動く。repo は bind mount しているため、container 側から repo へ書き戻したファイルは host 側でも root 所有になる。macOS の Docker Desktop は uid をマップするため顕在化しないが、Linux host では実際に root 所有のファイルが repo に落ちる。

`target/` は named volume に逃がしてあるので主要な生成物は当たらない。当たるのは bind mount へ書くもので、`Cargo.lock` の更新、`--check` を付けない `cargo fmt` による `src/**` の書き戻し、`cargo add` / `cargo generate-lockfile` などがこれにあたる。

所有権が問題になった場合は host 側で `sudo chown` して戻す。`compose.yaml` に `user:` を足す対処は、named volume 側（`/usr/local/cargo` 配下）が root 所有のままとなり別の失敗を招くため、単独では成立しない。

## キャッシュ

cargo registry と `target/` は named volume に保持する。作り直したい場合は volume を消す。

```sh
docker compose down -v
```

## やらないこと

- host の `cargo` で実行した結果を、Compose 経由の検証結果として報告する。
- MSRV の 5 箇所（`Cargo.toml` の `rust-version`、`Dockerfile` の `FROM`、`compose.yaml` の image tag、`.github/workflows/ci.yml` の `RUST_TOOLCHAIN`、`.github/build-setup.yml` の toolchain）をずらす。
- container 内へ開発用の追加 component を場当たりで入れる。恒常的に必要なものは `Dockerfile` に書く。
