#!/bin/sh
# THIRD-PARTY-LICENSES.md の生成に使う cargo-about を、upstream の release が配る prebuilt binary で入れる。
# version と asset の sha256 はここを正とする。release workflow の build job (.github/build-setup.yml) と
# Dockerfile の release-tools stage が呼ぶ。
#
#   .github/scripts/install-cargo-about.sh [--root <dir>]
#
# <dir>/bin/cargo-about に置く。--root を省くと、cargo install の既定と同じ ${CARGO_HOME:-$HOME/.cargo} を使う
# (Dockerfile は --root で入れ先を決める)。
#
# asset は実行環境の OS / architecture から選び、ここに固定した sha256 と照合する。同じ release の .sha256 は
# 使わない (asset と一緒に差し替えられうるため。Dockerfile が dist を入れる方式と同じ)。対応外の platform と
# sha256 の不一致では、何も置かずに exit 1 で止まる。version を上げるときは 3 つの asset の sha256 を取り直す
# (doc/guidelines/development-command-guidelines.md の「tool と action の version を上げるとき」)。
set -eu

version=0.9.2
root=${CARGO_HOME:-$HOME/.cargo}

fail() {
    echo "install-cargo-about: FAIL: $*" >&2
    exit 1
}

usage() {
    echo "usage: install-cargo-about.sh [--root <dir>]" >&2
    exit 2
}

while [ $# -gt 0 ]; do
    case $1 in
        --root)
            [ $# -ge 2 ] || usage
            root=$2
            shift 2
            ;;
        *) usage ;;
    esac
done

# 配布対象の build runner (macOS arm64、Linux x86_64 / arm64) と、Compose の release-tools (Linux の
# x86_64 / arm64) に対応する。Linux は musl の静的 binary を使い、実行環境の glibc に依存しない。
platform=$(uname -sm)
case $platform in
    "Darwin arm64")
        target=aarch64-apple-darwin
        sha256=ae72f0df0c399a1e96336f696fa55b1b28679fd725632eba8cf8e4568467cc3e
        ;;
    "Linux x86_64")
        target=x86_64-unknown-linux-musl
        sha256=9099a59e820c38a68b9d65f300662a567d56562f9a10f6aa4c7e86c17c2566af
        ;;
    "Linux aarch64")
        target=aarch64-unknown-linux-musl
        sha256=af5169282fb6f84e13471493f405437e43ac517744c9ae12fbe2cdf0a6f0e5a8
        ;;
    *) fail "no prebuilt cargo-about $version is pinned for '$platform'" ;;
esac

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

name="cargo-about-${version}-${target}"
curl --proto '=https' --tlsv1.2 -fsSL -o "$work/$name.tar.gz" \
    "https://github.com/EmbarkStudios/cargo-about/releases/download/${version}/${name}.tar.gz"

# sha256sum が無い環境 (版の古い macOS など) では shasum を使う。
if command -v sha256sum > /dev/null 2>&1; then
    actual=$(sha256sum "$work/$name.tar.gz" | cut -d ' ' -f 1)
else
    actual=$(shasum -a 256 "$work/$name.tar.gz" | cut -d ' ' -f 1)
fi
[ "$actual" = "$sha256" ] || fail "sha256 of $name.tar.gz is $actual, not the pinned $sha256"
echo "install-cargo-about: ok: sha256 $actual $name.tar.gz"

tar -xzf "$work/$name.tar.gz" -C "$work" "$name/cargo-about"
mkdir -p "$root/bin"
install -m 0755 "$work/$name/cargo-about" "$root/bin/cargo-about"
"$root/bin/cargo-about" --version
