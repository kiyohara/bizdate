# 作業ブランチメモ

- ブランチ: register-distribution-plan
- PR: #41
- 最終更新: 2026-09-09

## 目的

プロジェクト全体を点検し、次の配布整備を実行可能な Issue に分解して progress.md の索引へ登録する。

## 現在の状況

- 仕様正本、decision log index、Rust 実装・unit test・CLI E2E、README、Compose / CI、既存 Issue / PR を照合した。
- 配布整備の Issue #36〜#40 を作成した。個別実装は開始していない。
- 索引登録の独立 PR #41 を作成し、レビュー指摘 10 件を確認して計画を更新した。

## 決定事項

- 既定の macOS / Linux・amd64 / arm64、Releases / dist / Homebrew を前提に、仕様具体化、対象環境 CI、成果物 / workflow、Homebrew、手順整備の順で直列に進める。
- 新しい配布仕様は #36 で判断する。今回の索引登録では target triple、最低 OS、初回 version 等を決定しない。
- #30 は既存コメントが定義する上流更新 PR 待ちの単発 Issue。索引へ登録せず、新計画の依存にもしない。
- config、他国カレンダー、新しい日付演算、公開 library、Windows は現在の v1 対象外であり、新計画に追加しない。
- Cargo の依存更新の実装 Issue は今回追加しない。ただし 0015 の release workflow 追加時の見直し条件を #38 へ反映し、採否・保留の記録と必要な後続 Issue の起票を必須にした。
- #34 / PR #35 は既に完了しており、Copilot の古いフェーズ記述を重複起票しない。古い working branch note の記述は当時の記録として扱う。

## 次にやること

- 索引登録 PR のレビュー・merge 後、run-issue-task で #36 を実行する。
- 各実装 Issue の merge 後に次へ進む。実公開は #40 で整備する手順と承認に従う。

## 検証

- GitHub: 既存 Issue 15 件と PR 20 件を取得し、点検開始時の open Issue は #30 のみ、open PR は 0 件と確認した。
- 最新 main は 1472c4619bb0d51f1d2400182000ded256614fe6。CI run 34330331293 は success。
- 点検開始時の checkout と最新 main の差分は Copilot 指示と PR #35 の note のみ。実行コード・テスト・Cargo・Compose・Dockerfile・CI に差分なし。
- Compose: cargo test --locked は unit 69 件・CLI E2E 12 件成功。cargo fmt --check / cargo clippy --locked --all-targets -- -D warnings / cargo build --locked も成功。
- GitHub Releases は 0 件。既存 tap は Casks/slapex.rb を持つ。読み取りのみで確認した。
- dist 公式の設定資料と Homebrew installer 資料で、publish=false に対する配布対象指定と Formula / Cask の違いを確認した。詳細は #36 / #39 の一次資料リンクを参照する。
- Issue #36〜#40 の作成後、全件の title / state / body を読み戻して一致を確認した。依存と索引の行も照合した。
- progress.md の全節、参照先、git diff --check、note の情報統制を確認した。禁則の実値なし。
- 配布対象 4 環境の実行、release build / archive、実 Release、Homebrew install / upgrade、実際の内閣府 CSV 取得は今回未検証。後続 Issue の検証条件として明記した。

## リスク・ブロッカー

- #36 は未完了依存なし。後続 Issue は先行仕様・実装の merge 待ち。
- 初回公開に必要な権限設定・secrets・tag 操作は今回実施していない。実装 PR の完了と公開後の実動確認を分ける。

## セッションログ

- 2026-09-09: 全体点検、既存 Issue の重複確認、Compose 検証、配布整備 Issue の作成と索引登録。

## PR レビュー対応

対象 cycle: claude-code-01a98cf-20260909085302。今回が対応 1 周目である。

- third-party 表記: 0014 の先送りを確認し、#36 の依存全体の調査・生成・同梱方針、#38 の archive 同梱検証、#39 の Homebrew への保持確認を追加した。
- 正本境界: 配布物の仕様は #36 の spec、作業手順は #40 の guideline とした。先行実装に必要な運用判断は #36 の decision log に記録し、手順を重複させない。
- Copilot 同期: agent 設定管理ルールを確認し、#36 / #37 / #38 の検証項目へ追加した。
- required checks: #37 で設定を読み取り、影響・新旧 check 名・必要操作を PR と終了報告に示す。ユーザーが merge 前に更新し、読取不可は未設定と推定しない。
- runner: 公式一覧で macOS Intel と Linux arm64 の選択肢を確認した。実装時の再確認日・runner・検証手段を記録し、cross-build のみなら実行未検証・ブロッカーとする。
- Cargo 更新: 0015 の見直し条件を #38 で扱う。採用時は別 Issue、保留時は未決と再開条件を記録する。採用しないと決定した事項を未決へ強制登録しない。索引登録は独立 PR とする。
- 公開後追跡: #40 で公開担当者・公開実行前の起票と既存 Issue 再利用・公開後の証拠記録を定義する。README 切替は追跡 Issue の専用 PR で扱う。予定版案内は #40 の範囲に維持する。
- 表記: 依存を直前の先行 Issue へ統一し、#34 / PR #35 の完了参照を完了表へ移した。Issue の本文節順も揃えた。

検証は Issue 本文と索引・PR description の依存照合、各正本との整合、git diff --check、note の情報統制を行う。コード・CI の変更は無いため今回 cargo は再実行しない。required checks の現設定、実 runner job、ライセンス生成・同梱は今回未検証であり、後続の実行条件として追加した。

一次資料（2026-09-09 確認）:

- https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches
