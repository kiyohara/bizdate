# 作業ブランチメモ

- ブランチ: `claude/issue-53-one-password-integration`
- PR: #58
- 最終更新: 2026-09-13

## 目的

Issue #53 に従い、1Password 連携操作が承認待ちで失敗したときの扱いを共通正本に定め、既存の skill / guideline から 1Password 連携の有無で分岐する記述を外す。

本ブランチは PR #56 と同じ Issue に対する比較用の対応であり、PR #56 の差分は持ち込まず `main` から作業している。PR #56 の議論と、その追試レポート（1Password 連携のあるローカル環境での実測）は参考情報として使う。

背景にあるのは開発環境の多様化である。従来は全メンバーが macOS ローカルで 1Password 連携を持っていたが、PR #48 で cloud session（Claude Code on the web）が加わり、1Password が無い環境も扱う必要が出た。動作しないという問題ではなく、cloud VM 上の agent を人間の手番なしに進めるための最適化である。

## 現在の状況

新正本 `doc/guidelines/one-password-integration-guidelines.md` と入口（`.claude/rules/`、`.cursor/rules/`、`AGENTS.md`、`doc/guidelines/README.md`）を作り、既存 guideline / skill の分岐を新正本参照へ縮めた。decision log 0021 を追加した。

## 決定事項

- ルールは「1Password 連携ルール」1 本にまとめ、git / gh / MCP の各 guideline には処理フローを書かず、新正本への参照だけを置く。導線は `AGENTS.md` の 1 行（エラーが出たら承認が間に合わなかった可能性を疑い新正本を読む）とする。
- 方針は事後中断。前段にゲートや preflight を置かず、失敗した時点で中断し、報告と選択肢提示を行う。
- Issue の完了条件チェックリストは参考とし、シンプルさを優先する。満たさない項目は PR description に理由を書く（ユーザー確認済み）。
- `gh` の実行形式（`.op/` と `op` があれば `op plugin run -- gh`）は、承認可否の preflight ではなく実行形式の選択として `doc/guidelines/github-cli-guidelines.md` に残す。skill 側の複製は guideline 参照へ縮める（ユーザー確認済み）。
- MCP server の起動失敗は、agent からは `CONNECT_TIMEOUT` などの汎用表示しか見えず 1Password 起因かを判別できないため、切り分けをせず中断・報告する。`gh` fallback は MCP 未設定、allowlist 外、起動済み MCP の応答失敗に限る（ユーザー確認済み）。
- git 操作ルールの署名経路の切り分け（`op-ssh-sign` / `ssh-keygen` + SSH agent）は新正本へ移し、git 操作ルールには失敗時の参照先だけを残す（ユーザー確認済み）。
- remote の HTTPS 断定は撤回し、scheme と認証経路は環境依存とする。
- decision log の番号は 0021 とする。PR #56 と衝突するが、両 PR は択一であり同時に merge されない。
- review 指摘を受けて再検討した `gh` の実行形式について。Claude Code の Bash は `zsh -c` と shell snapshot で動き、`op plugin init gh` の alias（対話 shell の rc から source）は効かないため、素の `gh` は実バイナリであり gh 自身の keyring 認証で動く。「`op plugin run -- gh` を試し、失敗したら素の `gh` へ fallback する」案は、承認待ちの失敗でも credential が黙って入れ替わるため採らない。`.op/plugins/gh.json` の有無による選択は、メンバーが環境で宣言した設定に従う静的な選択として維持し、正本の原則文をその旨に限定した。

## 次にやること

- [x] PR を作成し、note を採番する。
- [ ] ユーザーのレビューを待つ。自動レビューは行わない（ユーザー指示）。

## 検証

- Rust コードの変更が無いため `cargo` は実行しない。
- `git grep` による残存確認、markdown link の実在確認、`git diff --check`、文末の常体確認を行う（結果は後述）。
- 承認待ちタイムアウトの再現は本セッションでは行わない。PR #56 の追試レポートで実測された文言（`error: 1Password: ...`、`authorization timeout`、`authorization prompt dismissed`、MCP host の `CONNECT_TIMEOUT`、SSH の `agent refused operation`）と、署名失敗時に commit が作られないこと、lock 中は `op plugin run -- gh` も再承認を要求することを、参考情報として新正本に反映した。

## リスク・ブロッカー

- 新正本の文言例は 1Password 8 / macOS の実測であり、版や OS で変わりうる。判定は文言の完全一致ではなく「1Password や承認に関する文言があるか、判断できないか」で行う設計にしている。
- `.cursor/rules/` の入口が意図した場面で発火するかは未検証。`.claude/rules/` 側は frontmatter 無しの全セッションロード。

## セッションログ

- 2026-09-13: Issue #53、PR #56、追試レポート、既存 guideline を読み、構成案をユーザーに確認した。`origin/main` から本ブランチを作成し、新正本と入口、既存整理、decision log を書いた。PR #58 を作成し、note を採番した。
- 2026-09-13: review（Codex 指摘なし、Claude Code `[imo]` 3 件 / `[nits]` 1 件、cycle `claude-code-a7c7ec5-20260913103203`）の 4 件をすべて採用した。正本の原則文の限定、cloud session 節の除外明示、CLI guideline の適用条件、skill の検索後の扱い。
