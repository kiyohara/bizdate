<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# cloud session の実行環境

- 状態: decided
- 作成日: 2026-09-12
- 最終更新日: 2026-09-12
- 関連: `doc/guidelines/cloud-session-guidelines.md`, `doc/guidelines/development-command-guidelines.md`, `doc/guidelines/agent-configuration-management.md`, `doc/guidelines/github-mcp-guidelines.md`, `doc/guidelines/git-operation-guidelines.md`, `doc/guidelines/issue-driven-task-execution.md`, `doc/design/decision-log/0011-ai-agent-lineup.md`, `doc/design/decision-log/0014-dependency-crates-and-toolchain.md`

## 背景

Claude Code on the web（Anthropic が host する cloud session）でこのリポジトリを扱えるようにする（Issue #47）。cloud session の sandbox を実測したところ、次の前提が崩れていた。

- Docker CLI / daemon / Compose は入っているが daemon は起動していない。`doc/guidelines/development-command-guidelines.md` の「Compose 経由」がそのままでは成立しない。init script（`service docker start`）は `ulimit` の変更を要求し、sandbox がそれを許さないため失敗する。`dockerd` を直接起動すれば約 3 秒で使える。
- sandbox の Rust toolchain は `Cargo.toml` の `rust-version`（0014 で最新 stable に固定）より古く、crate を build できない。
- 外向き通信は許可リスト方式で、直接接続は gateway が TLS を再終端する。VM の system CA はその CA を信頼するが、container 内の stock CA は信頼しない。sandbox の agent proxy 経由なら実サーバ証明書が返る。Docker Hub の blob 配信元は許可されておらず、公式 image の pull が失敗する。許可リスト内の mirror（gcr.io の Docker Hub mirror）からは同じ image を取得できる。ECR Public は manifest には到達できるが、blob の配信元が直接接続の許可リストに無い。
- `op` と `gh` が無く、`github-op-integrated` MCP server は起動できない。session は組み込みの GitHub MCP tool を提供する。
- 作業ブランチは platform が決め、push 先はそのブランチに固定される。commit 署名と author は platform 側で行われる。
- 実行環境の準備は、environment ごとの setup script（UI 設定、repo 外、filesystem snapshot として cache）と、repo の `.claude/settings.json` に置く SessionStart hook（毎 session）の 2 段で行える。setup script は Claude Code が起動する前、agent proxy が立つ前に走る。後者は Claude Code だけが読む配置であり、0011 の「後から見直す条件」（loading 機構の変化）に触れる。

同種の対応を先に行った別プロジェクト（非公開）の方式を検討材料にした。取り込んだ要点は「検討内容」に書く。参照先の識別情報はこのリポジトリに残さない。

## 候補

### 開発コマンドの実行方式

- A: Compose 経由を維持する。SessionStart hook から tool 中立な script を呼び、Docker daemon を起動して dev image を用意する。
- B: sandbox に `rust-version` と同じ rustup toolchain を入れ、cloud session だけ `cargo` を直接実行する例外を「Compose の外に出るもの」に足す。
- C: script に A を実装し、daemon が起動しないときは B へ自動 fallback する。
- D: 手順だけ文書化し（agent が毎回 daemon を起動する）、hook や script を置かない。
- E: cloud session を対象外とし、文書作業に限る。

### 処理本体の置き場

- P1: `.agents/scripts/`（既存の `worktree-setup.sh` と同じ tool 中立の置き場）。`.claude/settings.json` は登録だけ。
- P2: `.claude/` 配下に hook 専用のディレクトリを作る（SessionStart hook は Claude Code 固有の機構であるため）。

### base image の取得

- I1: `Dockerfile` に取得元の build arg（`BASE_REGISTRY`、既定は `docker.io/library`）を足し、cloud 専用 override が許可リスト内の mirror を渡す。
- I2: script が mirror から pull して Docker Hub 名に tag を付け替え、`Dockerfile` に触れない。
- I3: environment の許可 host に Docker Hub の配信元を追加してもらう。

### 文書の置き方

- W1: 新 guideline を 1 本置いて正本にし、既存 guideline には例外と誘導だけを足す。
- W2: 新 guideline を作らず、既存 guideline それぞれに節を足す。

## 検討内容

- A は開発コマンドの形も報告の扱いも変わらない。spike で daemon の起動、mirror からの base image 取得、override 込みの build、container からの `cargo fetch --locked`、`fmt` / `clippy` / `test` / `build` がすべて通ることを確認した（daemon 起動 1〜3 秒、base image 取得約 35 秒、build 数秒、fetch 数秒、clippy 11 秒、test 15 秒。cache 無しの `--provision` は約 35 秒、その直後の hook は約 6 秒）。
- B は daemon に依存しないが、toolchain が 2 系統になり、「host の toolchain の結果を検証として報告しない」原則に恒久的な例外を作る。MSRV の記載箇所も増える。
- C は 2 経路の保守になるうえ、Compose が動く場面でも生の `cargo` を使う誘因になる。daemon が起動できない事例が出ていない段階で例外を用意するのは「先回りで仮想シナリオに備えない」に反する。
- D は手順が守られる保証がなく、resume 後に daemon が落ちていることに気づかず sandbox の `cargo` に流れやすい。hook は 1 ファイルの登録で済む。
- E は cloud session の利点（手元の環境に依存しない実行）を捨てる。
- P1 と P2: script の主務（daemon 起動、image の用意、cache の記録）は hook 固有ではなく、environment の setup script からも同じ script を呼ぶ。他 tool の cloud 環境で同じ処理が要る可能性もある。既存の tool 中立 script と同じ置き場にし、`.claude/settings.json` は「共通正本 + 薄い入口」の入口として扱う方が配置規約に沿う。
- mirror の選択: ECR Public は agent proxy 経由なら pull できるが、proxy の無い setup script の文脈では blob の配信元（CDN）が許可リストに無く失敗する。gcr.io の mirror は配信元が許可リスト内にあり、どちらの文脈でも pull できる。
- I1 と I2 と I3: I2 は `Dockerfile` に触れないが、取得元が script の中に隠れ、local の image 名と実体の対応が見えなくなる。I3 は environment ごとの設定になり repo で管理できず、既定の環境では動かない。I1 は 2 行の変更で取得元が override に明示され、local の挙動は変わらない。
- W1 と W2: cloud session の内容は開発コマンド、GitHub 操作、git 操作、ブランチ、agent 設定の 5 領域にまたがる。W2 は各 guideline が長くなり、環境の使い始めや cache の扱いなど、どの guideline にも属さない手順の置き場が無い。W1 は shim 2 本と index 行が増えるが、1 テーマ 1 ファイルの原則に沿い、cloud session を使う人が読む入口が 1 つになる。
- setup script と hook の分担: setup script の実行結果は snapshot として cache され、後続 session はそこから始まる。cache が作り直されるのは setup script のテキスト変更、許可 host の変更、約 7 日の期限だけである。script や `Dockerfile` を直しても stub のテキストが変わらなければ cache は古いまま残るため、入力（script、`Dockerfile`、`compose.yaml`、`compose.cloud.yaml`）の digest を stub のコメントに埋め、`--print-stub` で貼り直し用の stub を生成する。hook は毎回、cache に記録した digest と repo の digest を比べ、ずれていれば警告して stub を出す。platform は setup script を API で更新する手段を提供しないため、貼り直しは人手に残す。
- setup script の文脈では agent proxy が無く、container 内から外へ出られない。build 層（`rustup component add`）と `cargo fetch` は proxy を前提にするため、`--provision` では daemon の直接 pull で取れる base image（重い部分）と state file の記録に絞り、薄い最終層の build は hook に任せる。build 時に gateway の CA を image に渡せば setup script で完結するが、`Dockerfile` への変更が増えるため見送った。
- setup script が非 0 で終わると session が起動しないため、`--doctor` と引数の誤り以外は失敗しても exit 0 とし、状況を stdout に短く出す。hook の stdout は agent の context に入る。environment は repository と branch をまたいで共有されるため、stub は script が無ければ何もせず exit 0 する。
- hook は local の Claude Code でも起動する。`CLAUDE_CODE_REMOTE=true` の guard に加えて、`dockerd` の存在を positive check とし、想定外の環境では何もしない。
- `/run` は root filesystem の一部であり、daemon 起動中の snapshot には pid file と socket が残る。script は生きていない pid の file を消し、`--provision` は自分で起動した daemon を最後に止める。
- GitHub 操作: 組み込み tool は `github-op-integrated` の allowlist に無い write（merge / resolve / workflow 実行 / file push）も露出する。tool の可視性ではなく guideline の境界で禁止を維持する。
- ブランチ: 「Issue 記載のブランチ名」は cloud session では満たせない。1 Issue = 1 ブランチ = 1 PR の本質はブランチ名ではなく対応関係にあり、session のブランチをそのまま使えば成立する。
- 別プロジェクトの方式から取り込んだ要点は、正本を repo に置いて UI は stub にすること、入力 digest を stub に埋めて cache の再構築を誘発すること、hook で drift を検出して警告すること、setup script を非 0 で終わらせないこと、cloud 判定を 2 段の guard にすること、所要時間を実測して 5 分予算との余裕を文書化することである。処理本体を tool 固有ディレクトリに置く点と、rule shim を作らない点は本リポジトリの配置規約に合わないため採らなかった。

## 決定

- cloud session でも開発コマンドは Compose 経由とする（候補 A）。B と C は採らず、daemon が起動しない session では sandbox の `cargo` で代替せず未実施として報告する。
- 処理本体は `.agents/scripts/cloud-session-setup.sh`（P1）、登録は `.claude/settings.json` の SessionStart hook（`startup|resume`）。script は hook / `--force` / `--provision` / `--doctor` / `--print-stub` の mode を持ち、`CLAUDE_CODE_REMOTE=true` のときだけ hook として動く。冪等・非対話で、`--doctor` と引数の誤り以外は失敗しても exit 0。
- cloud 固有の差分は `compose.cloud.yaml` に置き、hook が `COMPOSE_FILE=compose.yaml:compose.cloud.yaml` を session に設定する。container は host network で agent proxy を経由し、`NO_PROXY` は渡さない。
- base image は `Dockerfile` の `ARG BASE_REGISTRY`（I1）で取得元を差し替え、cloud では許可リスト内の mirror（gcr.io の Docker Hub mirror）を使う。local は既定の Docker Hub のまま。
- environment の setup script は任意とし、登録する場合は `--print-stub` が生成する stub に限る。`--provision` は daemon 起動、base image の pull、state file の記録、daemon の停止を行う。
- 文書は新 guideline `doc/guidelines/cloud-session-guidelines.md` を正本とし（W1）、shim 2 本と `AGENTS.md`、`doc/guidelines/README.md` を揃える。既存 guideline には例外と誘導だけを足す。
- GitHub 操作は cloud session では組み込み GitHub tool を第一選択とし、write の境界は `doc/guidelines/github-mcp-guidelines.md` のまま適用する。
- 作業ブランチは session が用意したものを使い、commit 署名は platform に委ねる。
- 0011 の配置規約は維持し、`doc/guidelines/agent-configuration-management.md` の「使い分け」表に実行環境 setup script の行を足す。0011 は superseded にしない。

## 理由

Compose を維持すれば、検証の正が 1 つのまま cloud session を扱える。hook で daemon を起動する仕組みは script と登録の 2 ファイルで済み、失敗時も「報告して止まる」だけで既存原則と衝突しない。B や C は例外と toolchain の系統を増やし、守られないルールを生みやすい。処理本体を tool 中立の置き場に置けば、setup script からも他 tool からも同じ script を呼べ、「共通正本 + 薄い入口」の形を保てる。

## 影響

- `.agents/scripts/cloud-session-setup.sh`、`.claude/settings.json`、`compose.cloud.yaml` を追加し、`Dockerfile` に `ARG BASE_REGISTRY` を足した。
- `doc/guidelines/cloud-session-guidelines.md` を追加し、`.claude/rules/` と `.cursor/rules/` の shim、`AGENTS.md`、`doc/guidelines/README.md` を揃えた。
- `doc/guidelines/development-command-guidelines.md`、`agent-configuration-management.md`、`github-mcp-guidelines.md`、`git-operation-guidelines.md`、`issue-driven-task-execution.md` に cloud session の節・文を追加した。`.agents/skills/run-issue-task/SKILL.md`、`.claude/rules/development-command-guidelines.md`、`.claude/rules/github-mcp-guidelines.md`、`.github/copilot-instructions.md` を同期した。`.cursor/rules/` の既存 shim は cloud session に関係しないため変更しない。
- `.mcp.json` の `github-op-integrated` は cloud session で起動に失敗する表示が出る。想定どおりとして対処しない。
- environment の setup script（UI）は repo で管理できない。登録内容は `--print-stub` の出力に固定し、変えるときは script 側を先に変える。
- MSRV を上げるときの手順に、stub の貼り直しが加わる（`doc/guidelines/development-command-guidelines.md`）。

## 後から見直す条件

- cloud session で Docker daemon が起動できない、または mirror や crates.io に到達できない事例が繰り返される場合（候補 B の再検討）。
- hook 内の build や pull の待ち時間が作業の妨げになる場合（build 時に gateway の CA を渡して setup script で完結させる案、hook の async 化）。
- Claude Code on the web の hook / setup script / snapshot の仕様が変わり、hook が cloud session で動かなくなった場合。
- Codex / Cursor の cloud 環境を使い始め、同じ script を呼ぶ入口が必要になった場合。
- 組み込み GitHub tool の構成が変わり、guideline の境界で禁止を維持できなくなった場合。
- Docker Hub の配信元が許可リストに入り、mirror が不要になった場合。
