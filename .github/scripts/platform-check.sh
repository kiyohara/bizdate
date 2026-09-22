#!/bin/sh
# 配布対象の runner 上で release バイナリを起動し、配布仕様 (doc/design/distribution.md) が定める
# 実行時前提を確かめて記録する。CI の platform job から呼ぶが、Compose でも同じ形で実行できる。
#
#   docker compose run --rm dev sh -c 'cargo build --locked --release && .github/scripts/platform-check.sh target/release/bizdate'
#
# 確認すること (失敗したら exit 1):
#   - help / version が stdout に出て exit 0 になる
#   - fixture を使った first / last が yes / no を返し、--quiet は stdout だけを消す (exit 0 / 1)
#   - 不正な入力と祝日データの欠落が exit 2 になり、診断が stderr に出る
#   - タイムゾーンの採用経路 (--timezone / BIZDATE_TZ / local) ごとに、必要なデータが有れば成功し、
#     無ければ exit 2 になる。--version はデータが無くても成功する
# 記録すること (stdout と、設定されていれば GITHUB_STEP_SUMMARY):
#   - Linux: 参照する GLIBC symbol version の最大値 (= 最低 glibc) と動的リンク先
#   - macOS: 動的リンク先と署名の種別
set -eu

bin=${1:?usage: platform-check.sh <path to the bizdate binary>}
root=$(cd "$(dirname "$0")/../.." && pwd)
case $bin in
    /*) ;;
    *) bin=$(pwd)/$bin ;;
esac
test -x "$bin" || { echo "platform-check: $bin is not executable" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
home=$work/home
data=$work/data
mkdir -p "$home" "$data/bizdate/holidays"

# 既存 fixture のメタ行を今の時刻で付け直す。fixture の fetched_at は固定値で、1 年で期限切れになるため。
{
    printf '# bizdate-meta schema=1\n# fetched_at=%s\n# source_url=https://example.com/holidays.csv\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    grep -v '^#' "$root/tests/fixtures/holidays.csv"
} > "$data/bizdate/holidays/holidays.csv"

# 採用する zone のデータだけを持つ zoneinfo。jiff は TZDIR を開けた場合それだけを使う。
tzdir=$work/zoneinfo
mkdir -p "$tzdir/Asia"
cp /usr/share/zoneinfo/Asia/Tokyo "$tzdir/Asia/Tokyo"

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -n 1)

fail() {
    echo "platform-check: FAIL: $*" >&2
    exit 1
}

# 親の環境に依存しないよう、必要な変数だけを渡す。先頭の NAME=VALUE 引数は env が拾う。
observe() {
    set +e
    out=$(env -i PATH=/usr/bin:/bin HOME="$home" XDG_DATA_HOME="$data" "$@" 2>"$work/stderr")
    code=$?
    set -e
    err=$(cat "$work/stderr")
}

# exit code と stdout が一致し、stderr が空であること。
expect() {
    want_code=$1
    want_out=$2
    shift 2
    observe "$@"
    if [ "$code" -ne "$want_code" ] || [ "$out" != "$want_out" ] || [ -n "$err" ]; then
        fail "$* -> exit $code, stdout '$out', stderr '$err' (want exit $want_code, stdout '$want_out')"
    fi
    echo "ok (exit $code): $*"
}

# exit 2 で stdout が空で、stderr が診断を含むこと。
expect_error() {
    diagnostic=$1
    shift
    observe "$@"
    case $err in
        *"$diagnostic"*) ;;
        *) fail "$* -> stderr '$err' lacks '$diagnostic'" ;;
    esac
    if [ "$code" -ne 2 ] || [ -n "$out" ]; then
        fail "$* -> exit $code, stdout '$out' (want exit 2 and empty stdout)"
    fi
    echo "ok (exit 2): $*"
}

# help が stdout に出て exit 0 になること。
expect_usage() {
    observe "$@"
    case $out in
        *"Usage: bizdate"*) ;;
        *) fail "$* -> stdout '$out' lacks usage" ;;
    esac
    if [ "$code" -ne 0 ] || [ -n "$err" ]; then
        fail "$* -> exit $code, stderr '$err' (want exit 0 and empty stderr)"
    fi
    echo "ok (exit 0): $*"
}

echo "== help / version"
expect_usage "$bin" --help
expect_usage "$bin" first --help
expect_usage "$bin" last --help
expect_usage "$bin" fetch-holidays --help
expect 0 "bizdate $version" "$bin" --version

echo "== judgment with the fixture (2026-01-01 is a holiday; 2026-01-31 is Saturday)"
expect 0 yes "$bin" first --date 2026-01-02
expect 1 no "$bin" first --date 2026-01-01
expect 0 yes "$bin" last --date 2026-01-30
expect 1 no "$bin" last --date 2026-01-31
expect 0 "" "$bin" first --date 2026-01-02 --quiet
expect 1 "" "$bin" first --date 2026-01-01 --quiet
expect 1 no "$bin" first --date 2026-01-02 --day-off 2026-01-02
expect 0 yes "$bin" first --date 2026-01-05 --day-off 2026-01-02

echo "== errors (exit 2)"
expect_error "date" "$bin" first --date 2026-02-30
expect_error "date" "$bin" first --date 2026-02-30 --quiet
expect_error "Usage:" "$bin" first --unknown
expect_error "local holiday data is missing" XDG_DATA_HOME="$work/empty" "$bin" first --date 2026-01-02

echo "== time zone routes with the system tzdb"
expect 0 yes "$bin" first --date 2026-01-02 --timezone Asia/Tokyo
expect 0 yes BIZDATE_TZ=Asia/Tokyo "$bin" first --date 2026-01-02
expect 0 yes "$bin" first --date 2026-01-02
expect 0 yes TZ=Asia/Tokyo "$bin" first --date 2026-01-02
expect_error "unknown IANA time zone" "$bin" first --date 2026-01-02 --timezone Invalid/Zone
expect_error "unknown IANA time zone" BIZDATE_TZ=Invalid/Zone "$bin" first --date 2026-01-02
# local timezone を採用する経路は、--date があっても解決を省略しない。
expect_error "cannot resolve local time zone" TZ=Invalid/Zone "$bin" first --date 2026-01-02

echo "== time zone routes with a tzdb that holds only Asia/Tokyo"
expect 0 yes TZDIR="$tzdir" "$bin" first --date 2026-01-02 --timezone Asia/Tokyo
expect 0 yes TZDIR="$tzdir" BIZDATE_TZ=Asia/Tokyo "$bin" first --date 2026-01-02
expect_error "unknown IANA time zone" TZDIR="$tzdir" "$bin" first --date 2026-01-02 --timezone Europe/London
expect_error "unknown IANA time zone" TZDIR="$tzdir" BIZDATE_TZ=Europe/London "$bin" first --date 2026-01-02
expect 0 "bizdate $version" TZDIR="$tzdir" "$bin" --version

echo "== runtime requirements"
{
    echo "### platform-check: $(uname -sm)"
    echo '```'
    echo "binary: $bin"
    file "$bin"
    case $(uname -s) in
        Linux)
            echo "--- build environment ---"
            sed -n 's/^PRETTY_NAME=//p' /etc/os-release
            ldd --version | head -n 1
            echo "--- dynamic link (ldd) ---"
            ldd "$bin"
            echo "--- versioned symbol requirements (objdump -T) ---"
            objdump -T "$bin" | grep -oE '(GLIBC|GCC)_[0-9.]+' | sort -u -V
            echo "minimum glibc: $(objdump -T "$bin" | grep -o 'GLIBC_[0-9.]*' | sort -u -V | tail -n 1)"
            ;;
        Darwin)
            echo "--- build environment ---"
            sw_vers
            echo "--- dynamic link (otool -L) ---"
            otool -L "$bin"
            echo "--- code signature (codesign -dv) ---"
            codesign -dv "$bin" 2>&1 || true
            ;;
        *)
            echo "no recorder for this OS"
            ;;
    esac
    echo '```'
} | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"
