---
name: maintain-progress
description: bizdate の `progress.md` を定期的に整理し、進行中タスクの索引として読みやすく保つ。役目を終えた完了タスク表を decision log / Issue / PR への参照を残した要約へ圧縮し、現況と進行中タスクの索引を最新化し、AGENTS.md / 各 guideline / rule が `progress.md` を指す前提(参照整合性)と、decision log・working-branch-notes との境界を維持する。観点別チェックで安全側に整理し、迷う点はユーザーに確認する。
---

# maintain-progress

`progress.md` を整理して、横断的な作業状況ボードとして読みやすく保つための skill。

`progress.md` は進行中タスクの索引であり、`doc/guidelines/issue-driven-task-execution.md` がこのファイルの存在と役割を前提にしている。整理の目的は **このファイルを薄く・正確に保つこと** であって、廃止や履歴の作り直しではない。

完了したタスクの詳細経緯は decision log / 各 Issue / 各 PR / working-branch-notes が正本である。`progress.md` には到達点と参照だけを残し、それらを複製しない。

## 参照する正本

作業前に必要な範囲で次を読む。

- `AGENTS.md` — agent 向け入口。
- `doc/guidelines/development-loop.md` — 索引に載せる単位と、進捗整理が独立 PR となる前提。
- `doc/guidelines/issue-driven-task-execution.md` — 索引を依存確認・状態更新に使う側の手順。
- `doc/guidelines/git-operation-guidelines.md` — commit 署名、保護ブランチ。
- `doc/guidelines/pull-request-guidelines.md` — PR title / description の書式。
- `doc/guidelines/working-branch-notes-handling.md` — 作業 note の作成と採番。
- `doc/guidelines/working-branch-notes-security.md` — 作業 note の情報統制。
- `doc/guidelines/decision-log-guidelines.md` — 検討経緯の置き場との境界。
- `doc/guidelines/agent-configuration-management.md` — `progress.md` の役割そのものを変える場合の手順（観点 6）。
- `progress.md` — 整理対象。

## いつ実行するか

次のいずれかに当てはまり、ボードが現状を素早く把握しづらくなっているとき:

- 完了済みのタスク表・項目が溜まり、「いま何が進行中か」が一目で分からない。
- ある作業フェーズの全項目が done になり、表として残す意味が薄れた。
- 進行中の横断タスクが始まった / 終わったのに索引へ反映されていない。
- 「後続で追加する guideline」など、条件付きで残していた項目の条件が満たされた。

定期点検として、まとまった Issue 群を消化し終えた区切りで実行するとよい。

## 整理の観点

各観点を順に確認し、必要な箇所だけ最小限に編集する。判断に迷う箇所は止めてユーザーに確認する。

### 1. 現況

- 冒頭付近に、最新の到達点といま追跡している横断プランの有無を 1〜数行で保つ。
- 進行中の横断プランが無い場合は、その旨と「新しい横断タスクが始まったら表を追加する」という運用だけを残す。
- 事実に反する「進行中なし」を断定しない。不明なら断定せず、確認した範囲を書く。

### 2. 進行中タスクの索引

- いま進めている / これから着手する横断タスクは、状態・依存・参照（Issue / PR）が分かる最小の表で索引する。`doc/guidelines/issue-driven-task-execution.md` がこの索引を依存確認に使う。
- 単発 Issue まで無理に索引へ載せない。索引は「横断的に見渡したい単位」に絞る（`doc/guidelines/development-loop.md`）。

### 3. 完了タスクの圧縮

- 全項目が done になったタスク表・フェーズは、行ごとの詳細を残さず、フェーズ単位の要約数行へ圧縮する。
- 圧縮時は **追跡可能性を失わない**。decision log 番号、Issue 範囲、PR 範囲への参照を要約に残し、詳細はそれらを正本とする。
- まだ進行中・未着手の項目が混在する表は圧縮しない。done と未 done が混ざる場合は、進行中だけを表に残し、done をフェーズ要約へ送る。

### 4. 条件付き項目の点検

- 「後続で追加する guideline」のような、条件が満たされたら解消する項目を点検する。
- 条件が満たされている（対応する資材が実際に置かれた）場合は、該当項目を削除し、必要なら完了要約へ送る。
- 条件が未達なら残す。ただし、条件が何かを読み取れる記述になっているかを確認する。

### 5. リリース台帳

- 本リポジトリはまだリリースを行っていない。`progress.md` の運用メモに「リリースを開始したら別途『リリース履歴』の節を設ける」方針を書いてある。
- リリースを開始したら、公開済みバージョンを台帳として保つ節を追加する。各行はバージョン・状態・1 行メモ（スコープやリリース PR への参照）に留める。
- リリース手順を担う skill を後から追加する場合は、その skill がこの節へ行を足す前提になるため、見出しと列構成を勝手に壊さない。

### 6. 参照整合性の維持

`progress.md` は次から参照されている。整理でこれらの前提を壊さない。

- `AGENTS.md` / `doc/README.md` / `doc/design/README.md` — 「作業状況の一覧は `progress.md`」という配置ルール。
- `doc/guidelines/issue-driven-task-execution.md` — 索引を依存確認・状態更新に使う。
- `doc/guidelines/development-loop.md` — 資材の役割分担。
- `doc/guidelines/decision-log-guidelines.md` — 横断的な状態・依存と Issue / PR 参照の配置先。
- `.github/copilot-instructions.md` — `progress.md` をレビュー観点に含む。

整理は通常これらの前提を保ったまま行える。**もしファイルの役割そのものを変える（索引をやめる、ファイルを廃止する等）場合は、本 skill の範囲を超える方針変更**であり、`doc/guidelines/agent-configuration-management.md` と `doc/guidelines/decision-log-guidelines.md` に従って、参照側ドキュメントの追従修正と decision log の記録を同じ変更に含める。安易に削除しない。

### 7. 境界の維持

- 検討経緯・方針の理由は書かない。それは decision log の役割。`progress.md` には到達点と参照だけを置く。
- ブランチ単位の作業目的・引き継ぎは書かない。それは `working-branch-notes/`。
- Issue 本文の指示内容を複製しない。それは Issue の役割。
- `working-branch-notes/**` と既存 decision log は履歴記録なので、本 skill では遡って書き換えない。

## commit と PR

整理した結果を変更としてリポジトリへ入れるまでを、この skill の範囲に含める。

1. 編集を始める前に、専用ブランチと working branch note を作る。`doc/guidelines/issue-driven-task-execution.md` はブランチ作成 → note 作成 → 作業の順であり、note は作業中の判断を残す前提の資材である。
2. 「整理の観点」に従って `progress.md` を編集する。
3. `progress.md` に変更が生じなかった場合は、commit と PR を作らず、その旨を伝えて終了する。点検の結果、整理すべき箇所が 1 件も無い結末があり得るためである。
4. 変更を PR として出す。
   - `main` は保護されている。**専用ブランチを切り、独立した PR** として出す。他の作業 PR へ同梱しない。
   - **進捗整理そのものには起点 Issue を作らない。PR に `Closes` を付けない。** 「進捗整理を行うための Issue」は、索引登録と同様に指示書として意味を持たないためである（`doc/guidelines/development-loop.md`）。
   - `doc/guidelines/working-branch-notes-handling.md` に従い working branch note を作る。PR 採番後は `number-working-branch-note` skill で採番する。
   - **PR 本文に、圧縮した完了フェーズと、要約に残した decision log / Issue / PR 参照を書く。** 索引の行は最小情報に絞る方針のため、何をどこへ送ったかは PR 側で辿れるようにする。判断に迷ってユーザーへ確認した点があれば、その結論も書く。
   - commit / push は `doc/guidelines/git-operation-guidelines.md`、PR title / description は `doc/guidelines/pull-request-guidelines.md` に従う。
   - **PR の merge は行わない。** レビューと merge 判断はユーザーが行う。

## やらないこと

- `progress.md` の廃止やファイル削除（役割変更を伴うため本 skill の範囲外。観点 6 を参照）。
- decision log・working-branch-notes の内容を `progress.md` へ転記する / それらを書き換える。
- 進行中・未着手を含む表の繰り上げ圧縮。
- 条件が未達の「後続で追加する」項目の削除。
- 進捗整理による `progress.md` の変更を他の作業 PR へ同梱すること。
- PR の merge。

## 終了時の確認

- ボード冒頭を読むだけで「いま進行中の横断タスクの有無」が分かる状態になっている。
- 圧縮した完了フェーズに decision log / Issue / PR への参照が残っている。
- 参照側ドキュメント（観点 6）の前提を壊していない。
- 変更は status board として最小限で、検討経緯や引き継ぎメモを持ち込んでいない。
- 変更を専用ブランチの独立 PR として出し、`Closes` を付けていない。変更が無かった場合は PR を作らず、その旨を報告している。
