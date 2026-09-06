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

よく使うもの:

```sh
docker compose run --rm dev cargo build
docker compose run --rm dev cargo test
docker compose run --rm dev cargo fmt --check
docker compose run --rm dev cargo clippy --all-targets -- -D warnings
```

ビルドしたバイナリも container の中で実行する。`target/` は named volume に置いてあり、host からは見えない。

```sh
docker compose run --rm dev ./target/debug/bizdate --version
```

複数コマンドをまとめる場合は `sh -c` を使う。

```sh
docker compose run --rm dev sh -c 'cargo build && ./target/debug/bizdate --version'
```

## image の更新

`Dockerfile` を変更したときと、MSRV を上げたときは image を作り直す。

```sh
docker compose build dev
```

## タイムゾーン

判定結果は「今日」の解釈に依存する（`doc/design/business-day.md`）。`compose.yaml` は host の `TZ` と `BIZDATE_TZ` を container へ引き継ぐ。

host 側で `TZ` が未設定の場合、container は UTC で動く。タイムゾーン依存の挙動を確認するときは明示的に与える。

```sh
TZ=Asia/Tokyo docker compose run --rm dev cargo test
```

## キャッシュ

cargo registry と `target/` は named volume に保持する。作り直したい場合は volume を消す。

```sh
docker compose down -v
```

## やらないこと

- host の `cargo` で実行した結果を、Compose 経由の検証結果として報告する。
- `compose.yaml` の image tag と `Cargo.toml` の `rust-version` をずらす。
- container 内へ開発用の追加 component を場当たりで入れる。恒常的に必要なものは `Dockerfile` に書く。
