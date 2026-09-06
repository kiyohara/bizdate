# design

このディレクトリには、`bizdate` の仕様設計と設計判断を置く。

## 置くもの

- 利用者体験や CLI 挙動の設計文書
- 業務日判定、祝日、出力、制限値などの仕様設計
- 設計判断の履歴である decision log
- 実装前に合意したい仕様の素案

## 置かないもの

- 利用者がそのまま手順として読む help: CLI の形が固まってから `doc/help/` に置く
- AI agent / Git / GitHub / PR などの作業ルール: `doc/guidelines/` に置く
- 作業状況の一覧: `progress.md` に置く

## 主な文書

- `concept.md`: 現在合意しているコンセプトの正本
- `business-day.md`: 日付・業務日・祝日の仕様正本
- `cli-interface.md`: CLI の option・exit code・入出力の仕様正本
- `decision-log/`: 方針決定ログ
