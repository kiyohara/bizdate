#!/bin/sh
# prepare-homebrew-formula.sh が書き出した Formula を、checkout 済みの tap (kiyohara/homebrew-tap) の
# Formula/bizdate.rb へ書き、commit して push する。release workflow の publish-homebrew job が呼ぶ。
#
#   .github/scripts/publish-homebrew-formula.sh [--dry-run] <Formula> <tap の working tree>
#
# 書かない条件 (exit 1。tap は変えない):
#   - Formula の version が prerelease (Homebrew は単一 version しか保持しない)
#   - tap の Formula の version のほうが新しい (patch release などで巻き戻さない)
#   - tap に同じ version があり、内容が異なる (同じ version を作り直さず、version を上げてやり直す)
#   - Formula/bizdate.rb 以外の file に変更が出る (既存の Casks/slapex.rb などを変えない)
# 何もせず exit 0: tap に同じ version が同じ内容で既にある (push 済みの job を再実行した場合)。
# --dry-run: commit までを行い、push しない (tap の working tree に commit が残る)。
set -eu

dry_run=false
if [ "${1:-}" = "--dry-run" ]; then
    dry_run=true
    shift
fi
formula=${1:?usage: publish-homebrew-formula.sh [--dry-run] <formula> <tap dir>}
tap_dir=${2:?usage: publish-homebrew-formula.sh [--dry-run] <formula> <tap dir>}
dest=Formula/bizdate.rb

fail() {
    echo "publish-homebrew-formula: FAIL: $*" >&2
    exit 1
}

version_of() {
    sed -n 's/^  version "\(.*\)"$/\1/p' "$1"
}

new=$(version_of "$formula")
[ -n "$new" ] || fail "no version line in $formula"
case $new in
    *-*) fail "$new is a prerelease; the tap keeps a single version and must not move to a prerelease" ;;
esac
[ -z "$(git -C "$tap_dir" status --porcelain)" ] || fail "the tap working tree is not clean"

if [ -f "$tap_dir/$dest" ]; then
    old=$(version_of "$tap_dir/$dest")
    [ -n "$old" ] || fail "no version line in the tap's $dest"
    # stable 同士の比較 (x.y.z)。tap 側が prerelease なら、同じ x.y.z の stable より古いとみなす。
    order=$(python3 - "$old" "$new" <<'PY'
import re
import sys


def key(v):
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(-.+)?", v)
    if not m:
        sys.exit(f"publish-homebrew-formula: FAIL: '{v}' is not a version this script compares")
    return (int(m[1]), int(m[2]), int(m[3]), m[4] is None)


old, new = (key(v) for v in sys.argv[1:3])
print("older" if old < new else "same" if old == new else "newer")
PY
)
    case $order in
        older) echo "the tap has bizdate $old; updating to $new" ;;
        same)
            if cmp -s "$formula" "$tap_dir/$dest"; then
                echo "the tap already has bizdate $new with the same content; nothing to do"
                exit 0
            fi
            fail "the tap has bizdate $new with different content; do not rebuild a published version, release a new one"
            ;;
        newer) fail "the tap has bizdate $old, newer than $new; not rolling the Formula back" ;;
        *) exit 1 ;;
    esac
else
    echo "the tap has no $dest; adding bizdate $new"
fi

mkdir -p "$tap_dir/Formula"
cp "$formula" "$tap_dir/$dest"
changed=$(git -C "$tap_dir" status --porcelain --untracked-files=all | cut -c4-)
[ "$changed" = "$dest" ] || fail "unexpected changes in the tap: $(echo "$changed" | tr '\n' ' ')"

git -C "$tap_dir" add "$dest"
git -C "$tap_dir" commit -q -m "bizdate $new"
git -C "$tap_dir" show --stat --format='%h %s' HEAD
if [ "$dry_run" = true ]; then
    echo "dry run: not pushing"
    exit 0
fi
git -C "$tap_dir" push -q origin HEAD
echo "pushed bizdate $new to the tap"
