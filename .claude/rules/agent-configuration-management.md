---
paths:
  - .agents/**
  - .claude/**
  - .cursor/**
  - .github/copilot-instructions.md
  - doc/guidelines/**/*.md
  - AGENTS.md
  - CLAUDE.md
---

# Agent 設定管理

- 詳細は `doc/guidelines/agent-configuration-management.md`
- skill / rule を作成・削除・rename する前に共通正本を読む。
- rule の basename は共通正本・Cursor 入口・Claude Code 入口の 3 箇所で揃える。
- 新しい rule を作ったら `AGENTS.md` にリンクを追加する（Codex は AGENTS.md 経由でしか到達できない）。
- skill は `.agents/skills/` を正本とし、`.claude/skills/` には symlink だけを置く。
