#!/bin/sh
# release workflow の release-verify が、build の workflow artifact の checksum file を整える前に呼ぶ。
# release-verify は build-global-artifacts (sha256.sum を作る job) と並行して走り、needs で待てない。
# この run の artifact の一覧を GitHub の API で読み、artifacts-build-global が上がるまで待つ。
# 待つのは、最新の artifacts-plan-dist-manifest (plan job が上げる) より後に作られたものである。
# re-run で build-global-artifacts が走り直すとき、前の attempt の artifact を拾わないため。
# 経緯は doc/design/decision-log/0022-release-workflow.md の 2026-09-25 追記。
#
#   .github/scripts/wait-for-build-artifacts.sh [待つ上限の秒数 (既定 900)] >> "$GITHUB_OUTPUT"
#
# 上がったら、checksum file を含む build の artifact (artifacts-build-*) の名前を JSON の配列にして、
# "names=<配列>" の 1 行を stdout に出す。待つ間の経過は stderr に出す。上限までに上がらなければ exit 1。
# 環境変数: GH_TOKEN (actions: read の権限)、GITHUB_REPOSITORY と GITHUB_RUN_ID (Actions が設定する)。
set -eu

limit=${1:-900}
: "${GITHUB_REPOSITORY:?}" "${GITHUB_RUN_ID:?}"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    echo "wait-for-build-artifacts: FAIL: $*" >&2
    exit 1
}

# 名前ごとの最新の作成時刻を返す (時刻は UTC の ISO 8601 で、文字列の順が時刻の順になる)。
latest() {
    awk -F '\t' -v name="$1" '$1 == name { print $2 }' "$work/artifacts.tsv" | sort | tail -n 1
}

start=$(date +%s)
while :; do
    if gh api --paginate "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/artifacts?per_page=100" \
        --jq '.artifacts[] | select(.expired | not) | [.name, .created_at] | @tsv' > "$work/artifacts.tsv"; then
        plan_at=$(latest artifacts-plan-dist-manifest)
        global_at=$(latest artifacts-build-global)
        if [ -n "$plan_at" ] && [ -n "$global_at" ] && [ "$global_at" != "$plan_at" ] &&
            [ "$(printf '%s\n%s\n' "$plan_at" "$global_at" | sort | tail -n 1)" = "$global_at" ]; then
            break
        fi
    else
        echo "listing the artifacts failed; retrying" >&2
    fi
    elapsed=$(($(date +%s) - start))
    [ "$elapsed" -lt "$limit" ] || fail "no artifacts-build-global newer than the plan after ${elapsed}s"
    echo "waiting for artifacts-build-global (${elapsed}s)" >&2
    sleep 10
done
echo "artifacts-build-global: created $global_at (plan: $plan_at)" >&2

awk -F '\t' '$1 ~ /^artifacts-build-/ { print $1 }' "$work/artifacts.tsv" | sort -u > "$work/names.txt"
sed 's/^/  /' "$work/names.txt" >&2
python3 - "$work/names.txt" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    names = [line.strip() for line in f if line.strip()]
print("names=" + json.dumps(names, separators=(",", ":")))
PY
