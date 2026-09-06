# 作業ブランチメモ

- ブランチ: `add-development-loop-skills`
- PR: #4
- 最終更新: 2026-09-06

## 目的

開発環境整備フェーズ 4（最終フェーズ）として、開発ループを回すための skill を配置する。フェーズ 1〜3 で作った guideline と MCP の上に、実行手順を packaging する。

対象は次の 6 つ。

1. `run-issue-task` — Issue 駆動タスクの実行
2. `register-progress-issue` — 既存 Issue の `progress.md` 索引への登録
3. `maintain-progress` — `progress.md` の整理
4. `number-working-branch-note` — PR 採番後の note リネーム
5. `review-pull-request` — PR review / コメント対応 / 再確認（3 モード）
6. `.claude/skills/` symlink と `development-loop.md` の skill 表

## 現在の状況

skill 5 種、references 3 本、symlink 5 本、`development-loop.md` の skill 表を配置した。PR 作成待ち。

## 決定事項

### slapex からの移植方針

skill は slapex の実装を土台にする。ただし bizdate に存在しない資材への参照は落とす。

| slapex の参照 | bizdate での扱い |
|---|---|
| development-command-guidelines | **参照しない**。`compose.yaml` と Cargo プロジェクトが無いため未整備（`progress.md` の「後続で追加する guideline」） |
| release skill | **参照しない**。release skill は CI / リリース体制と一緒に後送り |
| `progress.md` のリリース履歴 | リリース未実施のため、`maintain-progress` では「リリースを開始したら台帳を設ける」前提で条件付きに書く |

### `release` skill を含めない

リリース体制（`dist`、GitHub Actions、Homebrew）が未整備の段階で release skill だけ置いても実行できない。CI / リリース基盤を入れるときに同時に追加する。

### symlink は Claude Code のみ

`.agents/skills/` は Cursor と Codex が直読するため、`.claude/skills/` への symlink だけを作る（`doc/guidelines/agent-configuration-management.md`）。

## 次にやること

- skill 5 種と references を配置する。
- `.claude/skills/` の symlink を張る。
- `development-loop.md` に skill 表を追加する（PR #2 で「フェーズ 4 で追加する」と決めていた分）。
- PR を作成し、note を採番する。

## 検証

- **symlink**: 5 本すべてで `.claude/skills/<name>/SKILL.md` が読め、symlink 名と正本ディレクトリ名が一致することを確認。`find -L .claude/skills -maxdepth 1 -type l` は 0 件（broken symlink 無し）。
- **frontmatter**: 5 skill すべてに `name` と `description` があることを確認。
- **落とした参照**: `development-command-guidelines` と release skill への参照が skill 本文に残っていないことを grep で確認（0 件）。
- **参照の解決**: `.agents/skills/` を含めて全 `*.md` / `*.mdc` を走査し、未解決 0 件。`number-working-branch-note` 内の裸ファイル名 2 箇所は full path へ揃えた。
- **秘密情報スキャン**: 検出なし。
- **未検証**: skill が各 agent の一覧に実際に現れるかは、MCP 設定と同じく host の再起動が必要なため本セッションでは確認できていない。merge 後に Claude Code / Cursor / Codex を再起動して確認する。

## リスク・ブロッカー

- skill の discovery は host の再起動が必要。本セッションでは反映を確認できない。
- `review-pull-request` は運用実績が無い。最初の適用時に、canonical metadata の形式や resolve 可マーカーの運用が実際に回るかを確認する必要がある。

## セッションログ

- 2026-09-06: PR #3 merge 後に main から分岐。本 note を作成した。
- 2026-09-06: PR #4 を作成し、note を採番した。
- 2026-09-06: skill 5 種と references 3 本を配置。slapex 版から development-command-guidelines と release skill への参照を落とし、`review-pull-request` には CI tool（`actions_list` / `get_job_logs`）と Copilot と共通の prefix 体系を追記した。
- 2026-09-06: `.claude/skills/` の symlink 5 本を作成。`development-loop.md` に skill 表を追加（PR #2 で予告した分）。
