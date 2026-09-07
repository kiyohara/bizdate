<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 祝日 CSV の既定 URL とローカル保存形式

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-07
- 関連: `doc/design/business-day.md`, `doc/design/cli-interface.md`, `doc/design/decision-log/0006-holiday-data-source.md`

## 背景

0006 で祝日データの取得・ローカル保存・expire は決まったが、既定 URL の具体値と、ローカルファイルのエンコーディング・メタデータ載せ方が未決だった。実装と運用のために固定する。

## 候補

- A: 既定 URL を内閣府案内ページ記載の `syukujitsu.csv` とし、ローカルは UTF-8 CSV。bizdate メタデータはファイル先頭行に埋め込む
- B: ローカルは取得バイト列をそのまま保存し、メタデータは別ファイル（sidecar）にする
- C: ローカルは JSON など独自形式に変換して保存する

## 検討内容

内閣府は [国民の祝日について](https://www8.cao.go.jp/chosei/shukujitsu/gaiyou.html) で CSV を案内しており、実体 URL は `https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv` である。取得実体は Shift_JIS（CP932）系で、ヘッダは `国民の祝日・休日月日,国民の祝日・休日名称`、日付は `YYYY/M/D` 形式である。

ローカルを UTF-8 に揃えると、端末やエディタでの確認が楽になる。取得元の列構成はそのまま残し、エンコーディングだけ変換するのが安全である。

メタデータを別ファイルにすると欠落や不整合が起きやすい。同一ファイル先頭へ行を足す案は、管理対象が1ファイルで済み、expire 判定に必要な取得日時も同居できる。先頭行は `#` で始まるメタ行とし、本体 CSV と見分けられるようにする。

## 決定

- `--source` 未指定時の既定 URL は次とする
  - 案内: https://www8.cao.go.jp/chosei/shukujitsu/gaiyou.html
  - CSV: https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv
- ローカル保存ファイルは UTF-8 の CSV とする。取得元が Shift_JIS / CP932 等の場合は、保存時に UTF-8 へ変換する
- ローカルファイル名は `holidays.csv` とし、保存ディレクトリは 0007 で決めた `$XDG_DATA_HOME/bizdate/holidays/`（未設定時は `~/.local/share/bizdate/holidays/`）配下とする
- bizdate 用メタデータは、同一ファイルの先頭に行を追加して埋め込む。メタ行は `#` で始める
- メタ行に少なくとも次を記録する
  - `schema`: メタ形式の版。初期値は `1`
  - `fetched_at`: 取得日時（UTC の RFC 3339）
  - `source_url`: 実際に取得した URL
- 有効期限は `fetched_at` から1年後とし、メタに `expires_at` を書いてよい（書かない場合は読取時に `fetched_at + 1 year` で算出する）
- メタ行の直後に、UTF-8 化した取得元 CSV の本文（ヘッダ行とデータ行）を続ける。列構成は取得元のままとする
- 判定・解釈時は、先頭の `#` 行をスキップしてから CSV 本文を読む

メタ行の例:

```text
# bizdate-meta schema=1
# fetched_at=2026-09-06T02:24:33Z
# source_url=https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv
# expires_at=2027-09-06T02:24:33Z
国民の祝日・休日月日,国民の祝日・休日名称
1955/1/1,元日
...
```

## 理由

公式案内の URL を既定に固定でき、UTF-8 + 先頭メタ行なら単一ファイルで取得経緯と本体を管理できるため。

## 影響

- `business-day.md` と `cli-interface.md` の既定 URL・保存形式を更新する
- `fetch-holidays` は取得後に文字コード変換とメタ行付与を行う
- 他国 CSV を `--source` で指定する場合も、同じローカル形式（UTF-8・先頭メタ・本文 CSV）に正規化する

## 後から見直す条件

- 内閣府が CSV の URL・文字コード・列構成を変更した場合
- 先頭 `#` 行では足りないメタデータが出た場合
- 他国 CSV が `#` コメントと衝突する形式だった場合

## 追記 (2026-09-07): 読み取り境界

Issue #10 でローカル読み取りを実装し、期限ちょうどの扱いをテストで固定した。仕様は `business-day.md` の「ローカル読み取りの境界」に追記した。

- 1 年を固定の 365 日とする案では、うるう年をまたぐと周年日時からずれるため、UTC の暦年を加算する。2 月 29 日は翌年の 2 月末へ丸め、時刻を保つ。
- 有効期間を半開区間とし、`now >= expires_at` で無効とする。明示の `expires_at` は計算値より優先する。読み取り済みの集合も照会時に期限と年カバーを検証する。
- 必須メタの欠落・重複、未対応 schema、ヘッダ・日付の不正、空の祝日集合は読み取りエラーにする。破損データを部分的な祝日集合へ変えて成功させないためである。未知の先頭メタ行は読み飛ばす。
- 年カバーは Issue の「データに含まれる年の範囲」に従い、最小年から最大年の閉区間とする。祝日法の再計算による欠落補完はしない。
- 保存先は環境変数から自前で解決するという 0014 の判断を維持する。空文字や相対 path を基点として作業ディレクトリへ読み書きすることを避けるため、採用する基点は絶対 path に限定し、不正値はエラーとする。`HOME` は `XDG_DATA_HOME` 未設定時だけ参照する。
