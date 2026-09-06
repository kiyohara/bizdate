---
name: number-working-branch-note
description: PR 採番直後に working branch note のファイル名へ PR 番号を割り当てる。`working-branch-notes/draft_<branch>*.md` が存在し、対応する PR が OPEN な場合に、note のリネーム・関連参照(note 本文・PR description)の置換・commit・push までを一連の手順で安全に実施する。note の "確定" を意味するものではなく、採番後も note は通常通り更新される前提。
---

# number-working-branch-note

PR を作成した直後に `working-branch-notes/` 配下の `draft_...md` を `<PR-number>_...md` へ採番（リネーム）し、note 本文と PR description に残る旧ファイル名参照を新名へ揃え、commit から push、PR 反映までを安全に完了させるための skill。

この skill は note の「仕上げ」や「確定」を意味するものではない。採番後も note は作業の進行に応じて更新され続ける前提とする。skill の責務は **ファイル名規約の切り替え時点での整合取り** に限られる。

## 適用範囲

このリポジトリで PR を作成した後、ブランチ内に `working-branch-notes/draft_<escaped-branch>*.md` が残っているときの採番処理に限る。次のいずれかに該当する場合は対象外として停止し、ユーザーに状況を報告する。

- 現在ブランチが `main` である。
- 対応する PR を `github-op-integrated` MCP tool で取得できない。
- PR の `state` が `OPEN` ではない。
- PR の `headRefName` が現在ブランチと一致しない。
- `working-branch-notes/draft_<escaped-branch>*.md` が 1 件も存在しない（採番処理不要）。
- escape 後のブランチ名と一致しない無関係な `draft_*.md` が混ざっている（自動判断しない）。

## 参照する正本

実行前および疑問が出た時点で、以下を必ず参照する。

- `doc/guidelines/working-branch-notes-handling.md` — ファイル名規約、escape ルール、番号付き note と draft の優先関係。
- `doc/guidelines/working-branch-notes-security.md` — push 前の情報統制チェック観点。
- `doc/guidelines/github-mcp-guidelines.md` — GitHub 操作は MCP 優先。
- `doc/guidelines/github-cli-guidelines.md` — MCP fallback として `gh` を使う場合の形式。
- `doc/guidelines/git-operation-guidelines.md` — `git commit` / `git push` の署名と実行環境制約。
- `doc/guidelines/pull-request-guidelines.md` — PR title / description の書式。

## GitHub 操作形式

- PR の read / update は、必ず最初に `github-op-integrated` MCP tool を試す。
- 必要な MCP tool が現在の tools に見えていない場合は、`gh` へ進む前に利用中 agent の tool discovery 機構で `github-op-integrated` を検索する。
- `gh pr view` / `gh auth status` などの `gh` preflight を MCP tool の試行より先に実行しない。
- `gh` へ fallback する場合は `doc/guidelines/github-cli-guidelines.md` に従い、`.op/` と `op` が使えるなら `op plugin run -- gh ...` を使う。
- write 系を `gh` に fallback する場合は、再実行前に read 系 tool で未反映かどうかを確認してから実行する。

## stale 表現の定型置換

Step 5 と Step 10 で、次の定型のみ置換する。

| 置換前 | 置換後 |
| --- | --- |
| `PR 未作成` | `PR #<PR-number> 作成済み` |
| `PR 作成後に更新` | `PR #<PR-number> に更新済み` |
| `working branch note が未確定` | `working branch note を採番済み` |
| `（採番前）` | `#<PR-number>` |

採番は note の確定を意味しないため、`working branch note が確定済み` など確定を含意する表現へ置換してはならない。定型置換後に文脈が不自然になる場合は触らず、終了時に報告する。

## 手順

各ステップで失敗・矛盾を検出したら停止し、ユーザーに報告する。安全側に倒し、判断に迷うときは進めずに確認する。

### 1. 前提チェック

```sh
git branch --show-current
ls working-branch-notes/draft_*.md 2>/dev/null
```

そのうえで `list_pull_requests` で現在 branch を head に持つ open PR を取得する。現在ブランチが `main`、`draft_*.md` が 0 件、PR が取得できない、`state` が `OPEN` でない、`headRefName` が一致しない、のいずれかなら停止する。

### 2. 対象 note の特定

- 現在ブランチ名を `doc/guidelines/working-branch-notes-handling.md` の escape ルールで変換する。
- 対象は `draft_<escaped-branch>.md` と `draft_<escaped-branch>__*.md` の全件。
- escape branch 名と一致しない `draft_*.md` が混ざっていれば停止し、切り分けを求める。

### 3. 既存番号付き note との衝突確認

対応する `<PR-number>_<escaped-branch>.md` が既に存在する場合は停止する。番号付きを正、draft を移行漏れとする方針に従い、上書きや自動削除はしない。

### 4. rename

```sh
git mv working-branch-notes/draft_<escaped-branch>.md \
       working-branch-notes/<PR-number>_<escaped-branch>.md
```

suffix 付きがあれば同様に処理する。

### 5. note 本文更新

`最終更新:` 欄は触らない（note は最終仕様書ではなく、1:1 整合を要求しないため）。

- 先頭メタ行の `- PR:` 欄に `#<PR-number>` を入れる。既に正しい値があれば変更しない。
- 本文中の `draft_<escaped-branch>` 表記を `<PR-number>_<escaped-branch>` に置換する。置換対象は **具体的な escape branch 名を含む参照** に限る。`draft_...md` のような汎用 placeholder は対象外。
- 「stale 表現の定型置換」の表に当てはまる表現だけを書き換える。曖昧な表現は触らず報告する。

編集後、対象 note を `git add <path>` で再 stage する。Step 4 の `git mv` は rename 時点の内容しか index に載せないため、本文編集分を改めて stage しないと commit に含まれない。

### 6. 情報統制チェック

`doc/guidelines/working-branch-notes-security.md` の「編集時の確認」に沿って rename 後の note 全体を確認する。該当が見つかった場合は placeholder に置き換えてからでないと commit へ進まない。

### 7. commit 対象の限定

```sh
git status
git diff --cached --name-status
```

staged 変更が `working-branch-notes/` 配下の rename と本文更新だけであることを確認する。対象 note に unstaged の変更が残っていないことも確認する（Step 5 の `git add` 漏れ検出）。他の変更が混入していれば `git restore --staged` で外すか、切り分けを求める。

### 8. commit

変更が無ければ commit を作らず、その旨を伝えて終了する。変更がある場合は次で commit する。

```sh
git commit -m "docs: working branch note を PR #<number> で採番する"
```

署名は `doc/guidelines/git-operation-guidelines.md` に従う。失敗した場合は同ガイドラインの切り分け手順（まず `gpg.ssh.program` を確認する）に従う。

### 9. push

```sh
git push
```

### 10. PR description 更新

- description 内の `draft_<escaped-branch>` 表記を `<PR-number>_<escaped-branch>` に置換する。具体的な escape branch 名を含む参照のみが対象。
- 「stale 表現の定型置換」の表に当てはまる表現を書き換える。
- title は通常触らない。
- `doc/guidelines/pull-request-guidelines.md` に従い、日本語維持・tool 名なし・既存表現の置換に留める。新規セクションを追加しない。

`update_pull_request` で反映する。`gh` へ fallback する場合は body をファイル経由で渡し、shell エスケープの取りこぼしを避ける。

## 終了時の報告

- rename した note の一覧（旧名 → 新名）。
- 情報統制チェックで修正した箇所があればその概要。
- commit のメッセージと SHA、push の成否。
- PR description への変更点。
- 定型に当てはまらず触らなかった stale 表現があればその一覧。

## やらないこと

- note の「確定」や「完成版へのまとめ直し」。
- note 本文の網羅的レビュー。
- `最終更新:` 欄の自動更新。
- `<PR-number>_...md` が既に存在するときの自動上書き・自動削除。
- 関連しない作業ツリー変更の commit への巻き込み。
- 定型に当てはまらない stale 表現や title の推測による書き換え。
- PR description への新規セクション追加。
