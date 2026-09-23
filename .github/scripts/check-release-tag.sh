#!/bin/sh
# release workflow を起動した tag が、配布仕様 (doc/design/distribution.md の「tag」) の形であることを
# 確かめる。tag は v<Cargo.toml の version> に限る。dist は version さえ一致すれば v の無い形や
# package 名付きの形 (bizdate-v0.1.0 など) も受け付けるため、ここで形を絞る。
# release workflow の release-verify job が tag push のときに呼ぶ。Compose でも同じ形で実行できる。
#
#   docker compose run --rm dev .github/scripts/check-release-tag.sh v0.1.0
set -eu

tag=${1:?usage: check-release-tag.sh <tag>}
root=$(cd "$(dirname "$0")/../.." && pwd)

# platform-check.sh と同じく、Cargo.toml の [package] にある最初の version 行を読む。
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -n 1)
[ -n "$version" ] || { echo "check-release-tag: FAIL: no version found in Cargo.toml" >&2; exit 1; }

if [ "$tag" != "v$version" ]; then
    echo "check-release-tag: FAIL: tag '$tag' is not 'v$version' (the version in Cargo.toml)" >&2
    exit 1
fi
echo "check-release-tag: ok: $tag matches the version in Cargo.toml"
