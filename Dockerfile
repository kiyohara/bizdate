# 開発コマンドの実行環境。使い方は doc/guidelines/development-command-guidelines.md を参照する。
# tag は Cargo.toml の rust-version (MSRV) と揃える。
#
# BASE_REGISTRY は base image の取得元を差し替えるための build arg。local では既定の
# docker.io/library をそのまま使う。Claude Code on the web の cloud session では Docker Hub の
# blob 配信元に到達できないため、compose.cloud.yaml が許可リスト内の mirror を渡す
# (doc/guidelines/cloud-session-guidelines.md)。
ARG BASE_REGISTRY=docker.io/library
FROM ${BASE_REGISTRY}/rust:1.98-trixie AS dev

# 公式 rust image は minimal profile のため rustfmt と clippy を含まない。
# CI で回す fmt / clippy をローカルでも同じ toolchain で実行できるように追加する。
RUN rustup component add rustfmt clippy
# Dockerfile は toolchain を用意するだけでソースを COPY しない (ソースは compose の bind mount)。

# 配布成果物の生成と確認に使う dist と cargo-about を dev に足した image。compose.yaml の
# release-tools service が使う。cloud session の hook は dev image を同期 build するため、dev には
# 入れない (doc/guidelines/development-command-guidelines.md の「配布成果物の生成と確認」)。
FROM dev AS release-tools
# dist は公開されている Linux 向け binary を、release の公開 checksum と照合して入れる。
# version は dist-workspace.toml の cargo-dist-version と揃える (ずれると dist generate が止まる)。
RUN set -eu; \
    version=0.32.0; \
    case "$(uname -m)" in \
        aarch64) arch=aarch64; sha256=d29bcffeb3f8b0c517b4ce0dd2470926ed5cb0bb29d78c6bdd5f88d76ee14a6a ;; \
        x86_64) arch=x86_64; sha256=eb52f9fae0d0506774e9f1801c1168f87fa2c87a45e2d64d3ae7c89401929946 ;; \
        *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;; \
    esac; \
    work=$(mktemp -d); \
    archive="cargo-dist-${arch}-unknown-linux-gnu.tar.xz"; \
    curl --proto '=https' --tlsv1.2 -fsSL -o "$work/$archive" \
        "https://github.com/axodotdev/cargo-dist/releases/download/v${version}/${archive}"; \
    echo "${sha256}  $work/$archive" | sha256sum -c -; \
    tar -xJf "$work/$archive" -C "$work"; \
    install -m 0755 "$work/cargo-dist-${arch}-unknown-linux-gnu/dist" /usr/local/bin/dist; \
    rm -rf "$work"; \
    dist --version
# cargo-about は release workflow の build job と同じ script で、upstream の prebuilt を sha256 と照合して
# 入れる (version と sha256 は script を正とする)。/usr/local/bin/cargo-about に置く。
COPY .github/scripts/install-cargo-about.sh /tmp/install-cargo-about.sh
RUN /tmp/install-cargo-about.sh --root /usr/local \
    && rm /tmp/install-cargo-about.sh \
    && cargo about --version
