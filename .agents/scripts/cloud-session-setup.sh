#!/usr/bin/env bash
# cloud-session-setup: Claude Code on the web の cloud session で Compose の実行環境を整える。
#
# 背景:
#   cloud session の sandbox には docker CLI / dockerd / compose が入っているが daemon は
#   起動していない。sandbox の Rust toolchain は MSRV と一致しないため、開発コマンドは
#   通常どおり `docker compose run --rm dev cargo ...` で実行する
#   (doc/guidelines/development-command-guidelines.md)。この script は daemon を起動し、
#   開発用 image が無ければ build して、その前提を満たす。cloud 固有の差分 (container を
#   agent proxy 経由にする、base image を mirror から取る) は compose.cloud.yaml に置き、
#   COMPOSE_FILE で重ねる。
#
# 呼び出し元:
#   - .claude/settings.json の SessionStart hook (startup|resume)。既定 mode。
#     CLAUDE_CODE_REMOTE=true 以外では何も出力せず exit 0 する。
#   - environment の setup script (claude.ai/code の UI 設定): `--provision`。
#     `--print-stub` が生成した stub を UI に貼る。setup script は agent proxy が立つ前に
#     走るため container 内から外へは出られない。そこで base image の pull (重い部分) と
#     state file の記録だけを行い、薄い最終層の build は hook に任せる。
#
# 安全策:
#   - 冪等。何度実行しても同じ状態に収束する。
#   - 非対話。secret を扱わない。環境変数の値や log 全文を stdout に出さない。
#   - `--doctor` 以外は失敗しても exit 0 とする。setup script が非 0 で終わると session が
#     起動せず、SessionStart hook の stdout は agent の context に入るため、状況は短く
#     stdout に出す。
#   - `set -e` は使わない。外部コマンドの失敗は個別に扱う。
#
# 使い方:
#   .agents/scripts/cloud-session-setup.sh [--force | --provision | --doctor | --print-stub]
#
# 詳細: doc/guidelines/cloud-session-guidelines.md

set -uo pipefail

self="cloud-session-setup"
mode="hook"

usage() {
  cat <<'USAGE'
usage: cloud-session-setup.sh [--force | --provision | --doctor | --print-stub]

Claude Code on the web の cloud session で Docker daemon を起動し、開発用 image を用意する。
引数なしのときは SessionStart hook としての動作で、CLAUDE_CODE_REMOTE=true 以外では何もしない。

options:
  --force       CLAUDE_CODE_REMOTE=true でなくても hook と同じ動作を行う
  --provision   environment の setup script 用。base image を pull し、image の build を試み、
                state file を書き、自分で起動した daemon を止める
  --doctor      daemon / image / environment cache の状態を表示し、cache が古ければ exit 1
  --print-stub  environment の setup script に貼る stub を生成する
  -h, --help    この help を表示する
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --force) mode="force"; shift ;;
    --provision) mode="provision"; shift ;;
    --doctor) mode="doctor"; shift ;;
    --print-stub) mode="print-stub"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "$self: unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

# guard 1: local の Claude Code でも hook は走る。cloud 以外では無出力で抜ける。
if [ "$mode" = "hook" ] && [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# repo root。hook からは CLAUDE_PROJECT_DIR が渡る。setup script からの呼び出しや手動実行では
# script 自身の位置から解決する。
script_path="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
if [ -n "${CLAUDE_PROJECT_DIR:-}" ] && [ -f "${CLAUDE_PROJECT_DIR}/compose.yaml" ]; then
  repo_root="$CLAUDE_PROJECT_DIR"
else
  repo_root="$(cd "$(dirname "$script_path")/../.." && pwd)"
fi
cd "$repo_root" || { echo "$self: repo root に移動できない" >&2; exit 0; }

compose_files="compose.yaml:compose.cloud.yaml"
state_dir="/opt/bizdate-cloud"
state_file="$state_dir/state"
log_file="${TMPDIR:-/tmp}/$self.log"
daemon_log="/var/log/dockerd-cloud-session.log"
# cloud VM の既定 clone 先。stub の fallback にだけ使う (個人環境の path ではない)。
default_clone_dir="/home/user/bizdate"

started_daemon=0

say() { echo "$self: $*"; }

compose() { COMPOSE_FILE="$compose_files" docker compose "$@"; }

# guard 2: 期待する sandbox の構成か。dockerd が無い環境 (macOS など) では何もしない。
is_cloud_sandbox() {
  [ "$(uname -s)" = "Linux" ] \
    && command -v docker >/dev/null 2>&1 \
    && command -v dockerd >/dev/null 2>&1
}

# environment cache の再構築は setup script のテキストが変わったときにしか起きない。
# image の内容を決める入力の digest を stub に埋め、貼り直しでテキストが変わるようにする。
# script 自身は stdin から読み、clone 先の path が digest に混ざらないようにする。
inputs_digest() {
  {
    sha256sum < "$script_path"
    cat "$repo_root/Dockerfile" "$repo_root/compose.yaml" "$repo_root/compose.cloud.yaml"
  } 2>/dev/null | sha256sum | cut -d' ' -f1
}

recorded_digest() {
  [ -f "$state_file" ] && sed -n 's/^inputs_sha256=//p' "$state_file"
}

daemon_ready() { docker info >/dev/null 2>&1; }

# /run は root filesystem 上にあり、daemon 起動中に snapshot されると pid file と socket が
# 残る。生きていない pid の file だけを消す。
remove_stale_runtime_files() {
  local f pid
  for f in /var/run/docker.pid /var/run/docker-ssd.pid /run/containerd/containerd.pid; do
    [ -f "$f" ] || continue
    pid="$(cat "$f" 2>/dev/null || true)"
    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
      rm -f "$f"
    fi
  done
  if [ -S /var/run/docker.sock ] && ! daemon_ready; then
    rm -f /var/run/docker.sock
  fi
  return 0
}

# pid file の process が生きているのに API が応答しないのは、起動中か停止中である。
# provision の停止処理の直後や前 session の終了直後に当たるため、少し待ってから判断する。
wait_daemon_transition() {
  local i pid
  pid="$(cat /var/run/docker.pid 2>/dev/null || true)"
  { [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; } || return 0
  for i in $(seq 1 20); do
    daemon_ready && return 0
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 1
  done
  return 0
}

# `service docker start` は使わない。sandbox では init script の ulimit 変更が
# Operation not permitted で失敗し、daemon が起動しないため dockerd を直接起動する。
start_daemon() {
  local i
  if daemon_ready; then
    say "dockerd: 起動済み"
    return 0
  fi
  wait_daemon_transition
  if daemon_ready; then
    say "dockerd: 起動済み"
    return 0
  fi
  remove_stale_runtime_files
  nohup dockerd >>"$daemon_log" 2>&1 </dev/null &
  for i in $(seq 1 30); do
    if daemon_ready; then
      started_daemon=1
      say "dockerd: 起動した (${i}s)"
      return 0
    fi
    sleep 1
  done
  say "dockerd: 30 秒以内に起動しなかった"
  tail -n 5 "$daemon_log" 2>/dev/null | cut -c1-200 | sed "s/^/$self:   /"
  return 1
}

stop_daemon() {
  local i pid
  [ "$started_daemon" -eq 1 ] || return 0
  pid="$(cat /var/run/docker.pid 2>/dev/null || true)"
  if [ -n "$pid" ]; then
    kill -TERM "$pid" 2>/dev/null || true
  else
    pkill -TERM -x dockerd 2>/dev/null || true
  fi
  # API が閉じた後も process の終了処理が続く。process が消えるまで待ち、pid file を残さない。
  for i in $(seq 1 30); do
    if [ -n "$pid" ]; then
      kill -0 "$pid" 2>/dev/null || break
    else
      pgrep -x dockerd >/dev/null 2>&1 || break
    fi
    sleep 1
  done
  remove_stale_runtime_files
  say "dockerd: 停止した (snapshot に process を残さない)"
  return 0
}

image_name() { compose config --images 2>>"$log_file" | head -n 1; }

# Dockerfile の FROM と compose.cloud.yaml の BASE_REGISTRY から、cloud で使う base image を組む。
# MSRV や取得元を script に重複して書かないための解決。
base_image_ref() {
  local from registry
  from="$(sed -nE 's/^FROM +([^ ]+).*/\1/p' "$repo_root/Dockerfile" | head -n 1)"
  registry="$(compose config 2>>"$log_file" | sed -nE 's/^ *BASE_REGISTRY: *"?([^" ]+)"?.*/\1/p' | head -n 1)"
  [ -n "$registry" ] || registry="$(sed -nE 's/^ARG +BASE_REGISTRY=(.*)$/\1/p' "$repo_root/Dockerfile" | head -n 1)"
  [ -n "$from" ] || return 1
  echo "${from//\$\{BASE_REGISTRY\}/$registry}"
}

# setup script の文脈では daemon が直接 (VM の system CA で) pull できる。重い部分だけを
# snapshot に入れる。
pull_base_image() {
  local base
  base="$(base_image_ref)"
  if [ -z "$base" ]; then
    say "base image: Dockerfile から参照を組めない"
    return 1
  fi
  if docker image inspect "$base" >/dev/null 2>&1; then
    say "base image: $base あり"
    return 0
  fi
  say "base image: $base を pull する"
  if docker pull -q "$base" >>"$log_file" 2>&1 </dev/null; then
    say "base image: pull 完了"
    return 0
  fi
  say "base image: pull に失敗した"
  tail -n 3 "$log_file" 2>/dev/null | cut -c1-200 | sed "s/^/$self:   /"
  return 1
}

# 開発用 image (Dockerfile の最終層まで) を用意する。build 層の network は agent proxy を
# 経由するため、proxy の無い setup script の文脈では失敗し得る。その場合は hook に任せる。
ensure_image() {
  local image
  image="$(image_name)"
  if [ -z "$image" ]; then
    say "image: compose 定義から image 名を取れない"
    return 1
  fi
  if docker image inspect "$image" >/dev/null 2>&1; then
    say "image: $image あり"
    return 0
  fi
  say "image: $image が無いため build する"
  if compose build dev >>"$log_file" 2>&1 </dev/null; then
    say "image: build 完了"
    return 0
  fi
  if [ "$mode" = "provision" ]; then
    say "image: build は skip した (setup script の文脈では agent proxy が無く、build 層の network が通らない)。session 開始時に hook が build する"
    return 0
  fi
  say "image: build に失敗した"
  tail -n 5 "$log_file" 2>/dev/null | cut -c1-200 | sed "s/^/$self:   /"
  return 1
}

# COMPOSE_FILE を session 全体へ渡す。CLAUDE_ENV_FILE は SessionStart hook が後続の
# shell へ環境変数を渡すための file。無い文脈 (setup script、手動実行) では export 文を示す。
export_compose_file() {
  export COMPOSE_FILE="$compose_files"
  if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
    if ! grep -qs "^export COMPOSE_FILE=" "$CLAUDE_ENV_FILE"; then
      echo "export COMPOSE_FILE=$compose_files" >> "$CLAUDE_ENV_FILE"
    fi
    say "COMPOSE_FILE=$compose_files を session に設定した"
  else
    say "この shell の外で使うときは export COMPOSE_FILE=$compose_files を先に実行する"
  fi
}

write_state() {
  mkdir -p "$state_dir" || return 1
  cat > "$state_file" <<STATE
inputs_sha256=$(inputs_digest)
built_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
base_image=$(base_image_ref || true)
image=$(image_name)
STATE
  say "state: $state_file を書いた"
}

print_stub() {
  cat <<STUB
#!/bin/bash
# bizdate: cloud session の環境構築。正本は repository 側にある。
#   .agents/scripts/cloud-session-setup.sh
# この stub は薄いままにする。中身を変えるときは repository を直す。
# CLOUD_SETUP_INPUTS_SHA256=$(inputs_digest)
#   ↑ script と Dockerfile / compose.yaml / compose.cloud.yaml の digest。
#     environment cache は setup script のテキストが変わったときだけ再構築されるため、
#     digest を埋めてテキストが自然に変わるようにしている。
#     貼り直す内容は \`.agents/scripts/cloud-session-setup.sh --print-stub\` で生成する。
exec "\${CLAUDE_PROJECT_DIR:-$default_clone_dir}/.agents/scripts/cloud-session-setup.sh" --provision
STUB
}

# snapshot が repo の宣言に追いついているかを見る。state が無ければ登録の案内だけ、
# digest がずれていれば貼り直し用の stub を出す。戻り値が 1 なのは digest のずれだけ。
check_drift() {
  local want have
  want="$(inputs_digest)"
  have="$(recorded_digest || true)"
  if [ -z "$have" ]; then
    say "environment cache: 未登録 (setup script に stub を貼ると session 開始が速くなる。生成: --print-stub)"
    return 0
  fi
  if [ "$want" = "$have" ]; then
    say "environment cache: repo と一致 (${have:0:12}...)"
    return 0
  fi
  say "environment cache が repo に追いついていない (cache=${have:0:12}... repo=${want:0:12}...)"
  say "対処: 次の stub を cloud environment の setup script に貼り直すと cache が再構築される。"
  echo
  print_stub
  echo
  return 1
}

doctor() {
  local image rc=0
  if ! is_cloud_sandbox; then
    say "この環境は cloud session の sandbox ではない (dockerd が無い)"
    return 1
  fi
  if daemon_ready; then say "dockerd: 起動済み"; else say "dockerd: 停止中"; fi
  image="$(image_name)"
  if [ -n "$image" ] && docker image inspect "$image" >/dev/null 2>&1; then
    say "image: $image あり"
  else
    say "image: ${image:-?} 無し"
  fi
  if [ -f "$state_file" ]; then
    say "state: $(sed -n 's/^built_at=//p' "$state_file") に作成"
  else
    say "state: 無し"
  fi
  check_drift || rc=1
  return "$rc"
}

case "$mode" in
  print-stub)
    print_stub
    exit 0
    ;;
  doctor)
    doctor
    exit $?
    ;;
esac

# hook / force / provision
if ! is_cloud_sandbox; then
  say "この環境は cloud session の sandbox ではない (dockerd が無い)。何もしない。"
  exit 0
fi

export_compose_file

if ! start_daemon; then
  say "WARNING: Docker daemon を起動できなかった。sandbox の cargo で代替せず、実行できない検証は未実施として報告する。"
  say "  再試行: bash .agents/scripts/cloud-session-setup.sh --force / 状態確認: --doctor / 詳細: doc/guidelines/cloud-session-guidelines.md"
  exit 0
fi

if [ "$mode" = "provision" ]; then
  pull_base_image || true
  ensure_image || true
  write_state || say "state: 書けなかった"
  stop_daemon
  say "provision 完了"
  exit 0
fi

if ! ensure_image; then
  say "WARNING: 開発用 image を用意できなかった。log: $log_file"
  say "  sandbox の cargo で代替せず、実行できない検証は未実施として報告する。詳細: doc/guidelines/cloud-session-guidelines.md"
  exit 0
fi

check_drift || true
say "開発コマンドは通常どおり docker compose run --rm dev cargo ... で実行する。詳細: doc/guidelines/cloud-session-guidelines.md"
exit 0
