# 開発ループ入口

この文書は、bizdate で作業を始めるときに読む入口である。人間と AI agent が同じ流れを共有し、GitHub Issue / `progress.md` / PR / working branch note の役割を混ぜないために使う。

方針の経緯は `doc/design/decision-log/0012-development-loop.md` にある。詳細な手順は各 guideline を正本とし、この文書には全体の流れと参照先だけを置く。

## 基本方針

- 人間・AI agent が開始する作業は必ず GitHub Issue から始める。**恒常的な**例外は `progress.md` を保守する運用作業そのもの、すなわち索引登録（`register-progress-issue`）と進捗整理（`maintain-progress`）の 2 件で、これらには起点 Issue を作らない。「Issue を索引に登録するための Issue」「進捗整理を行うための Issue」は指示書として意味を持たないためである。恒常的な例外はこの 2 件に限り、通常の作業へ広げない。リリース手順を担う skill を追加する場合も、同種の運用作業として同じ扱いへ揃える。なお、開発ループの整備そのものを Issue 無しで進めたのは、ループが存在しない状態から始めた経緯による一度きりの扱いであり、`doc/design/decision-log/0012-development-loop.md` に記録がある。
- 複数 Issue をまとめて 1 ブランチで進めない。実行時は `doc/guidelines/issue-driven-task-execution.md` に従い、1 Issue = 1 ブランチ = 1 PR とする。
- タスクは直列に消化する。複数 Issue の並行作業はしない。
- 横断的に見渡したい Issue 群は `progress.md` の進行中タスク索引に登録する。
- **PR の merge は AI agent が行わない**。レビューと merge 判断は人間が行う。

### Dependabot が作成する更新 PR

上記の Issue 起点の手順は、人間・AI agent が開始する作業に適用する。Dependabot が定期的に作成する GitHub Actions の更新 PR は bot による保守として別に扱い、起点 Issue、working branch note、`progress.md` への登録を要求しない。設定の導入・変更や、更新に伴う仕様変更・機能改修は通常の Issue 駆動タスクとする。

更新頻度、グループ化、version update の open PR 上限は `.github/dependabot.yml` を正とする。更新 PR は直列にレビューし、各 action の変更内容、40 桁 SHA の固定、末尾 tag コメントとの整合、CI の成功を確認してから人間が merge する。自動 merge は使わない。日付を末尾コメントへ追記せず、固定時点は SHA と Git 履歴から辿る。採否理由は [0015](../design/decision-log/0015-dependabot-updates.md) を参照する。

## 標準フロー

```mermaid
flowchart TD
    A["GitHub Issue を用意する"] --> B{"横断的に追跡したい Issue 群か"}
    B -- "はい" --> C["progress.md の索引に登録する"]
    B -- "いいえ" --> D["作業ブランチと working branch note を作る"]
    C --> C2["索引登録だけの独立 PR を出す<br/>起点 Issue なし / Closes なし"]
    C2 --> C3["ユーザーが review / merge"]
    C3 --> D
    D --> E["Issue の作業内容を実施し、検証する"]
    E --> F["1 Issue = 1 PR を作成する"]
    F --> G["ユーザーが review / merge"]
    G --> H{"区切りか"}
    H -- "はい" --> I["progress.md を整理する"]
    H -- "いいえ" --> A
    I --> I2["進捗整理だけの独立 PR を出す<br/>起点 Issue なし / Closes なし"]
    I2 --> I3["ユーザーが review / merge"]
    I3 --> A
```

## 各資材の役割

| 資材 | 役割 |
|---|---|
| GitHub Issue | 作業の入力。背景、依存、作業内容、スコープ外、検証を置く。 |
| `progress.md` | リリース台帳と進行中タスクの索引。詳細経緯やブランチ作業ログは置かない。 |
| Pull Request | 1 Issue に対する変更単位。description には変更意図、検証、未検証事項、`Closes #<Issue>` を書く。索引登録と進捗整理は起点 Issue を持たない独立 PR となるため、`Closes` を付けない。代わりに、索引登録では登録した Issue の一覧と順序・依存の根拠を、進捗整理では圧縮した完了フェーズと要約に残した参照を description に書く。 |
| `working-branch-notes/` | ブランチ単位の作業目的、状況、判断、引き継ぎメモ。PR に含めるが最終仕様書ではない。 |
| decision log | 後から辿る必要がある設計判断や方針変更の記録。進捗管理や作業ログは置かない。 |
| guideline | 人間と AI agent が共通で従う恒久的な作業ルール。 |
| skill | 特定の作業を始めるための実行手順。詳細手順は各 skill の `SKILL.md` を正本とする。 |

## 使う skill

| タイミング | skill | 使いどころ |
|---|---|---|
| 既存 Issue 群を索引化したいとき | `register-progress-issue` | GitHub Issue を読み、依存・順序・ブロッカーを整理して `progress.md` に最小限の行を追加または更新する。 |
| Issue 着手から review cycle の完了まで一続きで回したいとき | `drive-issue-to-reviewed-pr` | `run-issue-task` と `review-pull-request` を順に呼ぶ orchestrator。review と再確認は context を分離するため subagent へ委譲する。 |
| 個別 Issue に着手するとき | `run-issue-task` | Issue 番号を入力に、依存確認、ブランチ作成、note 作成、実装、検証、PR 作成までを進める。 |
| PR 採番直後 | `number-working-branch-note` | `draft_<branch>.md` を `<PR番号>_<branch>.md` へ rename し、note 本文と PR description の参照を揃える。 |
| PR をレビューする / review comment へ対応するとき | `review-pull-request` | `review` / `address-comments` / `verify-comments` の 3 モード。MCP-first で投稿し、resolve は人間が行う。 |
| Issue 群を消化し終えた区切り | `maintain-progress` | `progress.md` を薄く保ち、完了済みタスクを要約し、進行中タスク索引を最新化する。 |

`drive-issue-to-reviewed-pr` は既存 skill を呼び出す orchestrator であり、被委譲 skill の責務を変えない。個別 skill を単独で使う従来の流れも続ける。

skill は `.agents/skills/` を正本とし、Claude Code には `.claude/skills/` の symlink 経由で見える（`doc/guidelines/agent-configuration-management.md`）。リリース手順を担う skill は、CI とリリース体制を導入するときに追加する。

## 参照順

1. 作業したい内容に対応する GitHub Issue を読む。
2. 横断的に追跡する必要があれば、`progress.md` の索引に登録する。索引登録は `register-progress-issue` に従い、起点 Issue を作らず独立 PR として出す。
3. 個別作業は `doc/guidelines/issue-driven-task-execution.md` に従って進める。
4. git 操作は `doc/guidelines/git-operation-guidelines.md`、GitHub 操作は `doc/guidelines/github-mcp-guidelines.md`（fallback は `doc/guidelines/github-cli-guidelines.md`）に従う。
5. 実装中に方針決定が必要になった場合は `doc/guidelines/decision-log-guidelines.md` に従う。
6. PR 作成時は `doc/guidelines/pull-request-guidelines.md` に従う。
7. 区切りで `maintain-progress` skill を使い、`progress.md` を整理する。進捗整理も索引登録と同じく、起点 Issue を作らず独立 PR として出す。

## 重複させない情報

- Issue 本文を `progress.md` に複製しない。
- `issue-driven-task-execution.md` の手順をこの文書に全文複製しない。
- working branch note を恒久仕様として扱わない。
- decision log を進捗表や作業メモとして使わない。
