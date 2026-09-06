# 作業ブランチメモ

- ブランチ: `add-pr-handling-to-maintain-progress`
- PR: PR 未作成
- 最終更新: 2026-09-06

## 目的

Issue #17 に従い、`maintain-progress` skill に `progress.md` 整理後の commit / PR の扱いを追記し、「参照する正本」節を新設する。あわせて `doc/guidelines/development-loop.md` の恒常的な例外を「索引登録と進捗整理」へ広げ、skill と正本の食い違いが生じないようにする。

Issue #5 / PR #6 が `register-progress-issue` について解消したのと同じ抜けが `maintain-progress` に残っている。

## 現在の状況

作業ブランチと note を作成した。実装はこれから。

## 決定事項

- **進捗整理は起点 Issue を持たない運用に確定する。** Issue #17 の「bizdate 固有の論点」が提示した 2 案のうち、起点 Issue なし側を採る。根拠は Issue 記載のとおり、(1) `development-loop.md` の標準フローで進捗整理が Issue 用意の step を経ていないこと、(2)「進捗整理を行うための Issue」が索引登録と同様に指示書として意味を持たないこと。slapex の扱いとも揃う。
- これに伴い Issue #17 の作業内容 3（`development-loop.md` の追随）を実施する。
- リリースは対象に含めない。bizdate に `release` skill がまだ無いため、skill 追加時に同じ扱いへ揃える旨を記述から読み取れる形にするに留める。

## 次にやること

- 実装、検証、PR 作成。

## 検証

未実施。

## リスク・ブロッカー

- `development-loop.md` L9 の例外記述は PR #6 の review で既存例外（開発ループ整備フェーズ）との衝突を指摘された箇所である。恒常的な例外を 2 件へ広げるにあたり、`doc/design/decision-log/0012-development-loop.md` L65 および `progress.md` の開発環境整備の記述と再び衝突しないか確認する。

## セッションログ

- 2026-09-06: Issue #17 を読み、依存（PR #6 の merge）が満たされていることを確認した。ブランチと note を作成した。
