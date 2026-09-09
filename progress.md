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

2026-09-09 の全体点検を踏まえ、次の横断プランとして配布準備（Issue #36〜#40）を登録した。既定の macOS / Linux・amd64 / arm64、GitHub Releases / dist / Homebrew を前提に、配布詳細の具体化から順に進める。個別実装と初回公開は未着手である。

[#30](https://github.com/kiyohara/bizdate/issues/30) は Dependabot の更新 PR 待ちの単発 Issue として継続する。索引には載せず、配布準備の依存にしない。

## 進行中タスク

配布準備を次の順で直列に消化する。依存欄は直前の必須依存だけを記載し、その Issue に対応する PR の merge を確認してから着手する。推移的な依存は先行 Issue の完了を通じて満たす。詳細な作業内容・検証・スコープ外は各 Issue を正とする。

| ID | Issue | 状態 | 依存 | 次にやること | PR |
|---|---|---|---|---|---|
| DIST-01 | [#36](https://github.com/kiyohara/bizdate/issues/36) | todo | なし | 配布対象・成果物・公開手順の詳細を具体化する | - |
| DIST-02 | [#37](https://github.com/kiyohara/bizdate/issues/37) | todo | #36 | 対象環境での CLI 検証を CI に追加する | - |
| DIST-03 | [#38](https://github.com/kiyohara/bizdate/issues/38) | todo | #37 | dist の成果物と Releases workflow を整備する | - |
| DIST-04 | [#39](https://github.com/kiyohara/bizdate/issues/39) | todo | #38 | Homebrew Formula 生成と更新連携を整備する | - |
| DIST-05 | [#40](https://github.com/kiyohara/bizdate/issues/40) | todo | #39 | リリース手順・skill・インストール案内を整備する | - |

## 次にやること

- 索引登録 PR の merge 後、`run-issue-task` で #36 から開始する。配布手段の詳細は decision log index で `open` のままであり、#36 で正本と decision log を整える。
- #36〜#40 は配布準備であり、実際の初回公開を完了扱いにしない。公開は #40 で整える手順に従ってユーザー承認後に進め、公開後の実動確認と未確認事項を記録する。
- Dependabot の更新 PR が出たら、そのレビューで #30 の残項目を確認して #30 に記録する。上流の release 待ちであり、いま実施する作業は無い。

## 完了

| 項目 | 状態 | メモ |
|---|---|---|
| 記録用ドキュメントの構成 | done | 配置ルール、decision log、agent 入口を導入した（0001） |
| プロダクト仕様の方針決定 | done | 0002–0009 を `decided`。`business-day.md` / `cli-interface.md` を正本化 |
| 開発体制の方針決定 | done | 0010–0013 を `decided`。ホスティング、agent 体制、開発ループ、ライセンス |
| 開発環境整備 | done | PR #1〜#4。agent 入口と作業記録の基盤、開発ループの正本、GitHub MCP 連携、開発ループ skill。ループ整備そのものは Issue を作らずフェーズ単位の PR で記録した（0012） |
| v1 実装 | done | Issue #7〜#15 / PR #19、#20、#22〜#28。Cargo と Compose の実行環境（0014）、CI（`fmt` / `clippy` / `test` / `build`）、日付とタイムゾーンの解決、祝日データの読み取り、業務日判定、`first` / `last` と `fetch-holidays` の CLI、統合テスト、README の利用方法（0001 追記）。後続で Copilot 指示の現況更新（Issue #34 / PR #35）も完了 |
| Dependabot による action 更新の運用 | done | Issue #21 / PR #29、Issue #31 / PR #32（0015）。設定と運用境界を導入し、実動確認の残項目は #30 で追跡 |
