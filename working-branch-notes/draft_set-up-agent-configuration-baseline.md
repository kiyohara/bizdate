# 作業ブランチメモ

- ブランチ: `set-up-agent-configuration-baseline`
- PR: （採番前）
- 最終更新: 2026-09-06

## 目的

開発環境整備フェーズ 1 として、AI agent の入口と作業記録の基盤を置く。これ以降のフェーズの PR が、ここで定めたルールに従って進められる状態にすることが目標である。

対象は次の 4 つ。

1. agent 設定管理ルール（skill / rule の作成・削除・rename・配置）
2. working branch notes の取り扱いルールと情報統制ルール
3. Pull Request 作成ガイドライン
4. GitHub Copilot code review 用の指示

## 現在の状況

4 つの正本、tool 固有入口、`AGENTS.md` / `doc/guidelines/README.md` の更新を完了した。PR 作成待ち。

## 決定事項

- 開発環境整備は Issue を作らず、フェーズ単位のブランチと PR で記録する（`doc/design/decision-log/0012-development-loop.md`）。全 4 フェーズ、1 フェーズ = 1 ブランチ = 1 PR とする。
- `.github/copilot-instructions.md` をこのフェーズに含める。フェーズ 2 以降の PR が Copilot code review を受けられるようにするためである。
- agent 設定管理ルールから **MCP server 共通資材の管理セクションを外す**。MCP 連携はフェーズ 3 で導入するため、実体が無い段階でルールだけ先に書かない。同ガイドライン自身の「先回りで仮想シナリオに備えない」方針に従う。同じ理由で worktree 用の local config 配置セクションもフェーズ 3 に送る。
- ディレクトリ配置を説明する `README.md` は開発者向けとして常体で書く（`doc/guidelines/document-style-guidelines.md`）。

## 次にやること

- 4 つの正本と tool 固有入口を作る。
- `AGENTS.md` の共通正本一覧を更新する。
- PR を作成し、note を採番する。

## 検証

- rule の basename 整合: 6 本すべてで共通正本 / `.cursor/rules/` / `.claude/rules/` / `AGENTS.md` / `doc/guidelines/README.md` の 5 箇所が揃うことをスクリプトで確認済み。
- 参照の解決: markdown リンクと repo 相対 path 参照を全 md / mdc に対して走査。未解決は decision log 0012 が名指しする `development-loop.md` と `issue-driven-task-execution.md` の 2 件のみで、いずれもフェーズ 2 の成果物を指す意図的な前方参照。
- 秘密情報スキャン: 検出なし。
- `.github/copilot-instructions.md` は 4,121 文字。目安の約 4,000 文字をわずかに超えるが、truncate されても影響が小さい順（prefix 体系 → 情報統制 → 指摘しない事項 → 構成観点 → note 観点）に並べてある。

## リスク・ブロッカー

- decision log 0012 の前方参照 2 件は、フェーズ 2 で `doc/guidelines/development-loop.md` と `doc/guidelines/issue-driven-task-execution.md` を作った時点で解消する。それまで Copilot review が「存在しないファイルへの参照」として指摘する可能性がある。

## セッションログ

- 2026-09-06: ブランチ作成。`working-branch-notes/` の README と template を配置し、本 note を作成した。
- 2026-09-06: 4 つの正本、Cursor / Claude Code 入口 各 4 本、`.github/copilot-instructions.md` を追加。`AGENTS.md` の共通正本一覧と `doc/guidelines/README.md` の一覧を更新した。既存 2 rule（decision-log / document-style）にも `doc/guidelines/README.md` の一覧行を追加した。
