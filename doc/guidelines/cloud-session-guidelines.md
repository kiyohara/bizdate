# Cloud Session 実行ルール

この文書は、Claude Code on the web の cloud session で bizdate を扱うとき、および cloud session 向けの設定を変更するときの共通正本である。AI agent と人間の両方がこのルールに従う。経緯は `doc/design/decision-log/0018-cloud-session-environment.md` を参照する。

cloud session とは、Anthropic が host する使い捨ての VM（sandbox）で Claude Code が動く形態を指す。ローカルの Claude Code、Cursor、Codex には影響しない。

## なぜ専用の設定が要るか

cloud session の sandbox では、このリポジトリの前提が次のように崩れる。

| 前提 | sandbox の状態 | 対処 |
|---|---|---|
| 開発コマンドは Compose 経由（`doc/guidelines/development-command-guidelines.md`） | docker CLI / daemon / compose は入っているが daemon は起動していない | SessionStart hook が daemon を起動し、開発用 image を用意する |
| toolchain は MSRV に揃える | sandbox の rustup toolchain は MSRV より古く、crate を build できない | sandbox の `cargo` は使わない。Compose 経由を維持する |
| 外向き通信 | 許可リスト方式で、直接接続は gateway が TLS を再終端する。container 内の CA では検証できない。Docker Hub の blob 配信元は許可されていない | container は sandbox の agent proxy を経由させ、base image は許可リスト内の mirror から取る（`compose.cloud.yaml`） |
| GitHub 操作は `github-op-integrated` MCP を第一選択（`doc/guidelines/github-mcp-guidelines.md`） | `op` と `gh` が無く、MCP server は起動できない | 組み込みの GitHub tool を第一選択にする（同 guideline の「cloud session」） |
| Issue 記載のブランチ名で作業する（`doc/guidelines/issue-driven-task-execution.md`） | 作業ブランチは session 作成時に platform が決め、push はそのブランチにだけ許可される | session のブランチをそのまま使う（同 guideline と `doc/guidelines/git-operation-guidelines.md` の「cloud session」） |
| commit 署名は 1Password（`doc/guidelines/git-operation-guidelines.md`） | 署名と author は platform 側で行われる | 署名の切り分け手順は適用しない（同 guideline の「cloud session」） |

## 設定ファイル

| ファイル | commit | 役割 |
|---|---|---|
| `.agents/scripts/cloud-session-setup.sh` | yes | 処理本体。hook（既定）/ `--force` / `--provision` / `--doctor` / `--print-stub` の mode を持つ。動作の詳細は script 冒頭のコメントと `--help` を正とする |
| `.claude/settings.json` | yes | SessionStart hook（`startup` と `resume`）で上記 script を呼ぶ登録だけを持つ。処理や恒久ルールを書かない。個人の設定は gitignored な `.claude/settings.local.json` に置く |
| `compose.cloud.yaml` | yes | cloud session 専用の override。container を host network にして agent proxy を通し、base image の取得元を mirror にする。image tag は書かない |
| `Dockerfile` の `ARG BASE_REGISTRY` | yes | base image の取得元の差し替え口。local では既定の `docker.io/library` を使う |
| environment の setup script | no（claude.ai/code の UI 設定） | `--print-stub` が生成する数行の stub。上記 script を `--provision` で呼ぶだけにし、処理を UI 側に書かない。script が無い repository や branch では何もせず exit 0 する（environment は repository と branch をまたいで共有される） |
| state file（VM 内 `/opt/bizdate-cloud/state`） | no | `--provision` が snapshot の出自（入力の digest、作成時刻、base image と image の名前）を記録する。hook が drift 検出に使う |

## 実行順序

1. platform がリポジトリを clone する。
2. environment cache が無ければ setup script（stub → `--provision`）が実行され、完了後に filesystem が snapshot される。cache があればこの手順は飛ぶ。setup script は agent proxy が立つ前に走り、container の中から外へは出られない。そのため `--provision` は daemon の起動、base image の pull、state file の記録だけを行い、薄い最終層の build は hook に任せる。
3. Claude Code が起動し、SessionStart hook が script を hook mode で実行する。`COMPOSE_FILE` を session に設定し、daemon を起動し、image が無ければ build し（base image が snapshot にあれば数秒）、cache の drift を確認する。hook の stdout は agent の context に入る。
4. 以後の作業は通常どおり。

hook は `resume` でも実行される。VM が作り直された後の再開でも daemon が起動する。

## 開発コマンド

コマンドの形は変えない。`doc/guidelines/development-command-guidelines.md` の基本形をそのまま使う。

```sh
docker compose run --rm dev cargo test --locked
```

- hook が `COMPOSE_FILE=compose.yaml:compose.cloud.yaml` を session に設定する。効いていない shell では `export COMPOSE_FILE=compose.yaml:compose.cloud.yaml` を先に実行するか、`docker compose -f compose.yaml -f compose.cloud.yaml run --rm dev ...` と書く。
- `TZ` は sandbox で未設定のため container は UTC で動く。タイムゾーン依存の挙動は明示的に与える。
- build したバイナリは container の中で実行する。`target/` は named volume にある。

## 検証結果の報告

cloud session の Compose は local と同じ `Dockerfile` から作った image を使う。検証結果は Compose 経由の結果として扱い、報告や note には「cloud session で実行」と添える。「Compose の外に出るもの」の runner 区別は要らない。

sandbox の `cargo` で得た結果を検証結果として報告しない。これは `doc/guidelines/development-command-guidelines.md` の原則と同じである。

## daemon が起動しないとき

hook の出力に `WARNING` が含まれる場合は、次の順で扱う。

1. `bash .agents/scripts/cloud-session-setup.sh --force` で再試行する。
2. `bash .agents/scripts/cloud-session-setup.sh --doctor` で daemon / image / cache の状態を見る。
3. それでも起動しなければ、sandbox の `cargo` で代替せず、実行できなかった検証を未実施として理由（hook の出力）とともに note と PR に書き、ユーザーに報告する。

## environment cache と drift

setup script の実行結果は filesystem snapshot として cache され、後続の session はそこから始まる。cache が作り直されるのは次の 3 つの場合だけである。

- setup script のテキストが変わった。
- environment の許可 host が変わった。
- 約 7 日の期限が来た。

`.agents/scripts/cloud-session-setup.sh`、`Dockerfile`、`compose.yaml`、`compose.cloud.yaml` を変えても、stub のテキストが変わらなければ cache は古いまま残る。このため stub には入力の digest をコメントとして埋め、貼り直しでテキストが変わるようにしている。

- hook は毎回、cache に記録された digest と repo の digest を比べる。一致しなければ「environment cache が repo に追いついていない」と警告し、貼り直し用の stub を出力する。state file が無い（setup script 未登録）場合は登録を促す 1 行だけを出す。
- 対処は人手で行う。`--print-stub` の出力を environment の setup script に貼り直す。platform は setup script を API で更新する手段を提供しないため、自動化しない。
- 警告があっても作業は続けられる。image が無い session では hook が build する。base image が snapshot にあれば数秒、無ければ 1 分程度、session の開始が遅れる。
- snapshot は branch をまたいで共有される。feature branch で `Dockerfile` を変えても、stub を貼り直すまで cache は前の状態のままである。

## Network access

environment の Network access は既定の Trusted のままでよい。使う host は次のとおりで、いずれも既定の許可リストに含まれる。

| 用途 | host |
|---|---|
| base image の取得（mirror） | `mirror.gcr.io`（`*.gcr.io`）とその配信元（`*.googleapis.com`） |
| `rustup component add`（image の build 時） | `static.rust-lang.org` |
| crate の取得 | `index.crates.io`、`static.crates.io` |

Docker Hub 本体（`registry-1.docker.io`）には到達できるが、blob の配信元は許可されていないため pull は失敗する。`compose.cloud.yaml` はこのために base image を gcr.io の Docker Hub mirror から取る。mirror は公式 image を同じ digest で配信する。ECR Public も候補だったが、blob の配信元が直接接続の許可リストに無く、agent proxy の無い setup script の文脈では pull できない。

許可されていない host への接続は、直接接続なら `403`、proxy 経由なら接続失敗として現れる。`curl -sS "$HTTPS_PROXY/__agentproxy/status"` で proxy 側の拒否理由を確認できる。拒否された host へ迂回しない。必要なら環境設定の変更としてユーザーに報告する。

## 所要時間

sandbox での実測（2026-09-12、4 vCPU）。

| 処理 | 所要時間 |
|---|---|
| daemon の起動 | 1〜3 秒 |
| base image の pull（mirror、約 590 MB） | 20〜35 秒 |
| 開発用 image の build（`rustup component add`、base image あり） | 数秒 |
| `cargo fetch --locked` | 数秒 |
| `--provision` 全体（cache 無しの setup script） | 20〜35 秒（実環境の setup script で約 20 秒） |
| hook（base image が snapshot にあり、image 無し） | 3〜6 秒（実環境で約 3 秒） |
| hook（image あり） | 1 秒未満 |
| 初回の `cargo test --locked`（named volume の作成と全 crate の build を含む） | 約 15 秒 |

setup script は 5 分以内に終わる必要がある。`--provision` は 1 分以内に収まる。setup script では container の network が通らないため、`cargo build` や `cargo fetch` の事前実行は含めない。

## 利用開始手順

1. environment は既定の設定（Trusted）で使える。setup script を登録しなくても hook が初回に image を build する。
2. session 開始を速くするには、cloud session の中で `bash .agents/scripts/cloud-session-setup.sh --print-stub` を実行し、出力を environment の setup script に貼る。次の新しい session から base image が snapshot に入り、hook は薄い最終層の build だけになる。
3. 新しい session を開き、冒頭に hook の出力（daemon、image、`COMPOSE_FILE`）が入ることと、`docker compose run --rm dev cargo fmt --check` が通ることを確認する。

## 変更するとき

- 処理を変えるときは `.agents/scripts/cloud-session-setup.sh` を直す。`.claude/settings.json` には呼び出しの登録以外を書かない。UI の stub に処理を書かない。
- script、`Dockerfile`、`compose.yaml`、`compose.cloud.yaml` を変えたら、`--print-stub` を再生成して environment に貼り直す。hook の drift 警告がその合図になる。
- cloud 向けの処理を足すときは、この script に足すか `.agents/scripts/` に別の script を置く。配置規約は `doc/guidelines/agent-configuration-management.md` の「cloud session の実行環境 script」を参照する。
- 変更後の検証は script を 2 回実行して冪等であること、`CLAUDE_CODE_REMOTE` 未設定で無出力 exit 0 になること、`--provision` が exit 0 で完走することを確認する。

## 落とし穴

- `service docker start` は sandbox では失敗する。init script が `ulimit` の変更を要求し、sandbox がそれを許さないためである。script は `dockerd` を直接起動する。
- `/run` は root filesystem の一部であり、daemon 起動中に snapshot されると pid file と socket が残る。script は dockerd / containerd でない pid の file を消し、socket は dockerd process が無いときだけ消す。`--provision` は自分で起動した daemon を最後に止める。
- container に `NO_PROXY` を渡さない。渡すと `index.crates.io` への接続が直接接続になり、container 内の CA で検証できず失敗する。
- hook は local の Claude Code でも起動する。`CLAUDE_CODE_REMOTE` が `true` でなければ無出力で終わるため、local の開発には影響しない。
- `.mcp.json` の `github-op-integrated` は cloud session で常に起動に失敗する。想定どおりであり、対処しない。
- hook は同期実行である。image が無い session では build が終わるまで session の開始が遅れる（base image が snapshot に無ければ 1 分程度）。
- environment の setup script は agent proxy が立つ前に走る。container の中から外へ出る処理（image の build、`cargo fetch`）は setup script では通らないため、`--provision` に足さない。
- environment の setup script は repository と branch をまたいで共有される。stub は script が無ければ skip して exit 0 するため、他の repository や script を含まない branch で session を開いても起動を妨げない。stub を `exec` だけの形に書き換えない。

## 関連ルール

- GitHub 操作: `doc/guidelines/github-mcp-guidelines.md` の「cloud session（Claude Code on the web）」
- git 操作、ブランチ、commit 署名: `doc/guidelines/git-operation-guidelines.md` の「cloud session（Claude Code on the web）」
- Issue 駆動タスクのブランチ: `doc/guidelines/issue-driven-task-execution.md` の手順 3
- 配置規約: `doc/guidelines/agent-configuration-management.md` の「cloud session の実行環境 script」
