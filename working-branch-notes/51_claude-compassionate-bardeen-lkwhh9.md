# 作業ブランチメモ

- ブランチ: `claude/compassionate-bardeen-lkwhh9`（cloud session が用意したブランチ）
- PR: #51
- 最終更新: 2026-09-12

## 目的

Issue #50 に従い、Issue 着手から「レビュー済み PR」までを 1 フローで回す orchestrator skill `drive-issue-to-reviewed-pr` を追加する。

ユーザーが毎回書き起こしていた prompt（実装と PR 作成 → review → 判断 → 対応 → 再確認 → 判断）を洗練し、subagent への委譲 interface、判断基準、head SHA と CI の確認点、停止条件を正本化する。

本 Issue は、本 Issue で作成する skill の手順そのもので処理する。skill の最初の実行例が自分自身の PR になり、それを再帰的な評価として扱う。

## 現在の状況

review cycle `claude-code-deeb92c-20260912122405` の 1 周目を終えた。P5 で 10 thread が resolve 可、1 thread が未対応（1 周目の修正が作った新たな矛盾）となったため、2 周目の P4 で対応した。2 周目の P5 へ進む。

## 決定事項

- skill 名は `drive-issue-to-reviewed-pr` とする。既存の動詞-目的語パターンからはやや外れるが、到達点（レビュー済み PR）が名前から読める点を採った。
- `references/` は作らず、委譲 brief の雛形を含めて SKILL.md 1 ファイルに収めた。brief の雛形だけを別ファイルにすると、SKILL.md の contract 表と実質同じ内容が 2 箇所に現れるため。
- 反復上限と処置の分類は `review-pull-request` に委ね、orchestrator へ複製しない。上限の正本を 1 箇所に保つ。
- 再確認を行う subagent は、review を実行した subagent の再利用を第一選択とし、失われている場合は GitHub 上の可視 metadata から context を再構築する新規 subagent へ fallback する。
- `progress.md` の索引には登録しない。単発 Issue として扱う。
- `.claude/rules/` と `.cursor/rules/` への入口は作らない。本件は skill であり rule ではない。

## 次にやること

- PR を作成し、note を採番する。（完了）
- SKILL.md の手順に従って review を subagent へ委譲する。SKILL.md の自己完結性も review 観点に含める。（完了）
- 指摘へ対応し、各 thread へ処置を返信する。（完了）
- 対応結果の再確認（P5）を subagent へ委譲し、収束を確認する。（1 周目完了）
- 2 周目の P5 で収束を確認する。

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

review 以降の結果。

| 項目 | 結果 |
|---|---|
| CI（head `cb48e89` / `deeb92c`） | `fmt / clippy / test / build` success |
| P2 review cycle | `claude-code-deeb92c-20260912122405`、対象 head `deeb92c` |
| P2 の指摘 | inline 11 件（must 6 / ask 1 / imo 3 / nits 1）+ review body 内 fyi 2 件 |
| P3 の判断 | 全 11 件を妥当と判断し採用。非採用・スコープ外とした指摘は無い |
| P4 の対応（1 周目） | SKILL.md を修正し、各 thread へ処置を返信した。PR description の未検証事項に `--from-pr` 入口の未実行を追記した |
| P5 の再確認（1 周目） | 10 thread が resolve 可。1 thread が未対応。依頼した整合確認（反復上限の 4 箇所一致、P3 分岐条件と判断基準、P5 の subagent 節と周辺）はいずれも通った。委譲中の push 禁止も既存記述と衝突しないと確認された |
| P4 の対応（2 周目） | 未対応 1 件（L63 / L204 の無条件禁止が L126 の許可と衝突）を修正。任意指摘 3 件（入口の表に P4 再開を載せる、再利用時に省略可の表記を 2 項目で揃える、ユーザーへ委ねる場合の P5 を定義する）も採用。任意指摘のうち 1 件は事実誤認のため非採用 |
| subagent 側の検証（委譲分） | `doc/guidelines/agent-configuration-management.md` の作成 checklist と禁止事項、backtick 付き repo 相対 path の実在、index リンク、文体（5 ファイル）、`git diff --check main..HEAD`、0019 の template 準拠、被委譲 2 skill の追記が 1 文であること、Issue #50 の記述要求項目の網羅 |

## 自己適用で得た所見

fresh context の subagent へ review を委譲した狙いは機能した。指摘 11 件のうち `[must]` 6 件は、いずれも設計意図を持つ側が context で補完して見落としていた自己矛盾・欠落である。

| 種別 | 内容 | 対応 |
|---|---|---|
| SKILL.md の自己矛盾 | 反復上限を「委ねる」と書きながら `2 周` の値を複製していた。0019 が却下した案の失敗形そのもの | 値を削り、単位の対応づけだけを残した |
| SKILL.md の自己矛盾 | P3 の分岐条件（処置すべき指摘が無ければ終了）が判断基準（全件非採用でも P4 を実施）と衝突 | 分岐条件を「指摘 0 件」に限定し、判断基準の 2 行を 1 行へ縮めた |
| 決定の落ち漏れ | 0019 の「再確認 subagent は再利用を第一選択」が SKILL.md に無く、本文は新規 subagent 前提に読めた | 「P5 の subagent」節を追加した |
| 経路の欠落 | subagent はユーザーへ直接問えないが、`review-pull-request` には「ユーザーに確認する」指示がある。中継経路が未定義だった | 確認事項を `未収束事項` として返し orchestrator が中継する扱いを明記し、停止表に行を足した |
| 入口の欠落 | `--from-pr` 入口に前提確認が掛からず、Issue 番号の取得元も無かった。未収束 cycle がある PR で cycle が二重になる | 前提確認、Issue 番号の取得元、未収束 cycle 時の P4 再開を追加した |
| 扱いの欠落 | CI が pending のときの進め方が無く、failure 時に先に直すのかも読めなかった | pending / 自 PR 起因の failure / 外部起因の failure の 3 分岐を書いた |
| fallback の範囲 | subagent 不在時の fallback が P2 だけを対象にしており、P5 と担当一致要件の関係が未定義だった | P2 / P5 の両方を対象にし、別 Agent 種別へ委ねた場合の担当一致を明記した |

**修正が新たな矛盾を作ることがある**、という所見も得た。1 周目で [ask] に応えて「P5 を orchestrator 自身で実行しない」を 2 箇所へ無条件の禁止として書いたところ、「subagent が使えない環境」節の「同一 agent が P2 と P5 を実行してよい」と衝突した。subagent 機構が無い環境では実行主体が orchestrator ただ 1 つであるため、一方が許すものを他方が禁じる形になっていた。2 周目で例外の相互参照を入れて閉じた。1 周で終わらせず P5 を回す設計が効いた例である。

非採用とした指摘も 1 件ある。「`development-loop.md` の引用箇所が『各資材の役割』ではなく『基本方針』である」という任意指摘だが、当該規定（索引登録と進捗整理は起点 Issue を持たない独立 PR となるため `Closes` を付けない）は同ファイル 48 行目の「各資材の役割」表の Pull Request 行にある。引用は正しく、修正は不要と判断した。

フロー運用そのものについて、この session で 2 件気づいた。

1. **`number-working-branch-note` にユーザー合意ゲートがある。** 採番は PR 作成直後に必ず通る経路であり、完全自律のフローにならない。現状の運用に合っていないため Issue #52 として登録した。orchestrator 側に記載すべきかは #52 の結論に従う。
2. **orchestrator は委譲中に push できない。** push すると head SHA が動き、review 中の subagent が context の取り直しを強いられる。SKILL.md の「head SHA と CI」と「やらないこと」に明記した。

fyi として、委譲先の fresh subagent 側でも新規 skill が skill 一覧に載っていた。session 途中の検出は subagent の起動時にも効いている。ただし SKILL.md は引き続き path 直読を既定とする。

## リスク・ブロッカー

- 未検証: description による発火。skill が一覧に載ることは確認したが、意図した場面で実際に発火するかはその場面を再現しない限り確認できない。ユーザーがローカルの Claude Code を再起動するか新しい cloud session を開いた際の確認事項として残す。
- 自己適用の盲点: skill の設計意図を持つ agent が SKILL.md を読むため、記述不足を context で補完して動けてしまう。fresh context の subagent へ「SKILL.md の自己完結性」を review 観点として渡すことで補う。この観点から上がる指摘を本 PR の主要な検証出力として扱う。
- `github-op-integrated` MCP は cloud session で起動に失敗する。想定どおりであり、組み込み GitHub tool で進める（`doc/guidelines/cloud-session-guidelines.md`）。

## セッションログ

- 2026-09-12: 常用 prompt の洗練点（委譲 interface、判断基準、head SHA の受け渡し、CI の確認点、反復上限の一元化、停止条件、途中再開、note 更新点、終了報告、agent 中立性）を整理し、Issue #50 として起票した。
- 2026-09-12: skill 本体・symlink・`doc/guidelines/development-loop.md`・被委譲 2 skill の参照・decision log 0019 と index を作成した。session 途中の skill 検出を実測し、SKILL.md の記述を実測に合わせて調整した。
- 2026-09-12: PR #51 を作成し、note を採番した。CI success を確認し、review cycle `claude-code-deeb92c-20260912122405` を subagent へ委譲した（対象 head `deeb92c`）。
- 2026-09-12: 指摘 11 件を実物で確認し、全件を採用して SKILL.md を修正した。採番の合意ゲートを Issue #52 として登録した。委譲中の push 禁止を SKILL.md に追加した。
- 2026-09-12: P5（1 周目）を review 担当の subagent 再利用で実行した。10 thread が resolve 可、1 thread が未対応。未対応 1 件と任意指摘 3 件を 2 周目の P4 で修正し、事実誤認の 1 件は非採用とした。
