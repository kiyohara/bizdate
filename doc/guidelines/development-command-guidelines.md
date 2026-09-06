# 開発コマンド実行ルール

この文書は、bizdate リポジトリで `cargo` などの開発コマンドを実行するときの共通正本である。AI agent と人間の両方がこのルールに従う。

toolchain と依存クレートの選定経緯は `doc/design/decision-log/0014-dependency-crates-and-toolchain.md` を参照する。

## 原則

- 開発コマンドは host OS 上で直接実行せず、Compose 経由で実行する。
- 実行環境は repo root の `compose.yaml` と `Dockerfile` を正とする。開発用 service 名は `dev` とする。
- host に Rust toolchain を入れることは禁止しない。ただし、検証結果として記録・報告する実行は Compose 経由のものとする。host の toolchain は MSRV や component 構成が揃っている保証が無い。

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

`Dockerfile` を変更したときは image を作り直す。

```sh
docker compose build dev
```

## MSRV を上げるとき

MSRV の値は 3 箇所に現れる。上げるときは 3 つを同時に更新する。

| 箇所 | 値 | 役割 |
|---|---|---|
| `Cargo.toml` | `rust-version` | crate が要求する最小 Rust version |
| `Dockerfile` | `FROM rust:<MSRV>-trixie` | container の toolchain を実際に決める |
| `compose.yaml` | `image: bizdate-dev:<MSRV>` | build した image に付ける local tag 名 |

`compose.yaml` の tag はビルド結果に影響しないが、ずれると tag が実態を偽る。`Dockerfile` の `FROM` だけ古いまま `rust-version` を上げると、build が MSRV エラーで落ちる。

3 箇所を更新したうえで image を作り直す。

```sh
docker compose build dev
```

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
- MSRV の 3 箇所（`Cargo.toml` の `rust-version`、`Dockerfile` の `FROM`、`compose.yaml` の image tag）をずらす。
- container 内へ開発用の追加 component を場当たりで入れる。恒常的に必要なものは `Dockerfile` に書く。
