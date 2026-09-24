---
name: run-release
description: bizdate の version を公開する作業を `doc/guidelines/release-guidelines.md` に沿って進める。公開する version を入力に、公開後確認 Issue の起票・再利用、必要ならリリース準備 PR、公開前確認、公開対象と結果の提示までを行い、ユーザーの文言での承認と tag の push を待つ。push の後は release workflow を見守り、失敗は承認を得てから扱い、公開後確認と証拠の記録、未確認項目の追跡 Issue、公開後の PR への引き継ぎまでを行う。tag の push、PR の merge、secrets の操作は行わない。
---

# run-release

bizdate の version を公開するための実行 skill。

手順の正本は `doc/guidelines/release-guidelines.md` である。本 skill は実行の順序、止まる箇所、報告だけを定め、確認の項目、コマンド、復旧の扱いは guideline を参照する。本 skill と guideline が食い違う場合は guideline を優先する。

## 入力

- 公開する version（例: `0.1.0`）。version はユーザーが決める。指定が無ければ、`Cargo.toml` の `version` と前回の公開からの変更を示して確認する。
- 公開後確認 Issue の番号（任意）。
- 複数の version を同時に扱わない。
- 途中から再開する場合（tag の push の後、公開後確認の途中など）は、公開後確認 Issue の「状態」節の状態と再開条件から、再開する手順を決める。

## 参照する正本

- `doc/guidelines/release-guidelines.md` — 手順の正本。
- `doc/design/distribution.md` — 配布物の仕様。
- `doc/guidelines/github-mcp-guidelines.md` / `doc/guidelines/github-cli-guidelines.md` — GitHub 操作。Release と workflow の操作はユーザーの承認を要する。
- `doc/guidelines/development-command-guidelines.md` — Compose 経由の実行と release workflow の job。
- `doc/guidelines/cloud-session-guidelines.md` — cloud session での制約。
- `doc/guidelines/issue-driven-task-execution.md`、`doc/guidelines/working-branch-notes-handling.md`、`doc/guidelines/working-branch-notes-security.md`、`doc/guidelines/pull-request-guidelines.md` — リリース準備 PR を出す場合。

## 手順

### 1. 前提の確認

- guideline の「役割と権限」で、agent が行わない操作を確かめる。
- レビュー待ちの自分の PR が他に無いことを確かめる。あればユーザーに確認する。
- `v<version>` の tag と Release が無いことを確かめる。有れば止めて報告する。

### 2. 公開後確認 Issue の起票・再利用

1. title が `v<version> の公開後確認` の Issue を、open と closed の両方から探す。
2. open の Issue があれば再利用する。本文の「状態」節を読み、再開する手順を決める。
3. closed の Issue があれば止め、ユーザーに確認する。その version は公開済みか中止である。
4. 無ければ `references/post-release-issue.md` の雛形で起票する。状態は公開待ちとし、担当と再開条件を書く。
5. 本文を read-back で確かめる。

### 3. リリース準備 PR

version が `Cargo.toml` の `version` と異なる場合だけ行う。同じなら 4 へ進む。

1. 作業ブランチと working branch note を作る（cloud session では session のブランチを使う）。
2. `Cargo.toml` の `version` を変え、`docker compose run --rm dev cargo update --workspace` で `Cargo.lock` を揃える。差分が 2 file の version だけであることを確かめる。
3. `docker compose run --rm dev .github/scripts/check-release-tag.sh v<version>` が ok になることを確かめる。
4. PR を出す。description に `Refs #<公開後確認 Issue>` と、merge は公開ではないことを書く。`Closes` を付けない。
5. `number-working-branch-note` で note を採番する。同 skill の報告は `.agents/skills/run-issue-task/SKILL.md` の「被委譲 skill の報告の引き上げ」と同じ扱いで、本 skill の報告へ含める。
6. PR の URL を報告し、ユーザーの merge を待つ。

### 4. 公開前確認

- guideline の「公開前確認」の各項目を行い、結果を公開後確認 Issue にコメントで残す。本文の候補 SHA と再開条件を更新する。
- 満たさない項目があれば止め、内容と推奨する対処を報告する。

### 5. 承認の提示

- guideline の「承認」の項目を 1 つの提示にまとめる。tag の push のコマンドは、guideline の「tag の push」に値を埋めて示す。
- **ここで止まる。** ユーザーの文言での承認と、tag を push したという知らせを待つ。承認が無いまま先へ進まない。

### 6. 監視と復旧

- tag の run を見つけ、guideline の「監視」の job を確かめ、終わったら結果を示す。
- Release が作られたら、公開後確認 Issue の状態を公開済みにし、確定 SHA、Release と run の URL を書く。
- 失敗したら guideline の「復旧」に従う。再実行と Release の編集は、対象と理由を示して承認を得てから行う。version を上げる場合は、Issue を中止にして新しい version で 1 からやり直す。

### 7. 公開後確認

- guideline の「公開後確認」の各項目を、行える環境で行う。行えない項目（macOS、Linux arm64、cloud session から届かない既定 CSV の取得など）は、手順と記録先を示してユーザーに依頼する。
- 証拠は guideline の「証拠の残し方」に従い、公開後確認 Issue にコメントで残す。結果が出た項目は本文の checklist に反映する。
- 承認を得ていれば、Release 本文に要約を加える。
- 前の version の未確認項目の追跡 Issue が open なら、その項目も確かめる。
- 確かめられない項目は、guideline の「未確認項目の追跡」に従って追跡 Issue を起こす。

### 8. 確認済みと引き継ぎ

- 全項目の結果を記録し、残る項目を追跡 Issue へ引き継いだら、状態を確認済みにし、未確認の追跡先を書く。
- 公開後の PR は、この Issue を入力とする `run-issue-task`（review まで回す場合は `drive-issue-to-reviewed-pr`）で出す。本 skill の中では作らず、次の作業として報告する。

## 停止条件

| 状況 | 扱い |
| --- | --- |
| version が決まっていない | 候補を示してユーザーに確認する |
| 同じ version の closed の公開後確認 Issue、tag、Release のいずれかがある | 止めて報告する |
| 公開前確認の項目を満たさない | 止めて、内容と推奨する対処を報告する |
| 承認を待つ | 止まる。文言での承認と tag の push の知らせを待つ |
| workflow が失敗した | guideline の「復旧」に従う。承認の無い再実行をしない |
| `git` / `gh` / MCP が 1Password の承認待ちで失敗した | `doc/guidelines/one-password-integration-guidelines.md` に従って中断する |

## 終了時の報告

- 公開後確認 Issue の URL、状態、再開条件。
- 公開した場合は tag、確定 SHA、Release と run の URL。
- 公開前確認と公開後確認の結果の要約。確かめられなかった項目と追跡 Issue。
- ユーザーに残る操作（承認、tag の push、PR の merge、Mac での確認など）。
- リリース準備 PR を出した場合は、その URL と、`number-working-branch-note` から引き上げた項目（完了として書き換えたタスク行の一覧と、触らなかった stale 表現・タスク行の一覧。途中で停止した場合は停止理由と未反映の変更。0 件または呼ばなかった場合はその旨）。

## やらないこと

- tag の作成と push。
- PR の merge。
- secrets の登録・変更・削除。
- 承認の無い workflow の再実行と cancel、Release の編集と削除。
- 既存の tag の付け替えと削除、同じ version の作り直し。
- 予行のための公開後確認 Issue の起票。
- 公開後確認を終える前の、README の予定表記の変更とリリース台帳への行の追加。
- 実行していない確認を確認済みとして記録すること。
