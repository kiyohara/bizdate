#!/bin/sh
# workflow が参照する第三者 action が、40 桁の commit SHA で固定されていることを確かめる。
# CI の lint job が呼ぶ。Compose でも同じ形で実行できる。
#
#   docker compose run --rm dev .github/scripts/check-action-pins.sh
#
# 対象は .github/workflows/ の *.yml と *.yaml (GitHub はどちらも workflow として読む) と、
# dist が release.yml の build job へ展開する .github/build-setup.yml。release.yml は dist の生成物で、
# SHA は dist-workspace.toml の github-action-commits と build-setup.yml から入る。
# この repo の reusable workflow (./ で始まる参照) は対象外。
# 検査する対象が 1 つも見つからない場合も失敗させ、空のまま通さない。
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

fail() {
    echo "check-action-pins: FAIL: $*" >&2
    exit 1
}

workflows=$(find .github/workflows -type f \( -name '*.yml' -o -name '*.yaml' \) | sort)
[ -n "$workflows" ] || fail "no workflow files under .github/workflows"
[ -f .github/build-setup.yml ] || fail ".github/build-setup.yml is missing"

# uses: の行を集める。grep は該当なしで 1、読めない file で 2 を返すため、どちらも失敗として扱う。
# shellcheck disable=SC2086 # workflow の path は空白を含まない
uses=$(grep -nE '^[[:space:]]*(-[[:space:]]+)?uses:' $workflows .github/build-setup.yml) ||
    fail "no uses: entries found"

# uses: の値は引用符付き (dist が build-setup の step を出す形) と引用符なしの両方がある。
# 入力は変数にあるため、ここでの || true は「残る行が無い」(grep の 1) だけを受け止める。
unpinned=$(printf '%s\n' "$uses" |
    grep -vE 'uses:[[:space:]]*"?\./' |
    grep -vE 'uses:[[:space:]]*"?[^@"[:space:]]+@[0-9a-f]{40}("|[[:space:]]|$)' || true)

if [ -n "$unpinned" ]; then
    echo "check-action-pins: FAIL: third-party actions not pinned to a commit SHA:" >&2
    printf '%s\n' "$unpinned" >&2
    exit 1
fi
checked=$(printf '%s\n' "$uses" | wc -l | tr -d ' ')
echo "check-action-pins: ok: $checked uses entries checked"
