# 進捗管理表

このファイルには、設計および実装作業の進捗を記録していく。

想定読者は、実装作業を行う人間、および AI agent である。

横断的な作業状況（作業項目、状態、次にやること、検証状況、未解決のリスクやブロッカー）を一覧で把握するための管理表として使う。詳細な検討経緯は `doc/design/decision-log/` に置き、このファイルでは現在の作業状況を把握しやすく保つ。

## 運用メモ

- このファイルは、プロダクト全体の進捗を見渡すための一覧として使う。仕様設計や decision log ではなく、横断的な作業状況の管理表として扱う。
- ブランチ単位の作業目的・状況・判断は `working-branch-notes/` に置く。このファイルの 1 アイテムが必ずしも 1 ブランチに対応するとは限らない。
- 開発ループの運用は `doc/design/decision-log/0012-development-loop.md` に従う。Issue 駆動の実行手順と skill は、開発環境整備で導入する。
- この運用は暫定であり、実際の作業に合わせて軽く更新していく。

## 現況

設計フェーズを完了した。プロダクト仕様（0002–0009）と開発体制の方針（0001、0010–0013）を decision log に記録し、`business-day.md` / `cli-interface.md` を仕様の正本とした。

次は開発環境の整備に進む。GitHub 上に開発ループ、AI agent 設定、レビュー体制を用意してから実装に入る。

## 進行中タスク: 開発環境整備

Issue 駆動ループそのものを用意する作業であり、この整備に限っては Issue を作らずフェーズ単位のブランチと PR で記録する（`doc/design/decision-log/0012-development-loop.md`）。

| # | 内容 | 状態 | PR |
|---|---|---|---|
| 1 | agent 入口と作業記録の基盤（agent 設定管理、working branch notes、PR ガイドライン、Copilot review 指示） | todo | - |
| 2 | 開発ループの正本（development-loop、Issue 駆動タスク実行、git / GitHub CLI 操作ルール、`progress.md` 再構成） | todo | - |
| 3 | GitHub MCP 連携（共通資材、利用ルール、各 tool の MCP 設定、worktree 補助） | todo | - |
| 4 | 開発ループ skill（Issue 実行、進捗登録、進捗整理、note 採番、PR レビュー） | todo | - |

## 次にやること

- 開発環境整備の 4 フェーズを順に進める。
- 整備完了後、実装タスクを GitHub Issue として登録し、Issue 駆動ループへ移行する。
- 実装がある程度形になった段階で、CI とリリース体制（`compose.yaml`、Cargo プロジェクト、開発コマンド実行ルール、GitHub Actions、`dist` による配布）を導入する。

## 完了

| 項目 | 状態 | メモ |
|---|---|---|
| 記録用ドキュメントの構成 | done | 配置ルール、decision log、agent 入口を導入した（0001） |
| プロダクト仕様の方針決定 | done | 0002–0009 を `decided`。`business-day.md` / `cli-interface.md` を正本化 |
| 開発体制の方針決定 | done | 0010–0013 を `decided`。ホスティング、agent 体制、開発ループ、ライセンス |

## 後続で追加する guideline

実装着手時に整備する。整備前は、これらを参照する記述を他のドキュメントや skill に書かない。

- 開発コマンド実行ルール（`compose.yaml` 経由での `cargo` 実行）: `compose.yaml` と Cargo プロジェクトを置くのと同じタイミングで追加する。
