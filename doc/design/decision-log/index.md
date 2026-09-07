# 方針決定ログ index

このディレクトリには、各種方針決定に伴う検討内容を記録していく。

この `index.md` は、AI agent と人間が最初に読む入口である。詳細な検討内容は個別ログファイルに分け、ここには現在有効な主要方針、未決事項、参照先だけを簡潔に記載する。

記録方法の詳細は `../../guidelines/decision-log-guidelines.md` を参照する。

なお、仕様の正本は `doc/design/` 直下の各 spec 文書であり、decision log と本 `index.md` はその確定経緯を辿る参考ログである。decision log を仕様の正本として扱わない（詳細は `../../guidelines/decision-log-guidelines.md` の「正本と参照の関係」）。

## 現在有効な主要方針

| ID | 状態 | 主題 | 現在の結論 | 詳細 |
|---|---|---|---|---|
| 0001 | decided | ドキュメント配置 | 仕様は `doc/design/`、経緯は `decision-log/`、進捗は root の `progress.md`、作業メモは `working-branch-notes/`。`doc/help/` と出力サンプルは CLI の形が固まってから | [0001-document-directory-structure.md](0001-document-directory-structure.md) |
| 0002 | decided | CLI command name | CLI command name は `bizdate` とする | [0002-cli-command-name.md](0002-cli-command-name.md) |
| 0003 | decided | v1 スコープ | 主用途はゲート。公開は first/last と fetch-holidays。埋め込み会社カレンダーは対象外。`--day-off` は可。v1 既定は日本。他国は `--source` 想定 | [0003-v1-scope.md](0003-v1-scope.md) |
| 0004 | decided | 日付とタイムゾーン | 日付のみ・`YYYY-MM-DD`。今日の既定は local。上書きは `--timezone` > `BIZDATE_TZ` > local | [0004-date-and-timezone.md](0004-date-and-timezone.md) |
| 0005 | decided | 業務日の定義 | 月〜金かつ国民の祝日でなく利用者指定休日でもない日。`--day-off`。config 上書きは将来 | [0005-business-day-definition.md](0005-business-day-definition.md) |
| 0006 | decided | 祝日データ | fetch-holidays がネット取得してローカル保存。判定はローカルのみ。欠落/1年expireはエラー。`--source` 既定は内閣府 CSV | [0006-holiday-data-source.md](0006-holiday-data-source.md) |
| 0007 | decided | CLI の形 | サブコマンド `first`/`last`/`fetch-holidays`。判定 exit 0/1/2。`--day-off`。保存先は XDG data | [0007-cli-shape.md](0007-cli-shape.md) |
| 0008 | decided | 実装言語と配布 | Rust（pure Rust/rustls、Docker ビルド、CI で fmt/clippy/test）。内部用 lib は置くが外部 API の互換性は保証しない。Homebrew は後続視野。まずはローカル+CI で動く CLI | [0008-language-and-distribution.md](0008-language-and-distribution.md) |
| 0009 | decided | 祝日 CSV URL とローカル形式 | 既定 URL は内閣府 `syukujitsu.csv`。ローカルは UTF-8 と先頭メタ行。期限は UTC 暦年、期限以上で無効。年カバー外・不正データはエラー | [0009-holiday-csv-local-format.md](0009-holiday-csv-local-format.md) |
| 0010 | decided | 開発ホスティングと CI 基盤 | GitHub（`kiyohara/bizdate`、public）。Issue / Actions / Releases。Homebrew tap は `kiyohara/homebrew-tap` を共用。CI とリリースは実装が形になってから導入 | [0010-hosting-and-ci-platform.md](0010-hosting-and-ci-platform.md) |
| 0011 | decided | AI agent 体制と設定配置 | Claude Code / Codex / Cursor の 3 体制 + Copilot review。共通正本は `doc/guidelines/` と `.agents/skills/`、tool 固有は薄い入口 | [0011-ai-agent-lineup.md](0011-ai-agent-lineup.md) |
| 0012 | decided | 開発ループ | GitHub Issue 駆動。1 Issue = 1 ブランチ = 1 PR、直列消化。merge は人間。`progress.md` は索引、`working-branch-notes/` は作業メモ | [0012-development-loop.md](0012-development-loop.md) |
| 0013 | decided | ライセンス選定 | MIT License。copyright は `2026 Tomokazu Kiyohara` | [0013-license-selection.md](0013-license-selection.md) |
| 0014 | decided | 依存クレートと toolchain | edition 2024、MSRV 1.98。`clap` / `jiff` / `csv` / `ureq`(rustls) / `encoding_rs`。XDG パス解決は自前実装 | [0014-dependency-crates-and-toolchain.md](0014-dependency-crates-and-toolchain.md) |

## 未決事項

| 主題 | 状態 | 次に決めること | 関連ログ |
|---|---|---|---|
| 配布手段の詳細 | open | GitHub Releases / Homebrew（`dist` + 個人 tap）の具体手順。実装が形になってから着手する | [0008-language-and-distribution.md](0008-language-and-distribution.md), [0010-hosting-and-ci-platform.md](0010-hosting-and-ci-platform.md) |
| 日本以外の祝日対応 | open | 対象国・CSV スキーマ・時期（`--source` 差し替え想定は記録済み） | [0003-v1-scope.md](0003-v1-scope.md), [0006-holiday-data-source.md](0006-holiday-data-source.md) |
| コンフィグファイル | open | `--config` や `~/.config/bizdate/options` など（v1 対象外） | [0005-business-day-definition.md](0005-business-day-definition.md) |

## 運用メモ

- AI agent はまずこの `index.md` を読み、必要な個別ログだけを参照する。
- 未決表は decision log 作業用のダッシュボードである。製品としてまだ決めていないことの本文は `../concept.md` および関連 spec に置く。
- 未決表の状態 `open` は、個別ログがまだ無いこと、または個別ログで後続に残した未決を表す。個別ログの `proposed` とは別語彙である。
- 詳細ログは 1 テーマ 1 ファイルを原則とする。
- 決定が変わった場合は古いログを消さず、状態を `superseded` にして新しいログへリンクする。
- ファイル名は `<連番>-<短い英語slug>.md` を基本とする。
