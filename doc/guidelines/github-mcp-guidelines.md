# GitHub MCP 利用ルール

この文書は、bizdate リポジトリで AI agent が GitHub 操作を実行するときに、GitHub MCP Server を優先的に使うための共通正本である。

対応する MCP server 共通資材は `.agents/mcp/github-op-integrated/` に置く。導入手順、wrapper script、各 tool 用の設定例はそちらの `README.md` と `config-examples.md` を参照する。

## 適用範囲

このルールが対象とする GitHub 操作は次のとおり。

- collaboration write 操作: PR の作成・編集、issue の作成・編集・コメント追加、レビューコメントへの返信、PR レビューの作成。
- read 操作全般: PR / issue / レビューコメントの取得、検索。
- CI の read 操作: workflow / workflow run / job / artifact の一覧と詳細、job log の取得、PR の check run と commit status の取得。

対象としない操作は、それぞれ別ルールに従う。

- `git commit`、GitHub remote への `git push` / `git fetch` / `git pull`: `doc/guidelines/git-operation-guidelines.md`。
- merge、file push（API 経由でファイル内容を commit する操作）、release 作成、repository settings 変更、**workflow の実行・再実行・cancel・log 削除**など、MCP 化対象に含めない操作: `doc/guidelines/github-cli-guidelines.md` に従い `gh` で実行する。

## MCP 優先・`gh` fallback

`gh` を直接実行するたびに 1Password の承認ダイアログが必要になる頻度を下げるため、対象の GitHub 操作はまず GitHub MCP Server 経由で実行する。

このとき、`gh auth status` や `gh pr view` などの `gh` preflight を MCP より先に実行しない。最初の試行先として MCP tool の利用可否を確認する。これは `gh` の廃止を意味せず、利用できない場合の fallback は下記の優先順位に従う。

優先順位:

1. MCP server が設定済みで、操作対象が allowlist 内の MCP tool で完結する場合は MCP を使う。
2. 操作が allowlist に無い、MCP が未設定、MCP が起動失敗・応答失敗する場合は、`doc/guidelines/github-cli-guidelines.md` に従って `gh` に fallback する。
3. local git / commit signing を伴う操作は MCP に寄せず、`doc/guidelines/git-operation-guidelines.md` に従う。

### 汎用 skill / plugin と競合する場合

汎用 skill、plugin、user-level skill / rule が GitHub app や `gh` を第一選択としていても、このリポジトリでは本ガイドラインの tool 優先順位で上書きする。複数の skill が同時に該当する場合も、bizdate の project-specific な設定と本ガイドラインを優先する。

汎用 skill の分類、妥当性判断、修正、検証などの手順は再利用してよい。ただし、GitHub app / `gh` を先に使う tool routing と `gh` preflight は採用しない。

### cloud session（Claude Code on the web）

cloud session の sandbox には `op` と `gh` が無く、Docker daemon 前提の `github-op-integrated` は起動できない（MCP host に出る起動失敗の表示は想定どおりで、対処しない）。代わりに session が組み込みの GitHub MCP tool を提供する。

- 組み込み GitHub tool を第一選択とする。「操作別の第一選択」の tool 名はそのまま読み替え、skill 内の `github-op-integrated` の記載も組み込み tool に読み替える。
- `gh` fallback は無い。write が失敗したら read 系 tool で反映を確認し、未反映なら同じ tool で再試行するか、ユーザーに報告する。
- 組み込み tool には allowlist 外の write（merge、review thread の resolve、workflow の実行・再実行・cancel、API 経由の file push、auto-merge の変更など）も見える。見えていても実行しない。「CI 操作の境界」と「Review event と resolve の制約」はそのまま適用する。

## 操作別の第一選択

| 操作 | 第一選択 | 補足 |
| --- | --- | --- |
| Issue の取得・検索 | `issue_read` / `list_issues` / `search_issues` | read は MCP を先に試す。 |
| Issue の作成・更新 | `issue_write` | write failure 後は read-back して反映有無を確認する。 |
| PR の特定・取得 | `list_pull_requests` / `search_pull_requests` / `pull_request_read(get)` | `gh pr view` を先行しない。 |
| PR diff / files の取得 | `pull_request_read(get_diff / get_files)` | |
| Review / comment の取得 | `pull_request_read(get_review_comments / get_reviews / get_comments)` | 後続操作に必要な ID が response に含まれるかも確認する。 |
| PR review の作成 | `pull_request_review_write` / `add_comment_to_pending_review` | pending review 作成 → inline comment 追加 → submit の順。 |
| Inline review comment への返信 | `add_reply_to_pull_request_comment` | write 後に thread を再取得して反映を確認する。 |
| PR conversation comment | `add_issue_comment` | PR 番号を `issue_number` として渡す。 |
| PR の作成・更新 | `create_pull_request` / `update_pull_request` | failure 後は read-back し、二重実行を防ぐ。 |
| PR の check 状態 | `pull_request_read(get_check_runs / get_status)` | CI の成否を PR 単位で見るときの第一選択。 |
| workflow / run / job / artifact の一覧 | `actions_list` | `method` は `list_workflows` / `list_workflow_runs` / `list_workflow_jobs` / `list_workflow_run_artifacts`。 |
| 個別 workflow / run / job の詳細 | `actions_get` | `method` は `get_workflow` / `get_workflow_run` / `get_workflow_job` / `get_workflow_run_usage` など。 |
| job log の取得 | `get_job_logs` | 単一 job は `job_id`、run 内の失敗 job 全部は `run_id` + `failed_only=true`。`tail_lines` で末尾のみ取得できる。 |
| workflow の実行 / 再実行 / cancel / log 削除 | `gh`（ユーザー承認） | 下記「CI 操作の境界」を参照。 |
| Review thread の resolve | 自動実行しない | 下記「Review event と resolve の制約」を参照。 |
| merge / release / workflow dispatch / settings | `gh` fallback | MCP allowlist の対象外。 |
| commit / push / fetch | local git | MCP 対象外。 |

MCP tool 自体が allowlist に無い場合と、tool は使えるが response に後続操作で必要な ID が無い場合を区別する。後者では別の MCP read method で取得できないかを確認し、情報不足だけを理由に黙って `gh` を先行させない。tool 名や method は GitHub MCP Server の version により変わり得るため、`.config/github-op-integrated.conf.example` の allowlist と実行環境の tool 一覧を突き合わせる。

## CI 操作の境界

CI は **read のみ MCP に載せる**。GitHub MCP Server は `actions_run_trigger` で workflow の実行・再実行・cancel・run log 削除を提供するが、**この tool は allowlist に含めない**。

- 理由: workflow の実行と cancel は副作用を伴い、run log の削除は取り消せない。これらは `doc/guidelines/github-cli-guidelines.md` の「ユーザーの確認を得てから実行する操作」に該当する。allowlist から外すことで、承認を経ずに実行される経路を仕組みとして塞ぐ。
- 必要になった場合は `gh workflow run` / `gh run rerun` / `gh run cancel` を、ユーザーの承認を得てから実行する。
- `actions_get` の `download_workflow_run_artifact` は read だが artifact の取得を伴う。ダウンロードしたファイルを実行しない。

## Review event と resolve の制約

- GitHub 上の操作 account は単一であるため、`APPROVE` と `REQUEST_CHANGES` を自動実行しない。PR review の投稿は `COMMENT` event に限定する。
- inline thread の resolve を自動実行しない。これは tool の機能制約ではない。fine-grained PAT で `resolveReviewThread` を実行するには Pull requests 権限に加えて `Contents: Read and Write` が必要であり、本プロジェクトは `Contents: write` を付与しないためである。resolve は人間が GitHub UI で行う。
- PR の merge、review の dismiss、reviewer request の変更も自動実行しない。

## MCP write が失敗したとき

write 系 MCP tool が失敗した場合は、次の順で扱う。

1. 直ちに `gh` で同じ write を再実行しない。
2. read 系 tool で部分反映・重複の有無を確認する。
3. MCP 未反映で、かつ fallback 条件を満たす場合だけ `gh` へ fallback する。
4. fallback する場合は、試した MCP tool、失敗内容、未反映確認の結果、実行するコマンドをユーザーへ明示する。

## permission

- fine-grained PAT の repository access にこのリポジトリを含め、次を許可する。
  - Pull requests: read / write
  - Issues: read / write
  - Actions: read（workflow / run / job / artifact / job log の取得に必要）
  - Checks: read（`pull_request_read(get_check_runs)` に必要）
  - Commit statuses: read（`pull_request_read(get_status)` に必要）
- `Contents: write`、`Actions: write`、merge、release、repository settings などの追加 permission は付与しない。CI は read だけで足りる。
- PAT の実値をリポジトリに置かない。`.config/github-op-integrated.conf` に 1Password secret reference だけを書く。
