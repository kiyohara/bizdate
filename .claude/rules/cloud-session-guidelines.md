# Cloud Session

- 詳細は `doc/guidelines/cloud-session-guidelines.md`
- cloud session（Claude Code on the web）で作業するとき、または cloud session 向けの設定（`.agents/scripts/cloud-session-setup.sh`、`.claude/settings.json`、`compose.cloud.yaml`、environment の setup script）を変更する前に共通正本を読む。
- 開発コマンドは cloud session でも `docker compose run --rm dev cargo ...`。daemon は SessionStart hook が起動する。起動していなければ sandbox の `cargo` で代替せず、正本に従って報告する。
- `.claude/settings.json` には hook の登録だけを書く。処理本体は `.agents/scripts/` に置く。
