# guidelines

このディレクトリには、AI agent と人間が共通で従う作業ルール、運用ガイドラインを置く。

## 置くもの

- 記録の仕方、文体、開発作業の進め方などの恒久ルール
- 1 テーマ 1 ファイル

## 置かないもの

- プロダクトの仕様: `doc/design/` に置く
- 作業状況: `progress.md` に置く

## 追加するとき

- 新しい guideline を足したら、この README と `AGENTS.md` の共通正本一覧を更新する。
- AI agent から読ませる場合は、正本を複製せず `.cursor/rules/` と `.claude/rules/` に薄い shim を置く。Codex は `AGENTS.md` のリンクから正本へ辿る。
