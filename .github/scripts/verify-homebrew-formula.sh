#!/bin/sh
# prepare-homebrew-formula.sh が書き出した Formula で、公開前の archive を Homebrew に install し、
# Formula の test と配置を確かめる。release workflow の release-verify job が native runner ごとに呼ぶ。
# 公開 tap からの brew install / upgrade の確認ではない (Formula の url を、この run の archive へ差し替える)。
#
#   .github/scripts/verify-homebrew-formula.sh <target> <Formula> <archive と checksum のある dir>
#
# 確認すること (失敗したら exit 1):
#   - runner の OS / architecture が target と一致する
#   - brew style を通る (dist の builtin publish job と同じく、利用者が書く desc と homepage の cop は除く)
#   - 一時的な local tap から install でき、Formula が選んだ archive が target のものである
#   - brew test が通る
#   - bizdate が bin に、README.md と LICENSE が doc に、THIRD-PARTY-LICENSES.md が pkgshare に入り、
#     archive の file と同一である
# 最後に install した formula と local tap を消す。
set -eu

target=${1:?usage: verify-homebrew-formula.sh <target> <formula> <dir>}
formula=${2:?usage: verify-homebrew-formula.sh <target> <formula> <dir>}
dist_dir=$(cd "${3:?usage: verify-homebrew-formula.sh <target> <formula> <dir>}" && pwd)

fail() {
    echo "verify-homebrew-formula: FAIL: $*" >&2
    exit 1
}

case $target in
    aarch64-apple-darwin) want="Darwin arm64" ;;
    aarch64-unknown-linux-gnu) want="Linux aarch64" ;;
    x86_64-unknown-linux-gnu) want="Linux x86_64" ;;
    *) fail "unknown target: $target" ;;
esac
have=$(uname -sm)
[ "$have" = "$want" ] || fail "runner is '$have', not '$want' for $target"

command -v brew > /dev/null 2>&1 || fail "brew is not on PATH"
export HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_ANALYTICS=1 HOMEBREW_NO_ENV_HINTS=1 HOMEBREW_NO_INSTALL_CLEANUP=1
brew --version | head -1

archive=bizdate-$target.tar.gz
[ -f "$dist_dir/$archive" ] || fail "$archive is not in $dist_dir"

work=$(mktemp -d)
tap=bizdate-verify/local
cleanup() {
    brew uninstall --formula "$tap/bizdate" > /dev/null 2>&1 || true
    brew untap "$tap" > /dev/null 2>&1 || true
    rm -rf "$work"
}
trap cleanup EXIT

# Homebrew は tap に無い Formula を style の対象にも install の対象にもしない。一時的な tap に置く。
brew tap-new --no-git "$tap" > /dev/null
tap_dir=$(brew --repository "$tap")

echo "== brew style"
# url を差し替える前の、tap へ書くものと同じ Formula を対象にする。
cp "$formula" "$tap_dir/Formula/bizdate.rb"
brew style --formula --except-cops FormulaAudit/Homepage,FormulaAudit/Desc "$tap/bizdate"

echo "== install from a local tap"
# url をこの run の archive (file://) へ差し替える。sha256 は変えないため、Formula の checksum と archive の
# 一致も install で確かめる。
sed "s|https://github.com/kiyohara/bizdate/releases/download/v[^/]*/|file://$dist_dir/|" "$formula" \
    > "$tap_dir/Formula/bizdate.rb"
brew install --formula "$tap/bizdate"

prefix=$(brew --prefix "$tap/bizdate")
cellar=$(brew --cellar "$tap/bizdate")
version=$(sed -n 's/^  version "\(.*\)"$/\1/p' "$formula")
keg=$cellar/$version
[ -d "$keg" ] || fail "$keg is not installed"
installed_from=$(brew info --json=v1 "$tap/bizdate" | python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["urls"]["stable"]["url"])')
case $installed_from in
    */"$archive") echo "ok: the Formula selects $archive" ;;
    *) fail "the Formula selects '$installed_from', not $archive" ;;
esac

echo "== brew test"
brew test "$tap/bizdate"

echo "== installed files"
mkdir "$work/extract"
tar -xzf "$dist_dir/$archive" -C "$work/extract"
unpacked=$work/extract/bizdate-$target
cmp -s "$keg/bin/bizdate" "$unpacked/bizdate" || fail "bin/bizdate differs from the archive"
"$prefix/bin/bizdate" --version
for file in README.md LICENSE; do
    # Homebrew は doc として keg の root に置く。
    cmp -s "$keg/$file" "$unpacked/$file" || fail "$file in the keg differs from the archive"
    echo "ok: $file"
done
pkgshare=$(brew --prefix)/share/bizdate
cmp -s "$pkgshare/THIRD-PARTY-LICENSES.md" "$unpacked/THIRD-PARTY-LICENSES.md" \
    || fail "$pkgshare/THIRD-PARTY-LICENSES.md is missing or differs from the archive"
echo "ok: $pkgshare/THIRD-PARTY-LICENSES.md"

record=$work/record.md
{
    echo "### verify-homebrew-formula: $target"
    echo '```'
    brew --version | head -1
    echo "formula: bizdate $version (local tap, url replaced with this run's archive)"
    echo "selected: $archive"
    echo "installed: $keg"
    echo "third-party notice: $pkgshare/THIRD-PARTY-LICENSES.md"
    echo "brew style: ok, brew test: ok"
    echo '```'
} > "$record"
cat "$record"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    cat "$record" >> "$GITHUB_STEP_SUMMARY"
fi
