# Pull Request 作成ガイドライン

この文書は、AI agent が bizdate リポジトリで Pull Request を作成または更新するときの共通正本である。

## 基本方針

- PR description は日本語で書く。
- PR title には、作成に使った tool 名やそれを示す prefix を含めない。
- description はレビュアーが変更意図と確認観点を把握できる粒度で書く。
- 変更内容、背景、レビュアーに特に見てほしい点、検証内容、未検証事項を分けて書く。
- 既存ルール、設定、配置、責務分担を整理した場合は、何をどの正本へ移し、どの入口をどう変更したかを具体的に書く。
- テストを実行していない場合は、理由を明記する。
- **PR の merge は AI agent が行わない**。レビューと merge 判断は人間が行う（`doc/design/decision-log/0012-development-loop.md`）。

## Tool 名の扱い

- PR title には `claude`、`codex`、`cursor` などの tool 名や、`[codex]` のような tool 由来の prefix を含めない。title はレビュアーが変更内容を把握するためのものである。
- PR description と PR コメントでは制限しない。`Co-Authored-By` や `🤖 Generated with ...` のような trailer も記載してよい。

## 推奨構成

```md
## 概要

## 主な変更

## 既存整理の詳細

## レビューしてほしい点

## 検証

## 補足
```

変更が小さい場合は項目を減らしてよい。ただし、日本語で具体的に書く方針は維持する。
