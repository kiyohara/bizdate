# GitHub MCP 利用

- 詳細は `doc/guidelines/github-mcp-guidelines.md`
- GitHub 操作の最初の試行先は `github-op-integrated` MCP tool。`gh auth status` などの preflight を先行しない。
- allowlist 外の操作（merge / release / workflow dispatch / settings）は `gh` で行う。
- `APPROVE` / `REQUEST_CHANGES` / thread resolve / merge は自動実行しない。
- write が失敗したら、read-back してから再実行を判断する。
- cloud session（Claude Code on the web）では組み込みの GitHub tool を第一選択にする。allowlist 外の write（merge / resolve / workflow 実行 / file push）は見えていても実行しない。
- cloud session では、組み込み tool に無い操作だけ `gh api`（REST）で補う。GraphQL を使う subcommand と `gh auth status` は使わない。禁止・要承認の操作は `gh` の経路でも同じ。
