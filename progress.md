# 進捗管理表

このファイルには、設計および実装作業の進捗を記録していく。

想定読者は、実装作業を行う人間、および AI agent である。

横断的な作業状況（作業項目、状態、次にやること、検証状況、未解決のリスクやブロッカー）を一覧で把握するための管理表として使う。詳細な検討経緯は `doc/design/decision-log/` に置き、このファイルでは現在の作業状況を把握しやすく保つ。

## 運用メモ

- このファイルは、プロダクト全体の進捗を見渡すための一覧として使う。仕様設計や decision log ではなく、横断的な作業状況の管理表として扱う。
- ブランチ単位の作業目的・状況・判断は `working-branch-notes/` に置く。このファイルの 1 アイテムが必ずしも 1 ブランチに対応するとは限らない。
- 開発ループの入口は `doc/guidelines/development-loop.md`、Issue 駆動タスクの実行手順は `doc/guidelines/issue-driven-task-execution.md` を正本とする。方針の経緯は `doc/design/decision-log/0012-development-loop.md`。
- Issue 駆動タスクは、この表で依存確認と状態更新を行う。Issue 本文や guideline の手順をこのファイルに複製しない。
- リリース台帳は「リリース履歴」の節に置く。列と更新の責任は同節に書き、公開の手順は `doc/guidelines/release-guidelines.md` を正本とする。
- この運用は暫定であり、実際の作業に合わせて軽く更新していく。

## 現況

v1 の CLI 実装と利用方法の整備（Issue #7〜#15 / PR #19、#20、#22〜#28）を完了し、続けて Dependabot による GitHub Actions の更新運用を導入した（Issue #21、#31 / PR #29、#32）。各フェーズの到達点と参照は「完了」の表にまとめる。

2026-09-09 の全体点検を踏まえ、次の横断プランとして配布準備（Issue #36〜#40）を登録した。最初の #36 で配布仕様を確定し、正本を `doc/design/distribution.md`、経緯を decision log 0016 に置いた。続く #37 で配布対象 4 target の native CI と最低 glibc の実測を `.github/workflows/ci.yml` に置き、#38 で `dist` の設定と release workflow（PR での archive の作成と検証、`v` tag の push での公開）を置いた（decision log 0022）。2026-09-23 に、PR ごとの CI の待ち時間を縮めるため、配布対象から Intel Mac を外す #64 と、cargo-about を prebuilt で導入する #65 を #39 の前に加え、#64 で配布対象を 3 target にした。続く #39 で Homebrew Formula の生成・検証と tap への書き込みを置き（decision log 0025）、#40 でリリース手順（`doc/guidelines/release-guidelines.md` と `run-release` skill）、README の公開予定のインストール案内、リリース台帳を整えた（decision log 0026）。2026-09-25 に初回公開として `v0.1.0` を公開し、公開後確認を終えた（[#84](https://github.com/kiyohara/bizdate/issues/84)。記録は「リリース履歴」）。

2026-09-12 に、Claude Code on the web の cloud session でも Compose 経由の開発コマンドと GitHub 操作が成立するよう実行環境と運用ルールを整えた（Issue #47 / PR #48、decision log 0018）。正本は `doc/guidelines/cloud-session-guidelines.md`。配布準備の状況は変わっていない。

2026-09-23 に、PR #67 の review cycle を cloud session で回して見つかった課題を #68〜#70 として起票した。model の識別子の扱い、cloud session での review metadata、`gh` の利用であり、「進行中タスク」の「cloud session での agent 運用」の表で追跡する。

[#30](https://github.com/kiyohara/bizdate/issues/30) は Dependabot の更新 PR 待ちの単発 Issue として継続する。索引には載せず、配布準備の依存にしない。

## 進行中タスク

### 配布準備

配布準備を次の順で直列に消化する。依存欄は直前の必須依存だけを記載し、その Issue に対応する PR の merge を確認してから着手する。推移的な依存は先行 Issue の完了を通じて満たす。詳細な作業内容・検証・スコープ外は各 Issue を正とする。

| ID | Issue | 状態 | 依存 | 次にやること | PR |
|---|---|---|---|---|---|
| DIST-01 | [#36](https://github.com/kiyohara/bizdate/issues/36) | done | なし | 完了。配布仕様を `doc/design/distribution.md` に、経緯を 0016 に置いた | [#42](https://github.com/kiyohara/bizdate/pull/42) |
| DIST-02 | [#37](https://github.com/kiyohara/bizdate/issues/37) | done | #36 | 完了。4 target の native `platform` job と最低 glibc の実測を CI に置いた | [#61](https://github.com/kiyohara/bizdate/pull/61) |
| DIST-03 | [#38](https://github.com/kiyohara/bizdate/issues/38) | done | #37 | 完了。`dist` の設定と release workflow を置いた。初回公開は `v0.1.0` で行った（#84） | [#62](https://github.com/kiyohara/bizdate/pull/62) |
| DIST-03a | [#64](https://github.com/kiyohara/bizdate/issues/64) | done | #38 | 完了。配布対象を 3 target にし、Intel Mac（`x86_64-apple-darwin`）を外した | [#67](https://github.com/kiyohara/bizdate/pull/67) |
| DIST-03b | [#65](https://github.com/kiyohara/bizdate/issues/65) | done | #64 | 完了。cargo-about を upstream の prebuilt（固定した sha256 と照合）で入れ、PR の run では `custom-ci` から `platform` を省いた | [#76](https://github.com/kiyohara/bizdate/pull/76) |
| DIST-04 | [#39](https://github.com/kiyohara/bizdate/issues/39) | done | #65 | 完了。Formula を dist で生成して検査・test 追加し、PR で公開前の archive から install を確かめ、tag push で custom の publish job が tap へ書く構成を置いた（0025）。tap への実公開は `v0.1.0` で確かめた（#84） | [#79](https://github.com/kiyohara/bizdate/pull/79) |
| DIST-05 | [#40](https://github.com/kiyohara/bizdate/issues/40) | done | #39 | 完了。リリース手順を guideline と `run-release` skill に、公開予定のインストール案内を README に、リリース台帳を「リリース履歴」に置いた。初回公開は `v0.1.0` で行った（#84） | [#83](https://github.com/kiyohara/bizdate/pull/83) |

### cloud session での agent 運用

PR #67 の review cycle を cloud session で回して見つかった課題を追跡する。依存欄の書き方は配布準備の表と同じとする。配布準備とは独立しており、どちらを先に進めるかはユーザーが決める。

| ID | Issue | 状態 | 依存 | 次にやること | PR |
|---|---|---|---|---|---|
| OPS-01 | [#68](https://github.com/kiyohara/bizdate/issues/68) | done | なし | 完了。model の識別子の禁止を PR title に限り、review metadata の `Model` は PR / review のコメントが実行環境の指示の対象外なら `unknown` にしないと定めた（0023） | [#72](https://github.com/kiyohara/bizdate/pull/72) |
| OPS-02 | [#70](https://github.com/kiyohara/bizdate/issues/70) | done | なし | 完了。`--provision` が `gh` を入れ、MCP tool に無い操作だけ `gh api` で補う線引きを規約と 0018 に置いた。許可リストへの host 追加は見送り | [#74](https://github.com/kiyohara/bizdate/pull/74) |
| OPS-03 | [#69](https://github.com/kiyohara/bizdate/issues/69) | done | #68 | 完了。canonical metadata を footer の直前に置き、parse を 5 キーの並びで行うと定めた。`Model` の確認手段（cloud session の本体は `get_session`、subagent は自身の system prompt）、投稿前の確認と編集による訂正を加えた（0024） | [#78](https://github.com/kiyohara/bizdate/pull/78) |

## 次にやること

- 初回公開（`v0.1.0`）は 2026-09-25 に行い、公開後確認を終えた（「リリース履歴」）。前の version からの `brew upgrade` は初回では確かめられないため、次の version の公開後確認で確かめる（[#85](https://github.com/kiyohara/bizdate/issues/85)）。tap の README への bizdate の Formula の案内は、ユーザーが tap 側で足す（`doc/guidelines/release-guidelines.md` の「初回公開（v0.1.0）」）。
- cloud session での agent 運用（#68〜#70）は #69 で完了した。表は、次の整理で「完了」の表へ移す。
- #38 で 0015 を再判断し、Cargo の Dependabot version updates と Dependabot alerts / security updates を採用した。設定は [#63](https://github.com/kiyohara/bizdate/issues/63) で入れ、merge 後の実動確認は [#73](https://github.com/kiyohara/bizdate/issues/73) で追う（いずれも索引には載せない単発の Issue）。alerts と security updates の有効化は repository settings の操作であり、ユーザーが行う。
- 配布準備（#36〜#40 と #64 / #65）は、`v0.1.0` の公開と公開後確認で到達点を確かめた。表は、次の整理で「完了」の表へ移す。以後も、公開後の確認と未確認の項目は version ごとの公開後確認 Issue で追う。
- Dependabot の更新 PR が出たら、そのレビューで #30 の残項目を確認して #30 に記録する。上流の release 待ちであり、いま実施する作業は無い。

## リリース履歴

公開した version の台帳である。公開の手順は `doc/guidelines/release-guidelines.md` を正本とする。

- 行は、公開後確認を終えた version について、公開担当者がその version の公開後確認 Issue を入力とする PR で足す。公開していない version、中止した version、確認を終えていない version の行は置かない。
- 列には次を書く。version は公開した version（例: `0.1.0`）、公開日は Release が公開された日（UTC、`YYYY-MM-DD`）、Release は Release の URL、検証は公開後確認 Issue へのリンクと結果の要約、未確認は残った項目の追跡 Issue へのリンク（無ければ「なし」）。
- 「未確認」の追跡 Issue が完了したら、次にこの表を変える PR（次の version の行を足す PR、または進捗整理）で「なし」に更新する。
- 行は履歴として残し、削除や圧縮をしない。

| version | 公開日 | Release | 検証 | 未確認 |
|---|---|---|---|---|
| `0.1.0` | 2026-09-25 | [v0.1.0](https://github.com/kiyohara/bizdate/releases/tag/v0.1.0) | [#84](https://github.com/kiyohara/bizdate/issues/84)。3 target の archive、third-party 表記、macOS での Homebrew の install と `brew test`、対応環境の値は ok。既定 CSV の取得は arm64 の 2 target で ok（`x86_64-unknown-linux-gnu` はユーザーの判断で確かめない） | [#85](https://github.com/kiyohara/bizdate/issues/85)（前の version からの `brew upgrade`） |

## 完了

| 項目 | 状態 | メモ |
|---|---|---|
| 記録用ドキュメントの構成 | done | 配置ルール、decision log、agent 入口を導入した（0001） |
| プロダクト仕様の方針決定 | done | 0002–0009 を `decided`。`business-day.md` / `cli-interface.md` を正本化 |
| 開発体制の方針決定 | done | 0010–0013 を `decided`。ホスティング、agent 体制、開発ループ、ライセンス |
| 開発環境整備 | done | PR #1〜#4。agent 入口と作業記録の基盤、開発ループの正本、GitHub MCP 連携、開発ループ skill。ループ整備そのものは Issue を作らずフェーズ単位の PR で記録した（0012） |
| v1 実装 | done | Issue #7〜#15 / PR #19、#20、#22〜#28。Cargo と Compose の実行環境（0014）、CI（`fmt` / `clippy` / `test` / `build`）、日付とタイムゾーンの解決、祝日データの読み取り、業務日判定、`first` / `last` と `fetch-holidays` の CLI、統合テスト、README の利用方法（0001 追記）。後続で Copilot 指示の現況更新（Issue #34 / PR #35）も完了 |
| Dependabot による action 更新の運用 | done | Issue #21 / PR #29、Issue #31 / PR #32（0015）。設定と運用境界を導入し、実動確認の残項目は #30 で追跡 |
| cloud session（Claude Code on the web）対応 | done | Issue #47 / PR #48（0018）。SessionStart hook で Docker daemon を起動して Compose 経由を維持し、GitHub 操作は組み込み tool で行う。正本は `doc/guidelines/cloud-session-guidelines.md` |
