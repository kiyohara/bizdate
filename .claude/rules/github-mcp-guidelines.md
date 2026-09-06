# GitHub MCP 利用

- 詳細は `doc/guidelines/github-mcp-guidelines.md`
- GitHub 操作の最初の試行先は `github-op-integrated` MCP tool。`gh auth status` などの preflight を先行しない。
- allowlist 外の操作（merge / release / workflow dispatch / settings）は `gh` で行う。
- `APPROVE` / `REQUEST_CHANGES` / thread resolve / merge は自動実行しない。
- write が失敗したら、read-back してから再実行を判断する。
