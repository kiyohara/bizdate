# bizdate

`bizdate` は、指定した日付（省略時はコマンド実行日）が、その月の最初または最後の業務日かを判定するコマンドラインツールです。曜日に加え、日本の祝日を扱います。

主用途は、cron やシェルから月次処理のゲートとして使うことです。業務日は、月曜日から金曜日のうち、日本の祝日・振替休日・国民の休日と、利用者が指定した休日を除いた日です。会社固有の休日や、元日以外の年末年始休暇は自動では除外しません。

現在、配布用バイナリは提供していません。インストール方法は、配布手段が固まってから追記します。以下は `bizdate` を実行できる環境での利用方法です。

## 祝日データの準備・更新

判定する前に、祝日データを取得してください。

```sh
bizdate fetch-holidays
```

`fetch-holidays` は、既定では [内閣府の祝日 CSV](https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv) を取得してローカルに保存し、保存先の path を stdout に 1 行出力します。再実行するとデータを更新します。

- 保存先は `$XDG_DATA_HOME/bizdate/holidays/holidays.csv` です。`XDG_DATA_HOME` が未設定の場合は `~/.local/share/bizdate/holidays/holidays.csv` に保存します。
- `XDG_DATA_HOME` を指定する場合は絶対 path にしてください。空文字や相対 path はエラーになります。
- 有効期間は取得時から 1 年です。期限切れになる前に再取得してください。祝日の追加・変更を反映する場合も再取得が必要です。
- `first` / `last` は保存済みのデータだけを使い、ネットワークから自動取得しません。データの欠落・期限切れ・対象年のカバー外は、土日かどうかにかかわらずエラーになります。

| option | 説明 | default |
|---|---|---|
| `--source URL` | 取得先を http(s) URL で指定します。ローカルファイルの path は指定できません | 内閣府の祝日 CSV |

取得先を変更する場合も、CSV は `国民の祝日・休日月日,国民の祝日・休日名称` の 2 列で、日付は `YYYY/M/D` 形式である必要があります。保存時には UTF-8 に変換します。

## 月の最初・最後の業務日を判定する

| subcommand | 判定内容 |
|---|---|
| `first` | 指定日が、その月の最初の業務日かどうか |
| `last` | 指定日が、その月の最後の業務日かどうか |

日付を指定する場合は `--date` を使います。省略すると、採用するタイムゾーンの今日を判定します。

```sh
bizdate first --date 2026-09-01
bizdate last --date 2026-09-30
```

判定結果は stdout に `yes` または `no` を 1 行出力します。業務日が 1 日だけの月では、その日は `first` と `last` の両方に該当します。業務日がない月では、どの日も該当しません。

### 判定の option

`first` / `last` で共通です。

| option | 説明 | default |
|---|---|---|
| `--date YYYY-MM-DD` | 判定する日付を指定します。存在しない日付や別の形式はエラーになります | 採用するタイムゾーンの今日 |
| `--timezone IANA` | `Asia/Tokyo` などの IANA 名で、「今日」の基準タイムゾーンを指定します | `BIZDATE_TZ`、未設定なら実行環境の local timezone |
| `--day-off LIST` | 追加の休日をカンマ区切りで指定します。各日付は `YYYY-MM-DD` または `MM-DD` 形式です | なし |
| `--quiet` | stdout への判定結果の出力を抑えます。exit code は変わらず、エラーは stderr に出ます | off |

タイムゾーンの優先順位は `--timezone` > 環境変数 `BIZDATE_TZ` > local timezone です。採用する値が空文字や未知の名前の場合はエラーになり、下位の指定には切り替わりません。

`--day-off` の年を省略した `MM-DD` は、採用するタイムゾーンの「今日の年」で解釈します。`--date` の年ではありません。別の年を判定するときは、休日にも年を指定してください。範囲指定、不正な日付、空要素は使えません。

```sh
bizdate last --date 2026-09-30 --day-off 2026-09-30,2026-12-31
```

### exit code

| code | `first` / `last` | `fetch-holidays` |
|---:|---|---|
| `0` | 該当します（`yes`） | 取得・保存に成功しました |
| `1` | 該当しません（`no`） | 使用しません |
| `2` | エラーです | エラーです |

**exit code `1` は失敗ではなく、判定の「否」です。** エラーには、不正な option・日付・タイムゾーン・`--day-off`、祝日データの欠落・期限切れ・年カバー外、取得・保存の失敗などがあり、診断を stderr に出力します。

## シェル・cron から使う

祝日データを事前に取得しておけば、月の最後の業務日だけ後続の処理を実行できます。`monthly-job` は実行したいコマンドに置き換えてください。

```sh
bizdate last --quiet --timezone Asia/Tokyo && monthly-job
```

`&&` の後続は、判定が exit code `0` の場合だけ実行されます。`1` と `2` のどちらでも後続は実行されないため、エラーを見逃さないよう stderr も確認してください。

cron では、判定を毎日実行し、祝日データは期限前に更新します。次の例では毎月 1 日の 6 時にデータを更新し、毎日 9 時に月末の業務日かを判定します。初回の判定より前に、同じ実行ユーザー・保存先で `fetch-holidays` を実行してください。

```cron
0 6 1 * * /path/to/bizdate fetch-holidays
0 9 * * * /path/to/bizdate last --quiet --timezone Asia/Tokyo && /path/to/monthly-job
```

`/path/to/bizdate` と `/path/to/monthly-job` は実際の絶対 path に置き換えてください。取得と判定で同じデータを参照するよう、`HOME` と、設定する場合は `XDG_DATA_HOME` を揃えてください。cron の起動時刻は cron 側のタイムゾーンに従います。`--timezone` は bizdate が「今日」を解釈するための指定です。

## help・version

```sh
bizdate --help
bizdate first --help
bizdate last --help
bizdate fetch-holidays --help
bizdate --version
```

help と version は stdout に表示し、exit code `0` で終了します。`--version` は root コマンドだけで使えます。短い option は提供していません。

## ライセンス

[MIT License](LICENSE) で公開しています。

## 開発者向けドキュメント

- ドキュメント配置の入口: [`doc/README.md`](doc/README.md)
- AI agent / 開発者向け共通入口: [`AGENTS.md`](AGENTS.md)
