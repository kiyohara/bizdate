# 作業ブランチメモ

- ブランチ: `lift-delegated-skill-report-items`
- PR: （採番前）
- 最終更新: 2026-09-13

## 目的

Issue #55。`number-working-branch-note` が報告する「完了として書き換えたタスク行の一覧」を、上位 skill（`run-issue-task` / `drive-issue-to-reviewed-pr`）の報告項目へ引き上げるかを判断し、結論に合わせて SKILL.md と decision log 0020 を更新する。

## 現在の状況

- 依存の PR #54 は merge 済み。0020 の「後から見直す条件」から #55 を参照する変更（`ed16a64`）も `main` に入っている。

## 決定事項

- 案 A を採用し、現時点の該当項目を名指しで併記する。
  - 一般規定: 被委譲 skill が自分の報告項目のうち「ユーザーの確認経路」と位置づけた項目は、上位 skill が要約で落とさず自分の報告へ含める。判定を「ユーザーの確認を要するか」という上位側の判断に委ねず、被委譲 skill 側の明記に寄せる。
  - 名指し: `number-working-branch-note` の「完了として書き換えたタスク行の一覧」。書き換えが 0 件でもその旨を報告する。
- B 単独は後続 skill に効かない。C は 0020 の「報告を省いた実行は前提を欠く」と衝突し、merge 前レビューでの気づきに依存する。
- decision log は 0020 への追記に留める（新しい方針の導入ではなく、0020 が記録した制約を規定で閉じる変更のため）。

## 次にやること

- [x] SKILL.md 2 件と 0020 / index を更新する
- [x] 検証を実施する
- [ ] PR を作成し、note を採番する

## 検証

- `number-working-branch-note` の「終了時の報告」との整合: 通る。書き換えた行の一覧はそのまま引き上げ、触らなかった行の一覧は `run-issue-task` で未解決事項として扱う。同 skill の報告項目 6 件と食い違う記述は無い。
- 両経路の辿り: 単独実行は step 9 → step 10（「被委譲 skill の報告の引き上げ」参照）、orchestrated flow は P1 → note の `セッションログ` → 「終了時の報告」の P1 引き上げ項目で辿れる。
- 他の被委譲 skill: `review-pull-request` は既存の報告項目（resolve 可 thread 一覧、人間に残る作業、未収束事項）で閉じている。`maintain-progress` / `register-progress-issue` は委譲経路が無い。
- 変更行の backtick path と 0020 の markdown link: 切れなし（スクリプトで実在確認）。
- 文体: 変更行にですます調の混在なし。
- `git diff --check`: 問題なし。
- `cargo`: Rust のコード変更が無いため省略。

## リスク・ブロッカー

- 引き上げ規定が実際の orchestrated flow で落ちずに機能するかは、次に採番で書き換えが発生する実行まで確認できない（未検証）。

## セッションログ

- 2026-09-13: Issue #55 を読み、依存（PR #54 merge 済み）を確認。`main` から作業ブランチを作成。
- 2026-09-13: 案 A（被委譲 skill 側の位置づけを根拠にする一般規定）+ 現時点の該当項目の名指しを採用し、SKILL.md 2 件、0020、index を更新。検証を記録。
