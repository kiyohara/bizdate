# doc

このディレクトリには、リポジトリ全体で共有するドキュメントを置く。

## 配置

| path | 置くもの |
|---|---|
| `doc/guidelines/` | AI agent と人間が共通で従う作業ルール、運用ガイドライン |
| `doc/design/` | `bizdate` の仕様設計、利用体験設計、設計判断の記録 |

作業状況の一覧は `progress.md`、ブランチ単位の作業メモは `working-branch-notes/` に置く。

## 方針

AI agent 専用の説明を別ファイルとして分離せず、人間も読める `README.md` と `doc/guidelines/` を共通正本として扱う。

各ディレクトリに新しい文書を追加する前に、そのディレクトリの `README.md` を確認する。`doc/guidelines/` の配置判断も `doc/guidelines/README.md` を正とする。

## まだ置かないもの

- `doc/help/`: 利用者がリポジトリ上で直接読む help / how-to。CLI の形と出力が固まってから追加する（`doc/design/decision-log/0001-document-directory-structure.md`）。
