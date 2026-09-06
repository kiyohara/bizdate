# 作業ブランチメモ

- ブランチ: `add-development-loop-guidelines`
- PR: #2
- 最終更新: 2026-09-06

## 目的

開発環境整備フェーズ 2 として、開発ループの正本を置く。フェーズ 1 で作った agent 入口の上に、実際の作業手順を載せる。

対象は次の 5 つ。

1. PR #1 のレビュー指摘（Codex）の反映
2. 開発ループ入口（`development-loop.md`）
3. Issue 駆動タスク実行ルール（`issue-driven-task-execution.md`）
4. Git 操作ルールと GitHub CLI 実行ルール
5. `progress.md` の再構成（リリース台帳 + 進行中タスク索引）

## 現在の状況

レビュー指摘の反映、4 つの正本、tool 固有入口 8 本、索引の更新、`progress.md` の更新を完了した。PR 作成待ち。

## 決定事項

### PR #1 のレビュー指摘の反映

Codex のレビューで `[fyi]` として挙がった 3 点を、一次資料で確認したうえで本ブランチの冒頭で修正する。いずれも動作上の不具合ではなく記述の正確性の問題だが、`agent-configuration-management.md` は agent 設定の正本であり、誤った仕様記述を残すと後続の判断を誤らせる。

| 指摘 | 一次資料 | 対応 |
|---|---|---|
| Cursor は `.mdc` 必須、`.md` は無視される | Cursor Rules ドキュメント | `*.{md,mdc}` を `*.mdc` に修正（3 ファイル） |
| Copilot code review は GitHub.com で AGENTS.md に対応 | GitHub custom instructions 対応表 | 「AGENTS.md を辿らない」という断定を、環境を区別した記述へ修正 |
| 「4,121 文字」はバイト数 | ローカル実測 | 実測は 2,087 文字。note の記述を訂正 |

あわせて、`agent-configuration-management.md` の「各 instruction file は先頭〜約 4,000 文字のみ反映」という記述を見直す。GitHub の公式対応表に文字数・サイズ上限の記載は無く、この数値の一次的な裏付けが取れなかった。上限が無いと断定はできないため、「上限は公表されていない。要点を前方に置く」という運用可能な表現へ改める。

Copilot 用ファイルに要点を直接書くという構成自体は維持する。AGENTS.md を読むことと、そこからリンクされた `doc/guidelines/` の正本まで辿ることは別だからである。

### 開発コマンド実行ルールの扱い

`development-command-guidelines.md` はフェーズ 2 では作らない。`compose.yaml` と Cargo プロジェクトが無い段階でルールだけ書かないため（`progress.md` の「後続で追加する guideline」）。よって `issue-driven-task-execution.md` からも同 guideline を参照しない。

## 次にやること

- レビュー指摘の 5 行を修正する。
- 4 つの正本と tool 固有入口を作る。
- `progress.md` を再構成する。
- PR を作成し、note を採番する。

## 検証

- **rule basename の整合**: 10 本すべてについて、共通正本 / `.cursor/rules/` / `.claude/rules/` / `AGENTS.md` のリンク / `doc/guidelines/README.md` の一覧、の 5 箇所が揃うことをスクリプトで確認済み。
- **参照の解決**: 全 `*.md` / `*.mdc` を走査し、未解決参照 0 件。decision log 0012 が名指ししていた `development-loop.md` と `issue-driven-task-execution.md` の前方参照 2 件も本 PR で解消した。
- **秘密情報スキャン**: 検出なし。
- **文字数の計測方法**: 前フェーズの誤りを踏まえ、以降は `LC_ALL=en_US.UTF-8 wc -m` または Python の `len()` で測る。`LANG` 未設定の環境では `wc -m` がバイト数を返す。
- 実行可能なコードを含まないため、テストは実行していない。

## リスク・ブロッカー

- `issue-driven-task-execution.md` は現時点で GitHub 操作を `github-cli-guidelines.md` に委ねている。フェーズ 3 で GitHub MCP を導入したら、MCP 優先へ記述を更新する必要がある。

## レビュー対応（PR #2）

Codex から `[must]` 1 件、`[fyi]` 1 件。いずれも一次資料で確認のうえ反映した。

### [must] 署名の切り分け手順（`git-operation-guidelines.md`）

指摘は正しい。本リポジトリの有効設定は `gpg.ssh.program = /Applications/1Password.app/Contents/MacOS/op-ssh-sign` であり、署名は 1Password app が直接処理する。`ssh-keygen -Y sign` は使われないため、`SSH_AUTH_SOCK` は署名の可否と無関係である。

**この誤りの原因は、実証せずに書いたことにある。** 作業時に `ssh-add -l` が `The agent has no identities.` を返したのを見て「署名は失敗するはず」と推論し、commit を試さないまま `SSH_AUTH_SOCK` を上書きして実行した。上書き後の commit が成功したため、上書きが必要だったと誤って結論づけた。実際には上書き無しでも成功していた可能性が高い。

対応として、切り分けを `git config --get gpg.ssh.program` の確認から始める形に変え、1Password signer を使う場合と標準 `ssh-keygen` を使う場合に分けた。前者では `ssh-add -l` を根拠にしないことを明記した。また `no identities` は「その agent に利用可能な鍵がない」ことしか示さず、接続先違いの確定条件ではない点も反映した。

本 PR の commit は `SSH_AUTH_SOCK` を上書きせずに作成し、記述が正しいことを実地で確認する。

### [fyi] instruction file の文字数上限

指摘は正しい。GitHub は 2026-06-12 の changelog で「That limit has now been removed」と明記しており、旧 4,000 文字制限は撤廃されている。`agent-configuration-management.md` の「上限は公表されていない」を「旧制限は撤廃された。分量ではなくシグナルの濃さで絞る」へ改めた。PR #1 の note の該当記述もあわせて訂正した。

### 環境変化への追随

作業中に `.op/` が作成され、`op plugin run -- gh` が利用可能になった。`github-cli-guidelines.md` に op plugin 連携のセクションを追加し、`.op/` と `op` が使える場合は `op plugin run -- gh ...` を使う運用を正本化した。この経路ではリポジトリにスコープを限定した fine-grained PAT が使われる。

## セッションログ

- 2026-09-06: PR #1 merge 後に main から分岐。本 note を作成した。
- 2026-09-06: Codex レビュー指摘 3 点を一次資料で確認のうえ反映（Cursor `.mdc` 必須 / Copilot の AGENTS.md 対応 / 文字数の計測誤り）。あわせて根拠の取れなかった「約 4,000 文字上限」の記述を、上限非公表を前提とした運用表現へ改めた。
- 2026-09-06: PR #2 を作成し、note を採番した。
- 2026-09-06: Codex レビュー（[must] 1 件 / [fyi] 1 件）へ対応。署名の切り分け手順を実行経路の確認から始める形へ修正し、文字数上限の記述を撤廃済みの事実へ改めた。あわせて op plugin 連携を GitHub CLI ルールへ追加した。
- 2026-09-06: 開発ループの正本 4 本と入口 8 本を追加。`AGENTS.md` / `doc/guidelines/README.md` / `progress.md` を更新した。
