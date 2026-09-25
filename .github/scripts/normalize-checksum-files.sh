#!/bin/sh
# dist が作る checksum file (archive ごとの .sha256 と、全 archive をまとめた sha256.sum) から、末尾の
# 空行を取り除く。dist 0.32.0 の write_checksum_file は各行を書いた後に改行をもう 1 つ足すため、file は
# 空行 1 行で終わり、macOS の shasum -a 256 -c は「1 line is improperly formatted」と警告する。
# release workflow の release-verify が、host が Release に上げる前の build の workflow artifact を
# 1 つずつ取得して呼ぶ。経緯は doc/design/decision-log/0022-release-workflow.md の 2026-09-25 追記。
#
#   .github/scripts/normalize-checksum-files.sh <artifact を取得した dir>
#
# dir 直下の *.sha256 と *.sum のうち、改行 2 つで終わる file から最後の改行を 1 つだけ取り除く。空行で
# 終わらない file (dist が直った場合) は変えない。形式と一致の検査は check-checksum-files.sh が行う。
# checksum file が 1 つも無い dir は、取得を誤ったとみなして exit 1 にする。
set -eu

dir=${1:?usage: normalize-checksum-files.sh <dir>}

fail() {
    echo "normalize-checksum-files: FAIL: $*" >&2
    exit 1
}

[ -d "$dir" ] || fail "$dir is not a directory"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

found=0
for file in "$dir"/*.sha256 "$dir"/*.sum; do
    [ -f "$file" ] || continue
    found=$((found + 1))
    name=${file##*/}
    if [ "$(tail -c 2 "$file" | od -An -tx1 | tr -d ' \n')" = 0a0a ]; then
        size=$(wc -c < "$file")
        head -c $((size - 1)) "$file" > "$work/normalized"
        cat "$work/normalized" > "$file"
        echo "normalized: $name (removed the trailing empty line)"
    else
        echo "unchanged: $name"
    fi
done
[ "$found" -gt 0 ] || fail "no checksum file (*.sha256, *.sum) in $dir"
