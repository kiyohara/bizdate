<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# CLI の形

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `doc/design/cli-interface.md`

## 背景

v1 の主用途はゲートである。あわせて祝日データの取得サブコマンドが必要になった（0006）。option、exit code、stdout / stderr、サブコマンドの境界を決めないと、シェルからの利用と実装がぶれる。slapex の exit 分類（0/1/2/3/4）は性質が違うため暗黙採用しない。

## 候補

- A: 判定は root + option。祝日取得だけサブコマンド。exit は predicate（0=yes, 1=no, 2=error）。stdout は `yes` / `no`
- B: すべてサブコマンド（`first` / `last` / `fetch-holidays`）
- C: mode 無指定で `first` / `last` / `neither` を1回の実行で出す
- D: slapex 型の成功 / 失敗 exit（判定の否も失敗扱い）

## 検討内容

判定も取得もサブコマンドに揃えると、help と拡張が一貫する。root に option を載せる案より、`bizdate first` / `bizdate last` / `bizdate fetch-holidays` の方が操作の種類が読み取りやすい。

無指定で3値を返す案は人間向けには便利だが、ゲートでは「どれを見るか」が曖昧になり、exit も1つに落とせない。

`grep` や `test` と同様、判定コマンドの exit 1 は失敗ではなく「否」である。用法誤りやデータ欠落だけを本物の失敗（exit 2）に分けると、`bizdate last && job` が自然に書ける。

利用者指定休日の option 名について、仮の `--holiday` は国民の祝日データ（`fetch-holidays` / `--source`）と語が衝突する。利用者指定は「祝日」ではなく休み日なので `--day-off` を採用する。`--closed` も候補だが、意味の直截さで `--day-off` を選ぶ。

短 option と JSON は v1 の価値に必須ではない。`--quiet` は cron で exit だけ見る用途に有用である。

## 決定

```sh
bizdate first [--date YYYY-MM-DD] [--timezone IANA] [--day-off LIST] [--quiet]
bizdate last  [--date YYYY-MM-DD] [--timezone IANA] [--day-off LIST] [--quiet]
bizdate fetch-holidays [--source URL]
```

- `first` / `last` / `fetch-holidays` はすべてサブコマンドとする
- 日付は `--date`。省略時は採用タイムゾーンの今日（[0004](0004-date-and-timezone.md)）
- タイムゾーン上書きは `--timezone` と環境変数 `BIZDATE_TZ`（優先順位は option > 環境変数 > local。詳細は [0004](0004-date-and-timezone.md)）
- 利用者指定休日は `--day-off`（[0005](0005-business-day-definition.md)）
- 祝日取得先は `fetch-holidays` の `--source`。既定は内閣府 CSV（[0006](0006-holiday-data-source.md)）
- 短 option と JSON 出力は v1 では提供しない
- コンフィグファイルによる option 既定上書きは v1 では提供しない（将来候補）
- 判定サブコマンド（`first` / `last`）の exit code は次とする
  - `0`: 該当する（yes）
  - `1`: 該当しない（no）。失敗ではない
  - `2`: 用法誤り、不正日付、祝日データ欠落 / 期限切れ / 年カバー外、その他のエラー
- 判定の stdout は `yes` または `no` を1行。診断とエラーは stderr
- `--quiet` を判定サブコマンドに提供し、指定時は stdout へ判定結果を出さない（exit だけで使う）
- `fetch-holidays` の exit code は次とする
  - `0`: 取得・保存に成功
  - `2`: 用法誤り、取得失敗、保存失敗、その他のエラー
- `fetch-holidays` の成功時 stdout は、保存したローカルデータの path を1行とする。診断とエラーは stderr
- ローカル祝日データの保存場所は、`$XDG_DATA_HOME/bizdate/holidays/holidays.csv` とする。`XDG_DATA_HOME` 未設定時は `~/.local/share/bizdate/holidays/holidays.csv` とする。UTF-8 と先頭メタ行の形式は [0009](0009-holiday-csv-local-format.md)。取得から1年で無効とする（[0006](0006-holiday-data-source.md)）
- `--help` と `--version` を提供する。help 文面は日本語とする

## 理由

操作種別をサブコマンドで揃え、判定は predicate exit、取得は副作用コマンドとして分離すると、ゲート用途と更新用途の両方に合うため。option 名は祝日データ語彙と衝突しない `--day-off` にする。

## 影響

- CLI 仕様の正本を `cli-interface.md` に置く
- 0005 の option 名を `--day-off` に合わせる
- 実装とテストは predicate exit を前提にする
- 利用者向け README では `fetch-holidays` 後に `bizdate last && job` する例を示す

## 後から見直す条件

- 公開モードが増えてサブコマンド構成を見直す場合
- JSON 出力や短 option の需要が明確になった場合
- exit 1 を失敗とみなす利用者層への互換 option が必要になった場合
- XDG 以外の保存場所が必要になった場合
