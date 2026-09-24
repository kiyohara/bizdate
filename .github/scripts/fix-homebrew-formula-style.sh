#!/bin/sh
# prepare-homebrew-formula.sh が書き出した Formula に brew style --fix をかけ、自動修正後に違反が
# 残らないことを確かめて、同じ file へ書き戻す。dist の template は brew style の自動修正できる違反
# (入れ子の if、定数の freeze、hash の整列など) を含むため。release workflow の release-verify が
# prepare の直後に呼ぶ。publish-homebrew はこの結果を install で確かめたものを tap へ書く。
#
#   .github/scripts/fix-homebrew-formula-style.sh <Formula>
#
# dist の builtin の publish job が除く 3 つの cop のうち、利用者が書く desc と homepage の 2 つだけを除く
# (FormulaAuditStrict は除かない)。自動修正できない違反が残れば exit 1 で、Formula は書き換えない。
set -eu

formula=${1:?usage: fix-homebrew-formula-style.sh <formula>}

fail() {
    echo "fix-homebrew-formula-style: FAIL: $*" >&2
    exit 1
}

# Ubuntu の runner は /home/linuxbrew に Homebrew が入っているが PATH に無い。
if ! command -v brew > /dev/null 2>&1 && [ -x /home/linuxbrew/.linuxbrew/bin/brew ]; then
    eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi
command -v brew > /dev/null 2>&1 || fail "brew is not on PATH"
export HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_ANALYTICS=1 HOMEBREW_NO_ENV_HINTS=1

# Homebrew は tap に無い Formula を style の対象にしない。一時的な tap に置く。
tap=bizdate-style/local
brew untap "$tap" > /dev/null 2>&1 || true
brew tap-new --no-git "$tap" > /dev/null
trap 'brew untap "$tap" > /dev/null 2>&1 || true' EXIT
tap_dir=$(brew --repository "$tap")
cp "$formula" "$tap_dir/Formula/bizdate.rb"

cops=FormulaAudit/Homepage,FormulaAudit/Desc
# --fix は違反を直しても exit 1 を返しうるため、結果は直後の検査で判定する。
brew style --formula --fix --except-cops "$cops" "$tap/bizdate" || true
brew style --formula --except-cops "$cops" "$tap/bizdate" || fail "brew style offenses remain after --fix"

cp "$tap_dir/Formula/bizdate.rb" "$formula"
echo "ok: $formula passes brew style"
