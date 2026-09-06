# コンセプト

この文書は、`bizdate` の現在合意しているコンセプトの正本である。詳細仕様は関連 spec を正とし、ここでは合意済みの概要だけを残す。決定経緯は `decision-log/` を参照する。

## 目的

指定した日付（省略時はコマンド実行日）が、その月の「最初の業務日」または「最後の業務日」かを判定するコマンドラインツールを作る。

曜日だけでなく、日本の祝日も扱う。主用途は cron / シェルからのゲートである。

## 決まっていること

- コマンド名は `bizdate` とする。経緯は [0002](decision-log/0002-cli-command-name.md)
- 主目的は、その月の最初 / 最後の業務日判定である
- 主用途はゲートとする。対話 UI は持たない。経緯は [0003](decision-log/0003-v1-scope.md)
- v1 の公開機能は、判定サブコマンド（`first` / `last`）と祝日取得サブコマンド（`fetch-holidays`）とする。経緯は [0003](decision-log/0003-v1-scope.md)、[0006](decision-log/0006-holiday-data-source.md)、[0007](decision-log/0007-cli-shape.md)
- v1 の既定カレンダーは日本の国民の祝日とする。経緯は [0003](decision-log/0003-v1-scope.md)
- 業務日 / 休日判定の公開 mode、埋め込み会社カレンダー、翌 / 前営業日、第 N 営業日、営業日数は v1 対象外とする。経緯は [0003](decision-log/0003-v1-scope.md)
- 判定対象はカレンダー日付のみ。入力は `YYYY-MM-DD`。今日の既定は local timezone。上書きは `--timezone` > `BIZDATE_TZ` > local。経緯は [0004](decision-log/0004-date-and-timezone.md)
- 業務日は月〜金かつ国民の祝日でなく利用者指定休日でもない日。利用者指定は `--day-off`。経緯は [0005](decision-log/0005-business-day-definition.md)
- 祝日は `fetch-holidays` が取得してローカル保存し、判定はローカルのみを読む。欠落 / 1年 expire / 年カバー外はエラー。`--source` 既定は `https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv`。ローカルは UTF-8 CSV で先頭にメタ行を埋め込む。経緯は [0006](decision-log/0006-holiday-data-source.md)、[0009](decision-log/0009-holiday-csv-local-format.md)
- CLI・入出力・exit code の詳細は `cli-interface.md`。経緯は [0007](decision-log/0007-cli-shape.md)
- 実装言語は Rust（pure Rust / rustls、Docker でビルド再現、CI で fmt・clippy・test）。Homebrew は後続視野。経緯は [0008](decision-log/0008-language-and-distribution.md)

## 関連仕様

- 日付・業務日・祝日: [`business-day.md`](business-day.md)
- CLI: [`cli-interface.md`](cli-interface.md)

## 次点の候補（v1 対象外）

必要になった時点で再検討する。

- 業務日か休日かを判定する公開 mode
- 埋め込み会社カレンダーや、設定ファイルによる休日集合の本格対応
- option 既定をコンフィグファイルで上書きする仕組み（例: `--config`、`~/.config/bizdate/options`）
- 翌 / 前営業日、第 N 営業日、営業日数
- 日本以外の祝日への本格対応（`--source` 差し替え想定は記録済み）
- GitHub Releases / Homebrew による配布（動作確認後）
- 公開ライセンス（公開直前。候補は MIT）

## まだ決めていないこと

実装前のプロダクト方針は上記まで決まった。残る未決は次のとおりである。

- 公開時のライセンス
- 配布手段（Releases / Homebrew）の詳細
- 日本以外の祝日 CSV のスキーマ
