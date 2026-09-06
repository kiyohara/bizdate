<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 開発ループ

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `progress.md`, `AGENTS.md`, `doc/design/decision-log/0001-document-directory-structure.md`

## 背景

設計対話の記録は decision log と spec で回っているが、実装に移ると作業単位が増える。タスクの入力をどこに置き、作業単位をどう切り、経緯をどこに残し、どこでレビューして merge するかを決めないと、AI agent がセッションをまたぐたびに文脈を取り違える。

複数の AI agent を併用する（[0011](0011-ai-agent-lineup.md)）ため、agent が変わっても同じ流れを辿れることが要件になる。

## 候補

- A: 対話ベースで進め、`progress.md` だけで状態を管理する
- B: GitHub Issue を作業の入力とし、1 Issue = 1 ブランチ = 1 PR で進める
- C: 外部のプロジェクト管理ツールを併用する

## 検討内容

A 案は着手が軽いが、タスクの指示内容が対話ログに埋もれる。別セッションの agent が「何をどこまでやるべきか」を再構成できず、`progress.md` に詳細を書き込むと進捗表が指示書になって肥大化する。

C 案はリポジトリの外に正本が増える。AI agent がリポジトリ内から到達できない情報が生まれ、[0001](0001-document-directory-structure.md) の「AI と人間で別ドキュメントを持たない」方針と衝突する。

B 案では、Issue が作業の指示書、`progress.md` が横断的な索引、working branch note がブランチ内の作業ログ、PR が変更単位、decision log が設計判断、と役割が分かれる。役割が重ならないため、どこを読めばよいかが agent にとって一意に決まる。

PR の merge を agent に任せるかは分けて考える必要がある。レビューと merge 判断は人間が持ち、agent は PR 作成までを行う形にすると、変更がリポジトリに入る前に必ず人間の確認が挟まる。

作業単位を並行させると、同じファイルを触る PR が競合しやすく、AI agent が他ブランチの状態を誤認する。直列に消化する。

## 決定

- 作業は GitHub Issue から始める。Issue 本文をそのタスクの指示書とする。
- **1 Issue = 1 ブランチ = 1 PR** とする。複数 Issue を 1 ブランチにまとめない。
- タスクは直列に消化する。複数 Issue の並行作業はしない。
- **PR の merge は AI agent が行わない**。レビューと merge 判断は人間が行う。
- 各資材の役割を次のとおり分ける。

| 資材 | 役割 |
|---|---|
| GitHub Issue | 作業の入力。背景、依存、作業内容、スコープ外、検証を置く |
| `progress.md` | リリース台帳と進行中タスクの索引。詳細経緯やブランチ作業ログは置かない |
| Pull Request | 1 Issue に対する変更単位 |
| `working-branch-notes/` | ブランチ単位の作業目的、状況、判断、引き継ぎメモ。最終仕様書ではない |
| decision log | 後から辿る必要がある設計判断や方針変更の記録。進捗や作業ログは置かない |
| guideline | 人間と AI agent が共通で従う恒久的な作業ルール |
| skill | 特定の作業を始めるための実行手順 |

- 手順の正本は `doc/guidelines/` に置く。入口は `doc/guidelines/development-loop.md`、実行手順は `doc/guidelines/issue-driven-task-execution.md` とする。
- 開発ループを構成する skill（Issue 実行、進捗登録、進捗整理、note 採番、PR レビュー）は `.agents/skills/` に置く（[0011](0011-ai-agent-lineup.md)）。
- レビューは AI agent による PR レビューと GitHub Copilot code review を併用する。

## 理由

Issue を指示書として独立させると、どの agent がどのセッションで着手しても同じ入力から始められる。役割を資材ごとに分けることで、`progress.md` が指示書化したり decision log が作業ログ化したりする劣化を防げる。merge を人間に残すのは、変更がリポジトリに入る前の確認点を必ず 1 つ残すためである。

## 影響

- `working-branch-notes/` を配置し、取り扱いルールと情報統制ルールを `doc/guidelines/` に置く。
- `progress.md` をリリース台帳と進行中タスク索引の形式にする。
- Issue の登録、実行、進捗整理、リリースの各手順を skill として用意する。
- 開発ループの整備そのものは Issue が存在しない状態から始まるため、整備作業に限っては Issue を作らず、フェーズ単位のブランチと PR で記録する。

## 後から見直す条件

- 直列消化が開発速度のボトルネックになった場合。
- Issue の粒度が安定せず、指示書として機能しなくなった場合。
- 複数人での開発に移行し、merge 権限の分担が必要になった場合。
