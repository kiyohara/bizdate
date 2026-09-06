# 作業ブランチメモ

- ブランチ: `add-pr-handling-to-maintain-progress`
- PR: PR 未作成
- 最終更新: 2026-09-06

## 目的

Issue #17 に従い、`maintain-progress` skill に `progress.md` 整理後の commit / PR の扱いを追記し、「参照する正本」節を新設する。あわせて `doc/guidelines/development-loop.md` の恒常的な例外を「索引登録と進捗整理」へ広げ、skill と正本の食い違いが生じないようにする。

Issue #5 / PR #6 が `register-progress-issue` について解消したのと同じ抜けが `maintain-progress` に残っている。

## 現在の状況

Issue #17 の作業内容 1〜3 をすべて実施し、検証 6 項目を確認した。PR 未作成。

## 決定事項

- **進捗整理は起点 Issue を持たない運用に確定する。** Issue #17 の「bizdate 固有の論点」が提示した 2 案のうち、起点 Issue なし側を採る。根拠は Issue 記載のとおり、(1) `development-loop.md` の標準フローで進捗整理が Issue 用意の step を経ていないこと、(2)「進捗整理を行うための Issue」が索引登録と同様に指示書として意味を持たないこと。slapex の扱いとも揃う。
- これに伴い Issue #17 の作業内容 3（`development-loop.md` の追随）を実施する。
- リリースは対象に含めない。bizdate に `release` skill がまだ無いため、skill 追加時に同じ扱いへ揃える旨を記述から読み取れる形にするに留める。

## 次にやること

- PR を作成し、note を採番する。
- ユーザーの review と merge 判断を待つ。

## 検証

Issue #17 の検証項目 6 件をすべて実施した。本リポジトリにはまだ CI もテストも無く、対象が document のみのため、すべて記述の読み合わせによる確認である。

| 項目 | 結果 |
|---|---|
| 追記が `development-loop.md` の基本方針（進捗整理は起点 Issue を作らず `Closes` を付けない）と一致する | 一致。同 PR で基本方針側を「索引登録と進捗整理」の 2 件へ広げ、skill 側の記述と揃えた |
| `register-progress-issue` step 6 / `number-working-branch-note` step 8 と表現が揃っている | 揃っている。「変更が生じなかった場合は、commit と PR を作らず、その旨を伝えて終了する」は `register-progress-issue` step 6 の文をそのまま踏襲した。専用ブランチ・独立 PR・note 採番・merge しないの各文も同様 |
| 既存の観点 1〜7、「やらないこと」、「終了時の確認」と矛盾しない | 矛盾なし。観点は編集内容の判断基準のままで、手順は独立した「commit と PR」節へ置いた。「やらないこと」に同梱と merge の 2 行、「終了時の確認」に独立 PR と `Closes` なしの 1 行を追加した |
| skill が参照する正本がすべて存在する | 10 件すべて存在を確認（`AGENTS.md`、`doc/guidelines/` の 8 件、`progress.md`） |
| `development-loop.md` の例外記述が `0012-development-loop.md` L65 および `progress.md` の開発環境整備の記述と矛盾しない | 矛盾なし。恒常的な例外を 2 件へ広げたうえで、開発ループ整備フェーズが「経緯による一度きりの扱い」であるという PR #6 で追加した一文はそのまま残した。0012 は「Issue の登録、実行、進捗整理、リリースの各手順を skill として用意する」と書くのみで、例外の件数を定めていない |
| `development-loop.md` の追記が `issue-driven-task-execution.md` step 7（索引表の行更新）を抑止しない | 抑止しない。追記の対象は `maintain-progress` が行う進捗整理という作業であり、Issue 駆動 PR の中で索引行の状態と PR 欄を更新する step 7 には触れていない。「他の作業 PR へ同梱しない」も `register-progress-issue` と同じく進捗整理による変更に限定して書いた |

## リスク・ブロッカー

- `development-loop.md` L9 の例外記述は、PR #6 の review で既存例外（開発ループ整備フェーズ）との衝突を指摘された箇所である。恒常的な例外を 2 件へ広げたため、同じ観点での再確認が要る。検証表のとおり確認済みだが、review でも重点的に見てほしい。
- 将来 `release` skill を追加するとき、リリースも同じ扱い（起点 Issue なし・独立 PR）へ揃える必要がある。今回は `development-loop.md` の基本方針に一文として残すに留め、skill 自体は作っていない。

## セッションログ

- 2026-09-06: Issue #17 を読み、依存（PR #6 の merge）が満たされていることを確認した。ブランチと note を作成した。
- 2026-09-06: `maintain-progress` skill に「参照する正本」節と「commit と PR」節を追加し、「やらないこと」「終了時の確認」を追随させた。
- 2026-09-06: `development-loop.md` の基本方針・標準フロー・資材表・参照順 7 の 4 箇所を、進捗整理を含む形へ修正した。
- 2026-09-06: Issue #17 の検証 6 項目を実施し、結果を検証セクションへ記録した。
