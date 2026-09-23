#!/bin/sh
# THIRD-PARTY-LICENSES.md の生成に使う cargo-about を cargo install で入れる。version はここを正とする。
# release workflow の build job (.github/build-setup.yml) と Dockerfile の release-tools stage が呼ぶ。
# 引数はそのまま cargo install へ渡す (Dockerfile は --root で入れ先を決める)。
#
# cargo-about 0.9 系は bin を feature cli の後ろに置いている。--features cli を付けないと bin が入らず、
# `cargo about` が "no such command: about" になる。
set -eu

cargo install cargo-about --version 0.9.2 --locked --features cli "$@"
