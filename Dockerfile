# 開発コマンドの実行環境。使い方は doc/guidelines/development-command-guidelines.md を参照する。
# tag は Cargo.toml の rust-version (MSRV) と揃える。
FROM rust:1.98-trixie

# 公式 rust image は minimal profile のため rustfmt と clippy を含まない。
# CI で回す fmt / clippy をローカルでも同じ toolchain で実行できるように追加する。
RUN rustup component add rustfmt clippy
