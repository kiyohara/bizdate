#!/bin/sh
# workflow が参照する第三者 action が、40 桁の commit SHA で固定されていることを確かめる。
# CI の lint job が呼ぶ。Compose でも同じ形で実行できる。
#
#   docker compose run --rm dev .github/scripts/check-action-pins.sh
#
# 対象は .github/workflows/*.yml と、dist が release.yml の build job へ展開する .github/build-setup.yml。
# release.yml は dist の生成物で、SHA は dist-workspace.toml の github-action-commits と build-setup.yml
# から入る。この repo の reusable workflow (./ で始まる参照) は対象外。
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

# uses: の値は引用符付き (dist が build-setup の step を出す形) と引用符なしの両方がある。
unpinned=$(grep -nE '^[[:space:]]*(-[[:space:]]+)?uses:' .github/workflows/*.yml .github/build-setup.yml |
    grep -vE 'uses:[[:space:]]*"?\./' |
    grep -vE 'uses:[[:space:]]*"?[^@"[:space:]]+@[0-9a-f]{40}("|[[:space:]]|$)' || true)

if [ -n "$unpinned" ]; then
    echo "check-action-pins: FAIL: third-party actions not pinned to a commit SHA:" >&2
    printf '%s\n' "$unpinned" >&2
    exit 1
fi
checked=$(grep -hE '^[[:space:]]*(-[[:space:]]+)?uses:' .github/workflows/*.yml .github/build-setup.yml | wc -l | tr -d ' ')
echo "check-action-pins: ok: $checked uses entries checked"
