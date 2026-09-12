# 作業ブランチメモ

- ブランチ: `claude/compassionate-bardeen-lkwhh9`（cloud session が用意したブランチ）
- PR: #51
- 最終更新: 2026-09-12

## 目的

Issue #50 に従い、Issue 着手から「レビュー済み PR」までを 1 フローで回す orchestrator skill `drive-issue-to-reviewed-pr` を追加する。

ユーザーが毎回書き起こしていた prompt（実装と PR 作成 → review → 判断 → 対応 → 再確認 → 判断）を洗練し、subagent への委譲 interface、判断基準、head SHA と CI の確認点、停止条件を正本化する。

本 Issue は、本 Issue で作成する skill の手順そのもので処理する。skill の最初の実行例が自分自身の PR になり、それを再帰的な評価として扱う。

## 現在の状況

skill 本体、symlink、`doc/guidelines/development-loop.md` の skill 表、被委譲 2 skill の相互参照、decision log 0019 と index 行を作成した。文書検証を実施し、PR 作成へ進む。

## 決定事項

- skill 名は `drive-issue-to-reviewed-pr` とする。既存の動詞-目的語パターンからはやや外れるが、到達点（レビュー済み PR）が名前から読める点を採った。
- `references/` は作らず、委譲 brief の雛形を含めて SKILL.md 1 ファイルに収めた。brief の雛形だけを別ファイルにすると、SKILL.md の contract 表と実質同じ内容が 2 箇所に現れるため。
- 反復上限と処置の分類は `review-pull-request` に委ね、orchestrator へ複製しない。上限の正本を 1 箇所に保つ。
- 再確認を行う subagent は、review を実行した subagent の再利用を第一選択とし、失われている場合は GitHub 上の可視 metadata から context を再構築する新規 subagent へ fallback する。
- `progress.md` の索引には登録しない。単発 Issue として扱う。
- `.claude/rules/` と `.cursor/rules/` への入口は作らない。本件は skill であり rule ではない。

## 次にやること

- PR を作成し、note を採番する。
- SKILL.md の手順に従って review を subagent へ委譲する。SKILL.md の自己完結性も review 観点に含める。
- 指摘の採否を判断し、必要なら対応と再確認を回す（上限 2 周）。

## 検証

| 項目 | 結果 |
|---|---|
| `test -f .claude/skills/drive-issue-to-reviewed-pr/SKILL.md` | 通る。symlink 経由で SKILL.md 全文が読める |
| symlink の target | `../../.agents/skills/drive-issue-to-reviewed-pr`（相対、末尾 `/` なし） |
| broken symlink | `find -L .claude/skills -maxdepth 1 -type l` で検出なし |
| frontmatter | `name` / `description` / `argument-hint` を持ち、既存 skill と同形。`name` は正本ディレクトリ名と symlink 名に一致 |
| repo 相対 path と markdown link | 切れなし |
| 文体（常体） | 追加・変更文書で混在なし |
| `git diff --check` | 問題なし |
| session 途中の skill 検出 | **検出された。** `.claude/skills/` の symlink と `SKILL.md` を作成した直後、session の再起動なしに skill が利用可能として通知された（cloud session、2026-09-12 実測）。ただし検出は保証されないため、SKILL.md は path 直読を既定としている |

Rust のコードに変更が無いため、`cargo` による再検証は省略した。

review 以降の検証結果は、PR 採番後に本セクションへ追記する。

## リスク・ブロッカー

- 未検証: description による発火。skill が一覧に載ることは確認したが、意図した場面で実際に発火するかはその場面を再現しない限り確認できない。ユーザーがローカルの Claude Code を再起動するか新しい cloud session を開いた際の確認事項として残す。
- 自己適用の盲点: skill の設計意図を持つ agent が SKILL.md を読むため、記述不足を context で補完して動けてしまう。fresh context の subagent へ「SKILL.md の自己完結性」を review 観点として渡すことで補う。この観点から上がる指摘を本 PR の主要な検証出力として扱う。
- `github-op-integrated` MCP は cloud session で起動に失敗する。想定どおりであり、組み込み GitHub tool で進める（`doc/guidelines/cloud-session-guidelines.md`）。

## セッションログ

- 2026-09-12: 常用 prompt の洗練点（委譲 interface、判断基準、head SHA の受け渡し、CI の確認点、反復上限の一元化、停止条件、途中再開、note 更新点、終了報告、agent 中立性）を整理し、Issue #50 として起票した。
- 2026-09-12: skill 本体・symlink・`doc/guidelines/development-loop.md`・被委譲 2 skill の参照・decision log 0019 と index を作成した。session 途中の skill 検出を実測し、SKILL.md の記述を実測に合わせて調整した。
