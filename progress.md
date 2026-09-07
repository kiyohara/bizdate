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

設計フェーズと開発環境整備を完了した。プロダクト仕様（0002–0009）と開発体制の方針（0001、0010–0013）を decision log に記録し、`business-day.md` / `cli-interface.md` を仕様の正本とした。開発ループ、AI agent 設定、GitHub MCP 連携、レビュー体制、skill は PR #1〜#4 で整備済み。

v1 実装に着手した。Issue #7〜#15 を進行中タスクとして追跡する。#7 で Cargo プロジェクト、Compose の実行環境、依存クレートの選定（0014）を置き、#8 で GitHub Actions の CI を入れた。#9 で日付とタイムゾーンの解決 API、#10 でローカル祝日 CSV の読み取りと期限・年カバーの検証、#11 で利用者指定休日と業務日・月の最初 / 最後の判定を実装し、#12 で判定サブコマンド `first` / `last` の CLI を配線して exit code 0 / 1 / 2 の写像を入れ、#13 で `fetch-holidays` の取得・UTF-8 変換・保存を実装した。#14 で実バイナリを起動する統合テストを整備し、取得から判定までの往復とエラー時の exit code を検証した。実装 PR は `fmt` / `clippy` / `test` / `build` の検査を受ける。

## 進行中タスク: 開発環境整備

Issue 駆動ループそのものを用意する作業であり、この整備に限っては Issue を作らずフェーズ単位のブランチと PR で記録する（`doc/design/decision-log/0012-development-loop.md`）。

| # | 内容 | 状態 | PR |
|---|---|---|---|
| 1 | agent 入口と作業記録の基盤（agent 設定管理、working branch notes、PR ガイドライン、Copilot review 指示） | done | [#1](https://github.com/kiyohara/bizdate/pull/1) |
| 2 | 開発ループの正本（development-loop、Issue 駆動タスク実行、git / GitHub CLI 操作ルール、`progress.md` 再構成） | done | [#2](https://github.com/kiyohara/bizdate/pull/2) |
| 3 | GitHub MCP 連携（共通資材、利用ルール、各 tool の MCP 設定、worktree 補助） | done | [#3](https://github.com/kiyohara/bizdate/pull/3) |
| 4 | 開発ループ skill（Issue 実行、進捗登録、進捗整理、note 採番、PR レビュー） | done | [#4](https://github.com/kiyohara/bizdate/pull/4) |

## 進行中タスク: v1 実装

表の順を推奨順とし、直列に消化する。依存欄は必須条件のみを記載し、単なる推奨順は含めない。

| ID | Issue | 状態 | 依存 | 次にやること | PR |
|---|---|---|---|---|---|
| V1-01 | [#7](https://github.com/kiyohara/bizdate/issues/7) Cargo と開発環境の骨組み | done | - | - | [#19](https://github.com/kiyohara/bizdate/pull/19) |
| V1-02 | [#8](https://github.com/kiyohara/bizdate/issues/8) CI workflow | done | #7 | - | [#20](https://github.com/kiyohara/bizdate/pull/20) |
| V1-03 | [#9](https://github.com/kiyohara/bizdate/issues/9) 日付とタイムゾーンの解決 | done | #7 | - | [#22](https://github.com/kiyohara/bizdate/pull/22) |
| V1-04 | [#10](https://github.com/kiyohara/bizdate/issues/10) 祝日データのローカル読み取り | done | #7 | - | [#23](https://github.com/kiyohara/bizdate/pull/23) |
| V1-05 | [#11](https://github.com/kiyohara/bizdate/issues/11) 業務日判定と月の最初 / 最後 | done | #9, #10 | - | [#24](https://github.com/kiyohara/bizdate/pull/24) |
| V1-06 | [#12](https://github.com/kiyohara/bizdate/issues/12) 判定サブコマンドの CLI | done | #11 | - | [#25](https://github.com/kiyohara/bizdate/pull/25) |
| V1-07 | [#13](https://github.com/kiyohara/bizdate/issues/13) `fetch-holidays` | done | #10, #12 | - | [#26](https://github.com/kiyohara/bizdate/pull/26) |
| V1-08 | [#14](https://github.com/kiyohara/bizdate/issues/14) 統合テストと E2E | done | #12, #13 | - | - |
| V1-09 | [#15](https://github.com/kiyohara/bizdate/issues/15) README の利用方法 | todo | #14 | `doc/help/` の要否を判断する | - |

## 次にやること

- v1 実装の索引を V1-09 から直列に消化する。
- 完了済みの開発環境整備の表は、`maintain-progress` skill でフェーズ要約へ圧縮する。
- #14 で「ローカルと CI で正しく動く CLI」に到達した時点で、配布手段（GitHub Releases / Homebrew / `dist`）を decision log で決め、Issue 化する。配布手段の詳細は decision log index で `open` のままである。

## 完了

| 項目 | 状態 | メモ |
|---|---|---|
| 記録用ドキュメントの構成 | done | 配置ルール、decision log、agent 入口を導入した（0001） |
| プロダクト仕様の方針決定 | done | 0002–0009 を `decided`。`business-day.md` / `cli-interface.md` を正本化 |
| 開発体制の方針決定 | done | 0010–0013 を `decided`。ホスティング、agent 体制、開発ループ、ライセンス |
