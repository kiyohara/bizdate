# 作業ブランチメモ

- ブランチ: `claude/blissful-goldberg-0z1593`（cloud session が用意したブランチ）
- PR: #48
- 最終更新: 2026-09-12

## 目的

Issue #47 に従い、Claude Code on the web の cloud session で Compose 経由の開発コマンドと GitHub 操作が成立するように、実行環境 script（`.agents/scripts/cloud-session-setup.sh`）、SessionStart hook（`.claude/settings.json`）、cloud 専用 override（`compose.cloud.yaml`）、新 guideline（`doc/guidelines/cloud-session-guidelines.md`）、各 guideline の cloud session 節、decision log 0018 を整える。

## 現在の状況

PR #48 の review と verify を完了し、startup 経路の実環境確認も済んだ。merge 待ち。merge はユーザーが行う。

## 決定事項

- 開発コマンドは cloud session でも Compose 経由を維持する（候補 A）。rustup 固定 toolchain（B）と自動 fallback（C）は採らない。理由は 0018 に書く。
- 処理本体は tool 中立な `.agents/scripts/` に置き、`.claude/settings.json` は hook 登録だけにする。環境 UI の setup script は script が生成する stub（数行）に留める。
- cloud 固有の差分（container を agent proxy 経由にする host network と proxy 変数）は `compose.cloud.yaml` に閉じ込め、hook が `COMPOSE_FILE` を export する。開発コマンドの形は変えない。
- Docker daemon は `service docker start` ではなく `dockerd` を直接起動する。init script が `ulimit` の変更で失敗するため（sandbox は rlimit の変更を許さない）。
- 参考にした別プロジェクト（非公開）の cloud session 対応方式から、環境構築の正本を repo に置いて UI は stub にする点、入力 digest を stub に埋めて cache の再構築を誘発する点、hook で drift を検出して警告する点、setup script を非 0 で終わらせない点を取り込んだ。参照プロジェクトの識別情報はこの repo に書かない。
- GitHub 操作は cloud session では組み込みの GitHub tool を第一選択にする。本 Issue と PR の作成がその先行適用になる。

## 次にやること

- ユーザーによる inline thread の resolve と merge。
- merge 前にユーザーが本ブランチで新 session を開き、hook の出力と `docker compose run --rm dev cargo fmt --check` を確認する。任意で `--print-stub` の出力を environment の setup script に貼り、再構築を確認する。

## 検証

spike の記録。

| 項目 | 結果 |
|---|---|
| S1: `service docker start` | 失敗。init script が `ulimit -Hn` / `ulimit -u` で `Operation not permitted` になり中断する |
| S1: `dockerd` を直接起動 | 成功。約 3 秒で API が応答。Server 29.3.1、storage driver overlayfs、cgroup v1、buildkit 初期化済み |
| S2: `docker compose build dev`（override 込み） | 初回失敗。Docker Hub の blob 配信元が egress policy で拒否される。`Dockerfile` に `ARG BASE_REGISTRY` を足し、override で許可リスト内の mirror を渡して成功（build 数秒。base image の取得は約 35 秒） |
| S3: `docker compose run --rm dev cargo fetch --locked` | 成功（数秒）。container は host network で agent proxy を経由する |
| S4: `cargo fmt --check` / `cargo clippy --locked --all-targets -- -D warnings` / `cargo test --locked` / `cargo build --locked`（Compose 経由） | すべて成功。clippy 11 秒、test 15 秒（unit 69 件、E2E 12 件）。`./target/debug/bizdate --version` は `bizdate 0.1.0` |

実装後の検証（cloud session 内。sandbox の絶対 path と host 名は省く）。

| 項目 | 結果 |
|---|---|
| `bash -n` / `shellcheck` | 警告なし |
| `CLAUDE_CODE_REMOTE` 未設定で hook mode | 無出力、exit 0 |
| `--print-stub` を 2 回、および repo を別ディレクトリへ複製して実行 | 出力が同一。digest は clone 先の path に依存しない |
| setup script 文脈の再現（proxy 変数と `CLAUDE_*` を外し、image・build cache・state を消してから `--provision`） | exit 0、約 35 秒。daemon 起動、base image の pull、build は設計どおり skip、state file 作成、daemon 停止 |
| 直後に hook mode（stdin に SessionStart の JSON、`CLAUDE_PROJECT_DIR`、`CLAUDE_ENV_FILE`） | exit 0、約 6 秒。daemon 起動、image build、`COMPOSE_FILE` を env file に 1 行追記、cache は repo と一致 |
| hook mode を再実行 | exit 0、1 秒未満。「起動済み」「image あり」。env file の追記は増えない |
| `--doctor` | 一致で exit 0。`compose.cloud.yaml` を変えると exit 1 で stub を出力。戻すと exit 0 |
| `--force`（`CLAUDE_CODE_REMOTE` 未設定） | hook と同じ動作 |
| env file の `COMPOSE_FILE` で `docker compose run --rm dev cargo fmt --check` / `cargo test --locked` | 成功 |
| `jq . .claude/settings.json`、override 込みの `docker compose config -q` | 成功 |
| markdown link と repo 相対 path、rule basename の一致、`git diff --check`、文体、情報統制 | 下記「機械的確認」のとおり |

daemon の停止直後に hook を実行すると、終了処理中の旧 process が pid file を持っていて新しい daemon が起動できない競合を検証で見つけた。script は process の終了を待ってから pid file を扱うように直した。

## リスク・ブロッカー

- base image の取得元は gcr.io の Docker Hub mirror に固定した。ECR Public は proxy 経由なら使えるが、proxy の無い setup script の文脈では blob の配信元が許可リストに無く pull できなかった。mirror が使えなくなった場合は環境の許可 host 追加（ユーザー操作）か `BASE_REGISTRY` の変更で対処する。
- setup script は 5 分以内に終わる必要がある。cache 無しの `--provision` は約 35 秒で、余裕は大きい。
- setup script の文脈では agent proxy が無く container の network が通らないため、`--provision` では image の最終層を build しない。session 開始時に hook が build する（約 6 秒）。
- SessionStart hook の実発火は、resume 経路（本 session）と startup 経路（stub を登録した environment の新 session）の両方で確認した。

### review 対応（PR #48）

inline comment 3 件（`[imo]` 2 件、`[nits]` 1 件）をすべて採用した。

- stub の `exec` は script が無いと exit 127 になり、setup script の非 0 終了は session の起動失敗になる。environment は repository と branch をまたいで共有されるため、stub に存在確認を入れ、無ければ exit 0 で skip するようにした。
- pid file の生存判定を `kill -0` から `/proc/<pid>/comm` の照合に変え、socket は dockerd process が無いときだけ消すようにした。pid 番号の再利用や起動遅延で、生きている daemon を壊す経路を塞ぐ。
- `--provision` は image の build を試みず、base image の pull と state の記録に絞った。guideline の記述と揃え、`--help` と冒頭コメントも直した。

投稿されなかった軽微点も同じ commit で直した。0018 の「使い分け表」が `agent-configuration-management.md` のものだと分かる表現にし、guideline の所要時間の不一致（「十数秒」と「数秒」）を揃え、exit code の記述を「`--doctor` と引数の誤り以外は exit 0」にした。

追加の検証。

| 項目 | 結果 |
|---|---|
| stub を script の無い path で実行 | skip の 1 行を出して exit 0 |
| stub を既定 clone 先で実行 | `--provision` が走り exit 0 |
| daemon を止め、別 process の pid を pid file に書いてから `--force` | pid file を stale として消し、数秒で daemon を起動 |
| `--provision`（daemon 起動中） | build を試みず、daemon を止めずに state を書いて exit 0 |
| session の resume 時の SessionStart hook | 実発火を確認。daemon 起動（2 秒）、image あり、cache 一致 |

### startup 経路の実環境確認（別 session）

stub を environment の setup script に登録した状態で、本ブランチを base に新しい cloud session を作って確認した。repo は変更していない。

| 項目 | 結果 |
|---|---|
| setup script（stub → `--provision`） | daemon 起動から base image の pull、state file の記録、daemon 停止まで約 20 秒。build は行わない |
| SessionStart hook（startup） | 6 行の出力。`COMPOSE_FILE` 設定、daemon 起動 1 秒、image が無いため build して完了（約 3 秒）、environment cache は repo と一致。`WARNING` なし |
| `COMPOSE_FILE` の伝播 | Bash tool の shell で設定済み。`CLAUDE_ENV_FILE` 経由の受け渡しが有効 |
| `--doctor` | daemon 起動済み、image あり、cache 一致で exit 0 |
| `docker compose run --rm dev cargo fmt --check` | 成功（1 秒未満） |
| `docker compose run --rm dev cargo test --locked` | 成功。81 件（unit 69、E2E 12）。named volume の作成と全 crate の build を含めて約 14 秒 |
| `./target/debug/bizdate --version` | `bizdate 0.1.0` |

## セッションログ

- 2026-09-12: 環境調査（daemon 未起動、toolchain が MSRV 未満、egress の TLS 再終端、組み込み GitHub tool）。プランを作成し承認を得た。
- 2026-09-12: Issue #47 を組み込み GitHub tool で作成。note を作成。spike S1〜S4 を実施し、base image の取得元を mirror に切り替えた。
- 2026-09-12: script・`.claude/settings.json`・`compose.cloud.yaml`・新 guideline・各 guideline の節・0018 を作成。検証で daemon 停止直後の再起動の競合と、setup script 文脈で container の network が通らない点を見つけ、script の設計を修正した。
- 2026-09-12: 3 commit に分けて push し、PR #48 を組み込み GitHub tool で作成。note を PR 番号で採番した。
- 2026-09-12: PR #48 の review 3 件に対応して push。resume 時の hook 実発火を確認した。
- 2026-09-12: 別 session で startup 経路を確認し、所要時間と文言を文書に反映した。
