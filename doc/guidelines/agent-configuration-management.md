# Agent 設定管理ルール

この文書は、bizdate リポジトリ内で Claude Code / Codex / Cursor から共通利用する agent skill と AI 向け rule を作成・削除・rename するときの共通正本である。AI と人間で別ドキュメントを持たず、全員がこの文書を読む。

対象 agent と配置の方針は `doc/design/decision-log/0011-ai-agent-lineup.md` に記録している。本ガイドライン自身もこのルール体系の管理対象である（自己参照）。

## ルールをシンプルに保つ

このガイドライン自体の運用方針として、**ルールが肥大化するほど遵守率は下がる**ことを大前提とする。

- 追記する前に、その項目が無くても運用で困らないかを自問する。困らないなら書かない。
- 既存の表・checklist・公式ドキュメントから自明に導けることは書かない。
- 先回りで仮想シナリオに備えない。実際に必要になった時点で追加する。
- 既存項目を改訂するときは、削れる行が無いかを同時に確認する。

新しい AI tool の採用、配置規約の変更、loading 機構の仕様変更などでルール体系自体に手を入れる場合は、上記方針に従いつつ「各 tool の loading 機構」の表と各 checklist を最新仕様に揃える（同じ PR で完結させる）。

## 各 tool の loading 機構（2026-09 時点）

| Tool | Rule の自動ロード | Skill の自動ロード | AGENTS.md 扱い |
| --- | --- | --- | --- |
| Cursor | `.cursor/rules/*.mdc` を frontmatter に従ってロード（`.md` は frontmatter を持てないため無視される） | `.agents/skills/` を起動時に自動 discover | 公式に rules の代替として認識 |
| Codex | AGENTS.md（global / project 階層, 32 KiB 上限） | `.agents/skills/` を cwd -> repo root で走査し、name + description のみ system prompt に preload。SKILL.md 全文は使用時にロード | プロジェクトルートの AGENTS.md を読む |
| Claude Code | `.claude/rules/*.md`（任意の `paths:` frontmatter で path scoping）+ CLAUDE.md（`@path` import で AGENTS.md を取り込み） | `.claude/skills/<name>/SKILL.md` を frontmatter に従ってロード | `CLAUDE.md` から `@AGENTS.md` で取り込む構成が公式推奨 |
| GitHub Copilot code review | `.github/copilot-instructions.md`（repo 全体）と `.github/instructions/*.instructions.md`（`applyTo:` で path scoping）をレビュー時に読む | なし | GitHub.com のレビューでは AGENTS.md も読む。ただしリンク先の正本まで辿る保証はないため、効かせたい内容は instruction file 内に直接書く |

帰結:

- **`.agents/skills/` を直接読める tool には固有ディレクトリへの symlink を作らない**。Cursor と Codex は `.agents/skills/` を直読するため symlink 不要。Claude Code は `.claude/skills/` しか読まないため symlink を作る。
- Codex は **AGENTS.md に書かれていない情報には自力で到達できない**（skill は除く: `.agents/skills/` は自動走査される）。新しい rule や正本を作ったら、Codex がそこへ辿り着けるよう **AGENTS.md からのリンクは必須** とする。
- Claude Code は `.claude/rules/` と CLAUDE.md（`@AGENTS.md` 経由）の両方を読む。役割分担は後述の「正本と入口の整理」を参照。
- GitHub Copilot code review は、GitHub.com 上のレビューでは AGENTS.md を読む。ただし **そこからリンクされた `doc/guidelines/` の正本まで辿る保証はない**。よって repo 全体のレビュー方針は `.github/copilot-instructions.md` に、path 別の詳細レビュー観点は `.github/instructions/*.instructions.md`（`applyTo:`）に **直接** 書く。正本へのリンクは人間 / 他 tool 向けポインタとして併記し、Copilot がそれを辿ることは当てにしない。instruction file にはかつて先頭 4,000 文字までという制限があったが、2026-06-12 に撤廃された。長さの制約は無くなったので、分量ではなくシグナルの濃さで絞る。

## 使い分け

| 対象 | 共通正本 | Cursor 入口 | Claude Code 入口 | Codex 入口 |
| --- | --- | --- | --- | --- |
| Agent skill | `.agents/skills/<skill-name>/` | `.agents/skills/` を直接読む（symlink 不要） | `.claude/skills/<skill-name>` symlink | `.agents/skills/` を直接読む |
| AI 向け rule | `doc/guidelines/<rule-name>.md` | `.cursor/rules/<rule-name>.mdc` 入口 | `.claude/rules/<rule-name>.md` 入口 | `AGENTS.md` から共通正本へ誘導 |
| Copilot review 指示 | `.github/copilot-instructions.md` | 対象外 | 対象外 | 対象外 |

迷ったら、まずこの表で対象を決めてから該当セクションだけ読む。

- skill の場合: `<skill-name>` は **正本ディレクトリ名と `.claude/skills/` symlink 名で揃える**。
- rule の場合: `<rule-name>` は **3 箇所（共通正本・Cursor 入口・Claude Code 入口）で basename を揃える**。

## 共通原則

- AI と人間で別々の正本を作らない。
- 恒久的なプロジェクト方針は `doc/guidelines/`、`doc/design/`、または `.agents/skills/` の正本に置く。
- `.cursor/**`、`.claude/**`、`.github/copilot-instructions.md` は tool 固有の入口として扱い、長い正本を複製しない。
- Claude Code / Codex / Cursor のどれか一方だけが読める配置にしない。
- 削除・rename 時は、正本だけでなく tool 固有入口も同じ変更で揃える。

## 正本と入口の整理

- **正本**には、判断基準、配置ルール、checklist、禁止事項など、保守すべき恒久ルールを置く。
- **入口**には、正本への参照と「いつ正本を読むか」だけを置く。入口に詳細 checklist を複製しない。
- Cursor / Claude Code の rule は、正本を読むための薄い shim とする。
- Codex / AI agent の入口は `AGENTS.md` とし、ここにも詳細 checklist を複製しない。
- 正本を変更した場合、入口の参照先や入口としての役割が壊れていないかだけ確認する。正本の本文変更に合わせて入口へ同じ内容を追記しない。
- **例外: `.github/copilot-instructions.md` と `.github/instructions/*.instructions.md`** は薄い shim ではなく、Copilot code review がリンクを辿れないため要点を直接書く。複製元の正本を実質的に変更したら、対応する Copilot 用ファイルの同期要否も確認する。同期チェックは人手の判断に委ね、CI や同期スクリプトのような重い仕組みは入れない。
- 入口に恒久ルールを書きたくなった場合は、まず正本に書き、入口には正本への誘導だけを置く。

## Agent skill 管理

### 配置

```text
.agents/skills/<skill-name>/
├── SKILL.md
└── references/
    └── ...

.claude/skills/<skill-name> -> ../../.agents/skills/<skill-name>
```

- `SKILL.md` は必須。frontmatter は Claude Code / Anthropic skill 仕様に合わせる（`description` 必須相当、必要に応じて `argument-hint` `allowed-tools` など）。
- AI が必要時に読む詳細資料は `references/` に置く。
- script / asset が必要な場合は、skill 配下の `scripts/` / `assets/` に置く。skill 内では **相対パス** で参照する。
- **Cursor / Codex 用 symlink は作らない**。両者とも `.agents/skills/` を直接走査するため不要。

### 作成 checklist

1. `.agents/skills/<skill-name>/` に正本を作る。
2. `.agents/skills` と `.claude/skills` が存在しない場合は作成する。
3. `.claude/skills/<skill-name>` を正本への symlink として作る（target: `../../.agents/skills/<skill-name>`、末尾 `/` を付けない）。
4. symlink 経由で `SKILL.md` が読めることを確認する。
5. `SKILL.md` frontmatter の `name` / `description` を確認する。

```sh
mkdir -p .agents/skills .claude/skills
ln -s ../../.agents/skills/<skill-name> .claude/skills/<skill-name>
test -f .claude/skills/<skill-name>/SKILL.md
```

### 削除 checklist

skill の正本を削除する場合は、同じ変更で `.claude/skills/<skill-name>` symlink も削除する。正本だけを削除して broken symlink を残してはならない。

削除後に確認する:

```sh
find -L .claude/skills -maxdepth 1 -type l -print
git ls-files | xargs rg -l "<skill-name>" 2>/dev/null
```

### rename checklist

1. 新しい `.agents/skills/<new-skill-name>/` と `.claude/skills/<new-skill-name>` symlink を作る。
2. 古い正本と symlink を削除する。
3. 旧名への残存参照を `git ls-files | xargs rg "<old-name>"` で確認する。

### 禁止事項

- `.claude/skills/` に skill 本体を直接配置する（symlink 限定）。
- `.cursor/skills/` や `.codex/skills/` を作成する（`.agents/skills/` 直読で十分なため、冗長な symlink を増やさない）。
- 正本削除後に symlink を残す。
- 正本ディレクトリ名と symlink 名をずらす。
- skill 内 script / asset の参照に絶対パスをそのまま使う（symlink 経由で破綻するため）。

## AI 向け rule 管理

### 配置

```text
doc/guidelines/<rule-name>.md
.cursor/rules/<rule-name>.mdc
.claude/rules/<rule-name>.md
AGENTS.md
CLAUDE.md
.github/copilot-instructions.md          # Copilot code review の repo 全体方針
.github/instructions/*.instructions.md   # Copilot code review の path 別詳細観点（`applyTo:`、任意）
```

- `doc/guidelines/<rule-name>.md` は AI と人間が読む共通正本。
- `.cursor/rules/<rule-name>.mdc` は Cursor 用入口。frontmatter で発火条件を制御し、共通正本を参照する。
- `.claude/rules/<rule-name>.md` は Claude Code 用入口。任意で `paths:` frontmatter を付けてスコープ限定できる。
- `AGENTS.md` は Codex / AI agent 共通 index。**Codex は AGENTS.md からしか rule 正本に到達できない** ため、すべての rule をここにリストすることを必須とする。
- `CLAUDE.md` は Claude 用 shim。原則 `@AGENTS.md` の取り込みに留める。

### 入口 rule frontmatter の書き方

**Cursor** (`.cursor/rules/<rule-name>.mdc`):

- 特定の作業に紐づく rule は `description` と `globs` を書き、`alwaysApply` は `false` にする。
- 全セッションで強制したい絶対的な方針のみ `alwaysApply: true` を選ぶ。

```markdown
---
description: <Cursor agent が「この rule を読み込むかどうか」を判断する短い説明>
globs:
  - <対象 path>
alwaysApply: false
---

# <rule title>

- 詳細は `doc/guidelines/<rule-name>.md`
- <この rule が関係する作業> の前に、必ず共通正本を読む
```

**Claude Code** (`.claude/rules/<rule-name>.md`):

- frontmatter 無し -> 全セッションでロード。
- `paths:` frontmatter 付き -> 対象ファイルを開いたときのみロード。

### 作成 checklist

1. `doc/guidelines/<rule-name>.md` に共通正本を作る。
2. `.cursor/rules/<rule-name>.mdc` を作り、共通正本への参照と「いつ読むか」だけを書く。
3. `.claude/rules/<rule-name>.md` を作り、共通正本への参照と「いつ読むか」だけを書く。
4. `AGENTS.md` の「共通正本」セクションに共通正本へのリンクを追加する（**全 rule 必須**）。
5. `doc/guidelines/README.md` の一覧を更新する。
6. 3 ファイルの basename（`<rule-name>`）が揃っていることを確認する。

### 削除 checklist

rule の共通正本を削除する場合は、同じ変更で Cursor / Claude Code 用入口 rule と `AGENTS.md` の参照も削除する。

削除後に確認する:

```sh
git ls-files | xargs rg -l "<rule-name>" 2>/dev/null
```

### rename checklist

1. 新しい共通正本と、新しい名前の Cursor / Claude Code 入口を作る。
2. `AGENTS.md` の参照を更新する。
3. 古い共通正本と古い入口、古い `AGENTS.md` 参照を削除する。
4. 旧名への残存参照を `git ls-files | xargs rg "<old-rule-name>"` で確認する。
