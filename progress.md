# 進捗管理表

このファイルには、設計および実装作業の進捗を記録していく。

想定読者は、実装作業を行う人間、および AI agent である。

横断的な作業状況（作業項目、状態、次にやること、検証状況、未解決のリスクやブロッカー）を一覧で把握するための管理表として使う。詳細な検討経緯は `doc/design/decision-log/` に置き、このファイルでは現在の作業状況を把握しやすく保つ。

## 運用メモ

- このファイルは、プロダクト全体の進捗を見渡すための一覧として使う。仕様設計や decision log ではなく、横断的な作業状況の管理表として扱う。
- ブランチ単位の作業目的・状況・判断は `working-branch-notes/` に置く。このファイルの 1 アイテムが必ずしも 1 ブランチに対応するとは限らない。
- 開発ループの入口は `doc/guidelines/development-loop.md`、Issue 駆動タスクの実行手順は `doc/guidelines/issue-driven-task-execution.md` を正本とする。方針の経緯は `doc/design/decision-log/0012-development-loop.md`。
- Issue 駆動タスクは、この表で依存確認と状態更新を行う。Issue 本文や guideline の手順をこのファイルに複製しない。
- リリースを開始したら、この表とは別に「リリース履歴」の節を設けてリリース台帳を兼ねる。
- この運用は暫定であり、実際の作業に合わせて軽く更新していく。

## 現況

v1 の CLI 実装と利用方法の整備（Issue #7〜#15 / PR #19、#20、#22〜#28）を完了し、続けて Dependabot による GitHub Actions の更新運用を導入した（Issue #21、#31 / PR #29、#32）。各フェーズの到達点と参照は「完了」の表にまとめる。

2026-09-09 時点で、進行中の横断プランは無い。open の PR は無く、open の Issue は [#30](https://github.com/kiyohara/bizdate/issues/30)（Dependabot 導入後の実動確認。上流の更新 PR 待ちの単発 Issue）だけで、索引には載せない。

## 進行中タスク

現在、索引する横断タスクは無い。新しい横断タスクが始まったら、`register-progress-issue` skill で状態・依存・参照（Issue / PR）が分かる最小の表をこの節に追加する。

## 次にやること

- #14 で「ローカルと CI で正しく動く CLI」に到達済みのため、配布手段（GitHub Releases / Homebrew / `dist`）を decision log で決め、Issue 化する。配布手段の詳細は decision log index で `open` のままである。
- Dependabot の更新 PR が出たら、そのレビューで #30 の残項目を確認して #30 に記録する。上流の release 待ちであり、いま実施する作業は無い。

## 完了

| 項目 | 状態 | メモ |
|---|---|---|
| 記録用ドキュメントの構成 | done | 配置ルール、decision log、agent 入口を導入した（0001） |
| プロダクト仕様の方針決定 | done | 0002–0009 を `decided`。`business-day.md` / `cli-interface.md` を正本化 |
| 開発体制の方針決定 | done | 0010–0013 を `decided`。ホスティング、agent 体制、開発ループ、ライセンス |
| 開発環境整備 | done | PR #1〜#4。agent 入口と作業記録の基盤、開発ループの正本、GitHub MCP 連携、開発ループ skill。ループ整備そのものは Issue を作らずフェーズ単位の PR で記録した（0012） |
| v1 実装 | done | Issue #7〜#15 / PR #19、#20、#22〜#28。Cargo と Compose の実行環境（0014）、CI（`fmt` / `clippy` / `test` / `build`）、日付とタイムゾーンの解決、祝日データの読み取り、業務日判定、`first` / `last` と `fetch-holidays` の CLI、統合テスト、README の利用方法（0001 追記） |
| Dependabot による action 更新の運用 | done | Issue #21 / PR #29、Issue #31 / PR #32（0015）。設定と運用境界を導入し、実動確認の残項目は #30 で追跡 |
