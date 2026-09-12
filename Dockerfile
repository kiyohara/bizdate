# 開発コマンドの実行環境。使い方は doc/guidelines/development-command-guidelines.md を参照する。
# tag は Cargo.toml の rust-version (MSRV) と揃える。
#
# BASE_REGISTRY は base image の取得元を差し替えるための build arg。local では既定の
# docker.io/library をそのまま使う。Claude Code on the web の cloud session では Docker Hub の
# blob 配信元に到達できないため、compose.cloud.yaml が許可リスト内の mirror を渡す
# (doc/guidelines/cloud-session-guidelines.md)。
ARG BASE_REGISTRY=docker.io/library
FROM ${BASE_REGISTRY}/rust:1.98-trixie

# 公式 rust image は minimal profile のため rustfmt と clippy を含まない。
# CI で回す fmt / clippy をローカルでも同じ toolchain で実行できるように追加する。
RUN rustup component add rustfmt clippy
