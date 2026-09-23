#!/bin/sh
# dist が作った配布 archive を、対象 target の native runner 上で展開して検証する。
# release workflow の release-verify job が target ごとに呼ぶ。Compose でも、container と同じ target の
# archive に対して同じ形で実行できる (手順は doc/guidelines/development-command-guidelines.md)。
#
#   .github/scripts/verify-release-archive.sh <target> <archive と checksum のある dir> <dist plan の JSON>
#
# 確認すること (失敗したら exit 1):
#   - runner の OS / architecture が target と一致する (emulation で動いた結果を検証済みとしない)
#   - archive 名、checksum 名、archive 内の file の並びが dist plan の出力と一致する
#   - checksum file が sha256sum -c の形式で、archive と一致する
#   - 展開した README.md と LICENSE が repo の file と同一で、THIRD-PARTY-LICENSES.md が
#     check-third-party-licenses.sh を通る
#   - 展開した bizdate が実行可能で、platform-check.sh を通る (help / version / fixture 判定 /
#     タイムゾーンの採用経路。最低 glibc と動的リンク先を記録する)
# 記録すること (stdout と、設定されていれば GITHUB_STEP_SUMMARY): archive 名、sha256、archive 内の一覧
set -eu

target=${1:?usage: verify-release-archive.sh <target> <dir> <plan json>}
dist_dir=${2:?usage: verify-release-archive.sh <target> <dir> <plan json>}
plan=${3:?usage: verify-release-archive.sh <target> <dir> <plan json>}
root=$(cd "$(dirname "$0")/../.." && pwd)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    echo "verify-release-archive: FAIL: $*" >&2
    exit 1
}

echo "== native runner"
case $target in
    aarch64-apple-darwin) want="Darwin arm64" ;;
    aarch64-unknown-linux-gnu) want="Linux aarch64" ;;
    x86_64-unknown-linux-gnu) want="Linux x86_64" ;;
    *) fail "unknown target: $target" ;;
esac
have=$(uname -sm)
[ "$have" = "$want" ] || fail "runner is '$have', not '$want' for $target"
echo "ok: $have"

echo "== expectations from the dist plan"
# plan の artifacts から、この target の archive を 1 つだけ選ぶ。1 行目に archive 名、2 行目に
# checksum 名、3 行目以降に archive 内の path (archive 名のディレクトリからの相対) を出す。
python3 - "$plan" "$target" > "$work/expected.txt" <<'PY'
import json
import sys

plan_path, target = sys.argv[1], sys.argv[2]
with open(plan_path, encoding="utf-8") as f:
    manifest = json.load(f)
archives = [
    (name, artifact)
    for name, artifact in manifest["artifacts"].items()
    if artifact.get("kind") == "executable-zip" and artifact.get("target_triples") == [target]
]
if len(archives) != 1:
    sys.exit(f"expected exactly one archive for {target} in the plan, found {len(archives)}")
name, archive = archives[0]
print(name)
print(archive["checksum"])
for asset in archive["assets"]:
    print(asset["path"])
PY
archive=$(sed -n '1p' "$work/expected.txt")
checksum=$(sed -n '2p' "$work/expected.txt")
sed -n '3,$p' "$work/expected.txt" | sort > "$work/expected-assets.txt"
case $archive in
    *.tar.gz) dir=${archive%.tar.gz} ;;
    *) fail "the plan names '$archive', which is not a .tar.gz archive" ;;
esac
echo "archive: $archive"
echo "checksum: $checksum"

[ -f "$dist_dir/$archive" ] || fail "$archive is not in $dist_dir"
[ -f "$dist_dir/$checksum" ] || fail "$checksum is not in $dist_dir"

echo "== checksum"
# 形式は "<sha256> *<archive 名>"。file 名が archive を指していることも確かめる。
listed_name=$(awk 'NF { sub(/^\*/, "", $2); print $2 }' "$dist_dir/$checksum")
[ "$listed_name" = "$archive" ] || fail "$checksum names '$listed_name', not '$archive'"
if command -v sha256sum > /dev/null 2>&1; then
    (cd "$dist_dir" && sha256sum -c "$checksum") || fail "sha256sum -c $checksum failed"
    digest=$(cd "$dist_dir" && sha256sum "$archive" | awk '{ print $1 }')
else
    # macOS の runner には sha256sum が無い場合がある。shasum も同じ形式を読む。
    (cd "$dist_dir" && shasum -a 256 -c "$checksum") || fail "shasum -a 256 -c $checksum failed"
    digest=$(cd "$dist_dir" && shasum -a 256 "$archive" | awk '{ print $1 }')
fi

echo "== archive layout"
# archive は archive 名のディレクトリ 1 つに入る (distribution.md の「archive」)。
tar -tzf "$dist_dir/$archive" > "$work/listing.txt"
grep -v "^$dir/" "$work/listing.txt" > "$work/outside.txt" || true
[ ! -s "$work/outside.txt" ] || fail "entries outside $dir/: $(tr '\n' ' ' < "$work/outside.txt")"
sed -n "s|^$dir/||p" "$work/listing.txt" | grep -v '^$' | sort > "$work/actual-assets.txt"
if ! diff "$work/expected-assets.txt" "$work/actual-assets.txt" > "$work/diff.txt"; then
    cat "$work/diff.txt" >&2
    fail "files in $archive differ from the plan ('<' only in the plan, '>' only in the archive)"
fi
sed 's/^/  /' "$work/actual-assets.txt"

echo "== bundled files"
mkdir "$work/extract"
tar -xzf "$dist_dir/$archive" -C "$work/extract"
unpacked=$work/extract/$dir
for file in README.md LICENSE; do
    cmp -s "$unpacked/$file" "$root/$file" || fail "$file in the archive differs from the repository"
    echo "ok: $file is identical to the repository"
done
"$root/.github/scripts/check-third-party-licenses.sh" "$unpacked/THIRD-PARTY-LICENSES.md"

echo "== binary"
[ -x "$unpacked/bizdate" ] || fail "bizdate in the archive is not executable"
"$root/.github/scripts/platform-check.sh" "$unpacked/bizdate"

record=$work/record.md
{
    echo "### verify-release-archive: $target"
    echo '```'
    echo "archive: $archive"
    echo "sha256: $digest"
    echo "checksum file: $checksum (verified)"
    echo "files:"
    sed 's/^/  /' "$work/actual-assets.txt"
    echo '```'
} > "$record"
cat "$record"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    cat "$record" >> "$GITHUB_STEP_SUMMARY"
fi
