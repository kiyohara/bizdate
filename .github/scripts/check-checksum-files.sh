#!/bin/sh
# Release に上げる checksum file (archive ごとの .sha256 と、全 archive をまとめた sha256.sum) を検査し、
# 末尾の空行の再発を止める。dist 0.32.0 は checksum file の末尾に空行を足し、release-verify が
# normalize-checksum-files.sh で取り除いてから Release に上げる。経緯は
# doc/design/decision-log/0022-release-workflow.md の 2026-09-25 追記。
#
# release workflow の release-verify が、整えた後の build の artifact を host と同じ形 (artifacts-* を 1 つの
# dir にまとめる) で取得して呼ぶ。公開後確認では、取得した Release の asset と dist-manifest.json に対して
# Compose の dev service で同じ形で実行できる (doc/guidelines/release-guidelines.md の「公開後確認」)。
#
#   .github/scripts/check-checksum-files.sh <archive と checksum file のある dir> <dist plan の JSON>
#
# 確認すること (失敗したら exit 1):
#   - plan が挙げる checksum file (archive ごとの checksum と unified checksum) がそろっている
#   - 空行と CR が無く、file は改行で終わり、各行が "<sha256 (16 進 64 桁)> *<archive 名>" の形である
#   - archive ごとの checksum はその archive だけを、unified checksum は plan の全 archive を 1 行ずつ記す
#   - sha256sum -c --strict と shasum -a 256 -c --strict が、警告を出さずに通る
# 記録すること (stdout と、設定されていれば GITHUB_STEP_SUMMARY): 検査した file と記した archive の数
set -eu

dist_dir=${1:?usage: check-checksum-files.sh <dir> <plan json>}
plan=${2:?usage: check-checksum-files.sh <dir> <plan json>}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
    echo "check-checksum-files: FAIL: $*" >&2
    exit 1
}

# 空行を警告するのは shasum (macOS と同じ perl の実装) と coreutils 8.x の sha256sum で、coreutils 9.x の
# sha256sum は空行を黙って読み飛ばす。どちらの tool にも頼らず、空行は下で直接確かめる。
for tool in sha256sum shasum; do
    command -v "$tool" > /dev/null 2>&1 || fail "$tool is not installed"
done

echo "== expectations from the dist plan"
# 1 行に "<checksum file 名><TAB><その file が記すべき archive 名>" を出す。archive ごとの checksum は
# archive の checksum 欄から、unified checksum は kind から選ぶ。
python3 - "$plan" > "$work/expected.tsv" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    artifacts = json.load(f)["artifacts"]
archives = sorted(
    (name, artifact["checksum"])
    for name, artifact in artifacts.items()
    if artifact.get("kind") == "executable-zip" and artifact.get("checksum")
)
if not archives:
    sys.exit("the plan has no archive with a checksum file")
for name, checksum in archives:
    if artifacts.get(checksum, {}).get("kind") != "checksum":
        sys.exit(f"the plan names {checksum} as the checksum of {name}, but lists no such checksum artifact")
    print(f"{checksum}\t{name}")
unified = sorted(name for name, artifact in artifacts.items() if artifact.get("kind") == "unified-checksum")
if len(unified) != 1:
    sys.exit(f"expected exactly one unified checksum in the plan, found {len(unified)}")
for name, _ in archives:
    print(f"{unified[0]}\t{name}")
PY
cut -f 1 "$work/expected.tsv" | uniq > "$work/files.txt"
sed 's/^/  /' "$work/files.txt"

cr=$(printf '\r')
: > "$work/record.txt"
while read -r file; do
    echo "== $file"
    path=$dist_dir/$file
    [ -f "$path" ] || fail "$file is not in $dist_dir"
    [ -s "$path" ] || fail "$file is empty"

    # dist 0.32.0 は末尾に空行を 1 行足す。位置を問わず空行を認めない。
    if grep -n '^$' "$path" > "$work/empty.txt"; then
        fail "$file has an empty line (line $(cut -d : -f 1 "$work/empty.txt" | paste -s -d ' ' -))"
    fi
    if grep -q "$cr" "$path"; then
        fail "$file has a carriage return"
    fi
    [ "$(tail -c 1 "$path" | od -An -tx1 | tr -d ' \n')" = 0a ] || fail "$file does not end with a newline"
    if grep -Ev '^[0-9a-f]{64} \*[^/]+$' "$path" > "$work/malformed.txt"; then
        fail "$file has a line not in the form '<sha256> *<archive>': $(head -n 1 "$work/malformed.txt")"
    fi

    awk -F '\t' -v file="$file" '$1 == file { print $2 }' "$work/expected.tsv" | sort > "$work/want.txt"
    sed 's/^[0-9a-f]* \*//' "$path" | sort > "$work/listed.txt"
    if ! diff "$work/want.txt" "$work/listed.txt" > "$work/diff.txt"; then
        cat "$work/diff.txt" >&2
        fail "$file does not list exactly the archives in the plan ('<' only in the plan, '>' only in the file)"
    fi

    # 利用者が使う tool で、警告も含めて確かめる。--strict は形式の崩れた行で失敗させる。
    for check in "sha256sum -c --strict" "shasum -a 256 -c --strict"; do
        if ! (cd "$dist_dir" && $check "$file") > "$work/out.txt" 2> "$work/err.txt"; then
            cat "$work/out.txt" "$work/err.txt" >&2
            fail "$check $file failed"
        fi
        if [ -s "$work/err.txt" ]; then
            cat "$work/err.txt" >&2
            fail "$check $file printed a warning"
        fi
        echo "ok: $check ($(grep -c ': OK$' "$work/out.txt") OK)"
    done
    echo "$file: $(wc -l < "$work/want.txt" | tr -d ' ') archive(s), no empty line, sha256sum and shasum --strict ok" >> "$work/record.txt"
done < "$work/files.txt"

record=$work/record.md
{
    echo "### check-checksum-files"
    echo '```'
    cat "$work/record.txt"
    echo '```'
} > "$record"
cat "$record"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    cat "$record" >> "$GITHUB_STEP_SUMMARY"
fi
