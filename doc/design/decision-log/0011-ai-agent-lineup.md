<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# AI agent 体制と設定の配置

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `AGENTS.md`, `doc/README.md`, `doc/guidelines/README.md`

## 背景

`bizdate` の開発は AI agent との共同作業を前提とする。単一の agent に固定すると、その tool の制約や調子に開発速度が縛られる。一方で複数 agent を併用すると、rule / skill / MCP 設定を tool ごとに書き分けることになり、正本が分裂しやすい。

どの agent を対象とし、共通正本と tool 固有入口をどう分けるかを決める必要がある。

## 候補

- A: 単一 agent に限定し、その tool の作法だけに合わせる
- B: 複数 agent を対象とし、共通正本を 1 か所に置き、各 tool 固有ファイルは薄い入口に留める
- C: 複数 agent を対象とし、tool ごとに独立した設定一式を持つ

## 検討内容

各 tool の loading 機構が異なる点が判断材料になる。

- Cursor: `.cursor/rules/*.{md,mdc}` を frontmatter に従ってロードする。skill は `.agents/skills/` を直接 discover する。
- Codex: `AGENTS.md` を読む。リンク先を自力では辿らないため、正本への到達経路を `AGENTS.md` に明示する必要がある。skill は `.agents/skills/` を走査する。
- Claude Code: `.claude/rules/*.md` と `CLAUDE.md` を読む。skill は `.claude/skills/<name>/SKILL.md` しか見ない。
- GitHub Copilot code review: `.github/copilot-instructions.md` をレビュー時に読む。リンクを辿らず、読み込む文字数にも上限がある。

C 案は、同じルールを 3 か所に複製することになり、更新漏れで tool 間の挙動がずれる。A 案は tool 固有の制約が開発全体の制約になる。

B 案では、共通正本を `doc/guidelines/`（rule）と `.agents/skills/`（skill）に置き、各 tool 固有ファイルは「いつ正本を読むか」だけを書く薄い入口にできる。skill は Cursor と Codex が `.agents/skills/` を直読するため、Claude Code 用の symlink だけ用意すればよい。

Copilot code review だけは例外になる。リンクを辿らないため、効かせたい要点を指示ファイル内に直接書く必要がある。これは複製だが、対象を高シグナルな要点に絞れば維持できる。

## 決定

- 対象 agent は **Claude Code / Codex / Cursor** の 3 体制とする。加えて、GitHub Copilot code review をレビュー時の補助として使う。
- 共通正本の置き場を次のとおりとする。
  - AI 向け rule: `doc/guidelines/<rule-name>.md`
  - agent skill: `.agents/skills/<skill-name>/`
  - MCP server 共通資材: `.agents/mcp/<server-name>/`
- tool 固有入口は薄い shim とし、恒久ルールを複製しない。
  - `AGENTS.md`: AI agent 共通 index。Codex はここからしか正本に到達できないため、全 rule のリンクを必須とする。
  - `CLAUDE.md`: `@AGENTS.md` の取り込み shim。
  - `.claude/rules/<rule-name>.md` / `.cursor/rules/<rule-name>.mdc`: 正本への参照と発火条件のみ。
  - `.claude/skills/<skill-name>`: `.agents/skills/<skill-name>` への symlink。Cursor / Codex 用の symlink は作らない。
- `.github/copilot-instructions.md` は例外とし、要点を直接書く。
- 上記の作成・削除・rename 手順は `doc/guidelines/agent-configuration-management.md` を正本とする。
- 恒久的なプロジェクト方針を agent 固有 shim にだけ書いてはならない。AI と人間で別ドキュメントを持たない。

## 理由

3 tool の loading 機構が異なる以上、共通正本 + 薄い入口の形にしないと、どこかの tool だけが読める配置が生まれる。skill は 2 tool が直読するため symlink を最小限に抑えられ、rule は入口を薄く保てば複製コストが上限を持つ。

## 影響

- rule を 1 本足すたびに、共通正本、Cursor 入口、Claude Code 入口、`AGENTS.md` リンクの 4 か所を揃える。
- skill を 1 本足すたびに、`.agents/skills/` の正本と `.claude/skills/` の symlink を揃える。
- MCP server 設定は tool ごとに設定ファイルが分かれる（`.mcp.json` / `.cursor/mcp.json` / `.codex/config.toml`）。いずれも secret を含まない起動定義だけを置く。

## 後から見直す条件

- 対象 agent を増減する場合。
- いずれかの tool の loading 機構が変わり、配置規約が成り立たなくなった場合。
- 入口の複製コストが、共通正本を保つ利点を上回った場合。
