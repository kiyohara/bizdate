# リリース手順

- 詳細は `doc/guidelines/release-guidelines.md`
- version の公開（リリース準備 PR、公開前確認、承認の提示、公開の監視と復旧、公開後確認、リリース台帳の更新）に関わる前に共通正本を読む。agent は `run-release` skill で進める。
- tag の push、PR の merge、secrets の操作は agent が行わない。公開の承認はユーザーの文言で受ける。
- 既存の tag を付け替えない。失敗した job を確認なしに再実行しない。やり直すときは version を上げる。
