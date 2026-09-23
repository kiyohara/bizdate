#!/bin/sh
# 生成した THIRD-PARTY-LICENSES.md が配布対象の依存と対応していることを確かめる。
# release workflow の release-prepare job が生成の直後に呼ぶ。Compose でも同じ形で実行できる。
#
#   docker compose run --rm release-tools sh -c 'cargo about generate --locked --fail --output-file THIRD-PARTY-LICENSES.md about.hbs && .github/scripts/check-third-party-licenses.sh THIRD-PARTY-LICENSES.md'
#
# 確認すること (失敗したら exit 1):
#   - 表の crate と version が、dist-workspace.toml の配布対象 target すべての依存 (normal と build。
#     bizdate 自身は除く) を合わせた集合と過不足なく一致する。依存は cargo tree で数える
#   - 表のすべての crate に、ライセンス本文が 1 つ以上ある
#   - 配布仕様 (doc/design/distribution.md) が名指しする条件の本文が、該当 crate の分として載っている
set -eu

file=${1:?usage: check-third-party-licenses.sh <path to THIRD-PARTY-LICENSES.md>}
root=$(cd "$(dirname "$0")/../.." && pwd)
test -s "$file" || { echo "check-third-party-licenses: $file is missing or empty" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    echo "check-third-party-licenses: FAIL: $*" >&2
    exit 1
}

[ "$(sed -n '1p' "$file")" = "# Third-Party Licenses" ] || fail "$file does not start with the expected heading"

# 配布対象 target は dist の設定を正とする。1 行の配列で書いてある前提で読む。
targets=$(sed -n 's/^targets = \[\(.*\)\]$/\1/p' "$root/dist-workspace.toml" | tr -d '"' | sed 's/, */ /g')
[ -n "$targets" ] || fail "no targets found in dist-workspace.toml"

# 期待する集合: 各 target の依存を cargo tree で数えて合わせる。表記は "<crate> <version>"。
for target in $targets; do
    cargo tree --locked --manifest-path "$root/Cargo.toml" -e normal,build --target "$target" \
        --prefix none --format '{p}' > "$work/tree-$target.txt"
done
cat "$work"/tree-*.txt | sed 's/ (.*$//' | grep -v '^bizdate v' | sed 's/ v\([^ ]*\)$/ \1/' | sort -u > "$work/expected.txt"
[ -s "$work/expected.txt" ] || fail "cargo tree listed no dependencies"

# 表の行: | crate | version | license |
sed -n 's/^| \([^ |]*\) | \([^ |]*\) | [^|]* |$/\1 \2/p' "$file" | grep -v '^crate version$' | sort > "$work/listed.txt"
if ! diff "$work/expected.txt" "$work/listed.txt" > "$work/diff.txt"; then
    cat "$work/diff.txt" >&2
    fail "crates in $file differ from the dependencies of: $targets ('<' only in cargo tree, '>' only in the file)"
fi

# 本文の節: "### <name> (<id>)" の後の "Used by: <crate> <version>, ..." から (crate, id) の組を作る。
awk '
    /^### / { id = $NF; gsub(/[()]/, "", id); next }
    /^Used by: / {
        sub(/^Used by: /, "")
        n = split($0, used, ", ")
        for (i = 1; i <= n; i++) {
            split(used[i], parts, " ")
            print parts[1], parts[2], id
        }
    }
' "$file" | sort -u > "$work/texts.txt"

cut -d ' ' -f 1,2 "$work/texts.txt" | sort -u > "$work/with-text.txt"
if ! diff "$work/listed.txt" "$work/with-text.txt" > "$work/diff.txt"; then
    cat "$work/diff.txt" >&2
    fail "some crates in $file have no license text ('<' listed without text, '>' text for an unlisted crate)"
fi

# 自前の LICENSE だけでは満たせない条件を持つ依存 (distribution.md の「third-party ライセンス表記」)。
for pair in "webpki-roots CDLA-Permissive-2.0" "encoding_rs BSD-3-Clause" "ring ISC" "ring Apache-2.0"; do
    crate=${pair% *}
    id=${pair#* }
    grep -q "^$crate [^ ]* $id\$" "$work/texts.txt" || fail "$file lacks the $id text for $crate"
done

echo "check-third-party-licenses: ok: $(wc -l < "$work/listed.txt" | tr -d ' ') crates for $targets"
