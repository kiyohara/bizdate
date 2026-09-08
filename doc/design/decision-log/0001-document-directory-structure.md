# 0001 document directory structure

- 状態: decided
- 作成日: 2026-08-18
- 最終更新日: 2026-09-08
- 関連: `doc/README.md`, `doc/guidelines/README.md`, `doc/design/README.md`, `doc/design/concept.md`, `progress.md`, `AGENTS.md`

## 背景

`bizdate` はコンセプトだけが決まっており、詳細仕様は対話を通じて決めていく段階である。検討内容と検討結果をリポジトリに残す必要がある。

参考として、過去に作った [kiyohara/slapex](https://github.com/kiyohara/slapex) のドキュメント構成がある。slapex は v1 以降まで育てた完成形であり、仕様、経緯、進捗、作業メモの分離に加えて、利用者向け help と出力サンプルまで含む。help とサンプルは CLI の形と出力が固まってから意味を持つため、それを最初から複製すると、まだ決まっていない内容まで先に固定してしまう。

## 候補

- slapex の完成形をほぼそのまま複製する。
- slapex の役割分担を借り、記録に必要なディレクトリから始める。
- 単一のメモファイルに検討を蓄積する。

## 検討内容

slapex では、仕様（`doc/design/` 直下）、経緯（`decision-log/`）、進捗（`progress.md`）、作業メモ（`working-branch-notes/`）を混ぜないことで、後続の AI agent が正本を取り違えにくくなった。この 4 分割は記録の骨格であり、bizdate でも最初から採用する価値がある。

一方、`doc/help/` は利用者がそのまま手順として読む文書であり、CLI の形が固まる前に置くと空のまま増える。出力サンプルも同様に、成果物が存在してから作る。この 2 つは骨格に含めない。

単一メモに全部書く案は着手が軽いが、仕様と経緯と進捗が混ざり、slapex で一度やり直した問題を繰り返す。

AI agent 向けの入口は `AGENTS.md` に集約する。対象 agent と各 tool 固有入口の構成は本ログでは扱わず、[0011](0011-ai-agent-lineup.md) で定める。`working-branch-notes/` の取り扱いルールと開発ループへの組み込みは [0012](0012-development-loop.md) で定める。

## 決定

記録用ドキュメントは次の構成とする。

```text
README.md
AGENTS.md
progress.md
doc/
├── README.md
├── guidelines/
│   ├── decision-log-guidelines.md
│   └── document-style-guidelines.md
└── design/
    ├── README.md
    ├── concept.md
    └── decision-log/
        ├── index.md
        ├── _template.md
        └── <連番>-<短い英語slug>.md
working-branch-notes/
```

- 仕様の正本は `doc/design/` 直下の spec 文書とする。初期の spec は `concept.md` のみとする。
- 検討経緯は `doc/design/decision-log/` に置く。decision log は仕様の正本ではない。
- 横断的な作業状況は root の `progress.md` に置く。
- ブランチ単位の作業メモは `working-branch-notes/` に置く。取り扱いルールは [0012](0012-development-loop.md) に従う。
- AI agent 向け入口は `AGENTS.md` とし、本文は正本へのポインタに留める。tool 固有入口の構成は [0011](0011-ai-agent-lineup.md) に従う。
- `doc/help/` と出力サンプルは、CLI の形と出力が固まってから追加する。

## 理由

「正本と履歴と進捗と作業メモを分ける」骨格は記録を始めた時点から効く。一方 help とサンプルは対象が確定してからでないと書けない。前者を先に入れ、後者を条件付きで後回しにすると、設計対話をすぐ記録でき、後からの追加も配置ルールで判断できる。

## 影響

- 設計対話で決まったことは、該当 spec を更新し、経緯を decision log に残し、`index.md` と必要なら `progress.md` を更新する。
- 新しい guideline や spec を足すときは、先に `doc/README.md` と該当ディレクトリの `README.md` を更新する。

## 後から見直す条件

- CLI の形が固まり、利用者向け help が必要になった場合。
- `doc/design/` 直下の spec が増えて見通しにくくなった場合。

## 追記 (2026-08-20)

`doc/guidelines/README.md` を追加した。各ディレクトリの README を配置判断の入口とする運用と揃えるためである。骨格の意図は変えない。

## 追記 (2026-09-08): v1 の利用方法の配置

Issue #15 で CLI の実装後に `doc/help/` の要否を検討した。v1 は `first` / `last` / `fetch-holidays` の 3 subcommand で、判定の option は共通である。祝日データの準備、判定、exit code、シェル・cron のゲート例は root の `README.md` に収まるため、現時点では `doc/help/` を作らない。分割すると短い利用手順を複数ページにまたがって読む必要が生じる。

CLI の形が固まったことを自動的なディレクトリ追加の条件とはせず、README から独立した利用手順や複数ページの説明が必要になった時点で分割を見直す。配布手段の決定後にインストール・運用の説明を加える場合も、README に収まるかを再評価する。出力サンプルの方針は本追記の対象外とする。
