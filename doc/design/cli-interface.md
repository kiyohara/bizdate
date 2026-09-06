# CLI インターフェース仕様

この文書は、`bizdate` CLI のコマンド形式、option、入出力ストリーム、exit code の正本である。

想定読者は、このツールを利用する人間と、CLI を実装・検証する担当者である。

日付・業務日・祝日の定義は `business-day.md` を参照する。決定経緯は `decision-log/0003-v1-scope.md`、`decision-log/0007-cli-shape.md`、`decision-log/0008-language-and-distribution.md` を参照する。

## コマンド形式

```sh
bizdate first [--date YYYY-MM-DD] [--timezone IANA] [--day-off LIST] [--quiet]
bizdate last  [--date YYYY-MM-DD] [--timezone IANA] [--day-off LIST] [--quiet]
bizdate fetch-holidays [--source URL]
```

- `first` / `last` / `fetch-holidays` はすべてサブコマンドとする
- 日付引数の positional 指定は提供しない

## 判定サブコマンド（`first` / `last`）

### option 一覧

| option | 値 | default | 制約 | 目的 |
|---|---|---|---|---|
| `--date` | `YYYY-MM-DD` | 採用 TZ の今日 | `business-day.md` の日付規則 | 判定対象日を指定する |
| `--timezone` | IANA 名 | （下記） | 既知の IANA 名 | 「今日」と年省略の基準 TZ を上書きする |
| `--day-off` | カンマ区切り日付リスト | なし | `business-day.md` の利用者指定休日 | 追加の休日を与える |
| `--quiet` | flag | off | | stdout へ判定結果を出さない。exit code だけで使う |

タイムゾーンの解決順は `--timezone` > `BIZDATE_TZ` > local timezone とする（`business-day.md`）。

- 短 option は v1 では提供しない
- JSON 出力は v1 では提供しない
- help 文面は日本語とする

未知の option、不正な組み合わせ、不正な日付形式は usage またはエラーを stderr に出し、exit code `2` で終了する。

### 入出力ストリーム

| stream | 内容 |
|---|---|
| stdout | 判定結果のみ。成功した判定では `yes` または `no` を1行出力する。`--quiet` 指定時は何も出さない |
| stderr | 診断、エラー、usage |

### exit code

| code | 意味 | 例 |
|---:|---|---|
| `0` | 該当する（yes） | `last` で指定日がその月の最後の業務日である |
| `1` | 該当しない（no）。失敗ではない | `last` で指定日がその月の最後の業務日でない |
| `2` | エラー | 用法誤り、不正日付、祝日データ欠落 / 期限切れ / 年カバー外、内部エラー |

`grep` や `test` と同様、exit code `1` は失敗ではなく判定の「否」である。シェルでは次のようにゲートできる。

```sh
bizdate last && monthly-job
```

## 取得サブコマンド（`fetch-holidays`）

### option 一覧

| option | 値 | default | 制約 | 目的 |
|---|---|---|---|---|
| `--source` | URL | `https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv` | http(s) URL | 祝日データの取得先 |

### 入出力ストリーム

| stream | 内容 |
|---|---|
| stdout | 成功時に、保存したローカルデータの path を1行出力する |
| stderr | 診断、エラー、usage |

### exit code

| code | 意味 | 例 |
|---:|---|---|
| `0` | 取得・保存に成功 | ローカルデータを更新した |
| `2` | エラー | 用法誤り、取得失敗、保存失敗、内部エラー |

## 共通

| option | 目的 |
|---|---|
| `--help` | usage を表示して終了する |
| `--version` | version を表示して終了する |

`--help` と `--version` は正常終了とし、exit code `0` とする。

ローカル祝日データの保存場所は `$XDG_DATA_HOME/bizdate/holidays/holidays.csv`（未設定時は `~/.local/share/bizdate/holidays/holidays.csv`）とする。UTF-8・先頭メタ行の形式は `business-day.md` を参照する。

## 実装方針との関係

- 実装言語は Rust とする。公開ライブラリクレートとしては提供しない
- 依存は pure Rust に寄せ、HTTP の TLS は rustls を使う
- ビルドとテストの再現は Docker（または Compose）を正とする
- 詳細は [0008](decision-log/0008-language-and-distribution.md)

## v1 で扱わないこと

- 短 option
- JSON 出力
- 業務日 / 休日判定の公開 mode
- コンフィグファイルによる option 既定上書き
- Windows 向けの動作保証
- GitHub Releases / Homebrew の具体手順（動作確認後の後続）
