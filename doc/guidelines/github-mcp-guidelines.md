# GitHub MCP 利用ルール

この文書は、bizdate リポジトリで AI agent が GitHub 操作を実行するときに、GitHub MCP Server を優先的に使うための共通正本である。

対応する MCP server 共通資材は `.agents/mcp/github-op-integrated/` に置く。導入手順、wrapper script、各 tool 用の設定例はそちらの `README.md` と `config-examples.md` を参照する。

## 適用範囲

このルールが対象とする GitHub 操作は次のとおり。

- collaboration write 操作: PR の作成・編集、issue の作成・編集・コメント追加、レビューコメントへの返信、PR レビューの作成。
- read 操作全般: PR / issue / レビューコメントの取得、検索。

対象としない操作は、それぞれ別ルールに従う。

- `git commit`、GitHub remote への `git push` / `git fetch` / `git pull`: `doc/guidelines/git-operation-guidelines.md`。
- merge、file push（API 経由でファイル内容を commit する操作）、release 作成、workflow dispatch、repository settings 変更など、MCP 化対象に含めない操作: `doc/guidelines/github-cli-guidelines.md` に従い `gh` で実行する。

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
| Review thread の resolve | 自動実行しない | 下記「Review event と resolve の制約」を参照。 |
| merge / release / workflow dispatch / settings | `gh` fallback | MCP allowlist の対象外。 |
| commit / push / fetch | local git | MCP 対象外。 |

MCP tool 自体が allowlist に無い場合と、tool は使えるが response に後続操作で必要な ID が無い場合を区別する。後者では別の MCP read method で取得できないかを確認し、情報不足だけを理由に黙って `gh` を先行させない。tool 名や method は GitHub MCP Server の version により変わり得るため、`.config/github-op-integrated.conf.example` の allowlist と実行環境の tool 一覧を突き合わせる。

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

- fine-grained PAT の repository access にこのリポジトリを含め、Pull requests と Issues の read / write を許可する。
- `Contents: write`、merge、release、workflow dispatch、repository settings などの追加 permission は付与しない。
- PAT の実値をリポジトリに置かない。`.config/github-op-integrated.conf` に 1Password secret reference だけを書く。
