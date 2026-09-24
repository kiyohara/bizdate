#!/bin/sh
# dist が生成した Homebrew Formula を検査し、起動テスト (test do) を足した Formula を書き出す。
# release workflow の release-verify (PR でも走る) が呼ぶ。publish-homebrew (tag push で tap へ書く) は、
# release-verify がこの script と install で確かめた Formula をそのまま書く。
#
#   .github/scripts/prepare-homebrew-formula.sh <dist の Formula> <archive と checksum のある dir> <dist plan の JSON> <出力先>
#
# 確認すること (失敗したら exit 1。出力先には何も書かない):
#   - class 名、desc、homepage、license が Cargo.toml と一致する
#   - version が dist plan の app_version と一致し、prerelease ではない
#   - OS / CPU の分岐が配布対象の 3 target と一致する (macOS は Apple Silicon だけ。Intel Mac の分岐が無い)
#   - 各分岐の url が v<version> の Release asset を指し、sha256 が同じ dir の <archive>.sha256 と一致する
#   - test do が無い (dist の template が test を持つようになったら、足し方を見直すため止める)
# 足すもの: test do (--version の出力と、third-party 表記が pkgshare にあること)。
# 祝日データは取得しない (利用者が fetch-holidays で用意する既存仕様のまま)。
set -eu

formula=${1:?usage: prepare-homebrew-formula.sh <formula> <dir> <plan json> <out>}
dist_dir=${2:?usage: prepare-homebrew-formula.sh <formula> <dir> <plan json> <out>}
plan=${3:?usage: prepare-homebrew-formula.sh <formula> <dir> <plan json> <out>}
out=${4:?usage: prepare-homebrew-formula.sh <formula> <dir> <plan json> <out>}
root=$(cd "$(dirname "$0")/../.." && pwd)

python3 - "$formula" "$dist_dir" "$plan" "$out" "$root/Cargo.toml" <<'PY'
import json
import os
import re
import sys

formula_path, dist_dir, plan_path, out_path, cargo_path = sys.argv[1:6]

REPOSITORY = "https://github.com/kiyohara/bizdate"
# 配布対象 (doc/design/distribution.md) と、Formula の分岐の対応。
TARGETS = {
    ("mac", "arm"): "aarch64-apple-darwin",
    ("linux", "arm"): "aarch64-unknown-linux-gnu",
    ("linux", "intel"): "x86_64-unknown-linux-gnu",
}
TEST_BLOCK = '''
  test do
    assert_match "bizdate #{version}", shell_output("#{bin}/bizdate --version")
    assert_path_exists pkgshare/"THIRD-PARTY-LICENSES.md"
  end
'''


def fail(message):
    sys.exit(f"prepare-homebrew-formula: FAIL: {message}")


with open(formula_path, encoding="utf-8") as f:
    text = f.read()
with open(plan_path, encoding="utf-8") as f:
    manifest = json.load(f)
# ubuntu-22.04 の python3 (3.10) には tomllib が無い。[package] の 1 行の文字列値だけを読む。
package = {}
with open(cargo_path, encoding="utf-8") as f:
    section = None
    for line in f:
        header = re.fullmatch(r"\[(.+)\]\s*", line)
        if header:
            section = header.group(1)
        elif section == "package":
            m = re.fullmatch(r'(\w+) = "(.*)"\s*', line)
            if m:
                package[m.group(1)] = m.group(2)

releases = [r for r in manifest.get("releases", []) if r.get("app_name") == "bizdate"]
if len(releases) != 1:
    fail(f"expected exactly one bizdate release in the plan, found {len(releases)}")
version = releases[0]["app_version"]
if "-" in version:
    fail(f"{version} is a prerelease; the Formula keeps a single version and must not move to a prerelease")


def single(pattern, what):
    found = re.findall(pattern, text, re.MULTILINE)
    if len(found) != 1:
        fail(f"expected exactly one {what}, found {len(found)}")
    return found[0]


if single(r'^class (\w+) < Formula$', "class line") != "Bizdate":
    fail("the class is not Bizdate")
checks = {
    "desc": (single(r'^  desc "(.*)"$', "desc"), package["description"]),
    "homepage": (single(r'^  homepage "(.*)"$', "homepage"), package["homepage"]),
    "license": (single(r'^  license "(.*)"$', "license"), package["license"]),
    "version": (single(r'^  version "(.*)"$', "version"), version),
}
for key, (actual, expected) in checks.items():
    if actual != expected:
        fail(f"{key} is '{actual}', not '{expected}'")
if re.search(r'^\s*test do\b', text, re.MULTILINE):
    fail("the generated Formula already has a test block; review how this script adds one")

# url / sha256 を、囲んでいる OS と CPU の分岐とともに読む。dist の template は
# if OS.mac? / if OS.linux? の中に if Hardware::CPU.arm? / .intel? を置き、その中に url と sha256 を書く。
os_name = cpu = None
found = {}
last_url = None
for line in text.splitlines():
    stripped = line.strip()
    if stripped in ("if OS.mac?", "if OS.linux?"):
        os_name, cpu = ("mac" if "mac" in stripped else "linux"), None
    elif stripped in ("if Hardware::CPU.arm?", "if Hardware::CPU.intel?"):
        cpu = "arm" if "arm" in stripped else "intel"
    elif stripped.startswith("url "):
        key = (os_name, cpu)
        if key in found:
            fail(f"more than one url for {key}")
        found[key] = {"url": re.fullmatch(r'url "(.*)"', stripped).group(1)}
        last_url = key
    elif stripped.startswith("sha256 "):
        if last_url is None or "sha256" in found[last_url]:
            fail("a sha256 line does not follow a url line")
        found[last_url]["sha256"] = re.fullmatch(r'sha256 "(.*)"', stripped).group(1)
    elif stripped.startswith("mirror "):
        fail("mirror lines are not expected")
    elif stripped == "def install":
        break

if set(found) != set(TARGETS):
    fail(f"the Formula branches on {sorted(found)}, not {sorted(TARGETS)}")
for key, target in TARGETS.items():
    archive = f"bizdate-{target}.tar.gz"
    want_url = f"{REPOSITORY}/releases/download/v{version}/{archive}"
    if found[key]["url"] != want_url:
        fail(f"url for {target} is '{found[key]['url']}', not '{want_url}'")
    if "sha256" not in found[key]:
        fail(f"no sha256 for {target}")
    checksum = os.path.join(dist_dir, f"{archive}.sha256")
    if not os.path.isfile(checksum):
        fail(f"{archive}.sha256 is not in {dist_dir}")
    with open(checksum, encoding="utf-8") as f:
        want_sha = f.read().split()[0]
    if found[key]["sha256"] != want_sha:
        fail(f"sha256 for {target} is {found[key]['sha256']}, not {want_sha} from {archive}.sha256")
    print(f"ok: {target}: {archive} sha256 {want_sha}")

# class の閉じる end の直前に test を足す。
body = text.rstrip("\n")
if not body.endswith("\nend"):
    fail("the Formula does not end with the class's end")
with open(out_path, "w", encoding="utf-8") as f:
    f.write(body[: -len("end")].rstrip("\n") + "\n" + TEST_BLOCK + "end\n")
print(f"ok: wrote {out_path} (bizdate {version}, with a test block)")
PY
