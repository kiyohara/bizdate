<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 依存クレートと toolchain の選定

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `doc/design/business-day.md`, `doc/design/cli-interface.md`, `doc/design/decision-log/0008-language-and-distribution.md`, `doc/design/decision-log/0009-holiday-csv-local-format.md`, `doc/design/decision-log/0010-hosting-and-ci-platform.md`, `doc/design/decision-log/0013-license-selection.md`

## 背景

[0008](0008-language-and-distribution.md) で実装言語を Rust とし、「依存は pure Rust に寄せる」「HTTP の TLS は rustls」「ビルドとテストの再現は Docker（または Compose）を正とする」までを決めた。ただし具体的な採用クレート、edition、MSRV は未確定だった。

Cargo プロジェクトを置くにあたり、次を確定する必要がある。

- 仕様が要求する 6 系統（CLI パーサ、日付・タイムゾーン、CSV、HTTP client、文字コード変換、XDG パス解決）の採用クレート
- edition と MSRV。MSRV は開発用 Docker image の tag と CI の toolchain を揃える基準になる

## 候補

### toolchain

- A: MSRV を開発 image と同じ最新 stable に固定し、1 系統だけ保守する
- B: MSRV を数バージョン後ろに置き、CI で MSRV ジョブを別途回す

### 依存クレート

| 系統 | 候補 |
|---|---|
| CLI パーサ | `clap`（derive）/ `argh` / `pico-args` / 自前 |
| 日付・タイムゾーン | `jiff` / `chrono` + `chrono-tz` / `time` |
| CSV | `csv` / 自前パーサ |
| HTTP client | `ureq` / `reqwest`（`default-features = false` + rustls） |
| 文字コード変換 | `encoding_rs` / 自前の Shift_JIS 変換表 |
| XDG パス解決 | `directories` / `dirs` / 自前 |

## 検討内容

### toolchain

配布は GitHub Releases のビルド済みバイナリを起点とする（0008、[0010](0010-hosting-and-ci-platform.md)）。利用者が自分の toolchain でソースからビルドする経路を主要動線に置かないため、古い toolchain を支える必要が薄い。一方で MSRV を後ろに置くと、CI に MSRV 用ジョブが増え、開発 image の toolchain と CI toolchain の 2 系統を保守することになる。

edition は 2024 を使う。edition 2024 は Rust 1.85 以降で利用でき、選定する MSRV はこれを十分に上回る。

### CLI パーサ

`cli-interface.md` の要件は、サブコマンド 3 つ、long option のみ、日本語 help、`--version` である。`clap` は derive でこれらを満たせ、ライセンスは MIT OR Apache-2.0、pure Rust である。バイナリサイズは `argh` / `pico-args` より大きいが、v1 では配布サイズを制約に置いていない。

ただし「日本語 help」は derive の記述だけでは完全には満たさない。about と各 option の説明は日本語になるが、`Usage:` / `Options:` の section heading は clap 側の文字列であり、`help_template` と `next_help_heading` で明示的に上書きしない限り英語のまま残る。help 文面をどこまで日本語にするか（heading まで揃えるか、説明文が日本語なら足りるとするか）は、CLI の option を配線する Issue #12 で決める。

「短 option は v1 では提供しない」という仕様は clap 既定の `-h` / `-V` にも及ぶと読める。`disable_help_flag` / `disable_version_flag` を立て、`ArgAction::Help` / `ArgAction::Version` を long option として明示的に定義すれば満たせることを実機で確認した。`-V` は exit code 2 で拒否される。

### 日付・タイムゾーン

`business-day.md` の要件は、`YYYY-MM-DD` のパース、IANA 名によるタイムゾーン解決、採用タイムゾーンにおける「今日」、暦月の境界である。

`chrono` は IANA 名の解決に `chrono-tz`（tzdb をバイナリへ埋め込む）、local timezone の検出に `iana-time-zone` を足す構成になり、3 クレートに分かれる。`jiff` は単一クレートで civil date、IANA 名の解決、local timezone の取得、暦月の境界を扱える。対象プラットフォームは macOS と Linux のみで（0008）、いずれも system tzdb を持つため、tzdb を埋め込まない既定構成で足りる。

`jiff` は 0.x であり、semver 上は破壊的変更があり得る。ただし公開ライブラリクレートを提供しないため（0008）、破壊的変更の影響はこのクレート内に閉じる。

### CSV

取得元 CSV の列構成はそのまま保持する（[0009](0009-holiday-csv-local-format.md)）。祝日名に区切り文字や quote が現れた場合を自前パーサで正しく扱うのは、書くコストより読み違えるリスクの方が大きい。`csv` クレートは pure Rust、ライセンスは Unlicense/MIT、API が安定した 1.x である。

### HTTP client

`fetch-holidays` の通信は 1 回の GET だけである。`reqwest` は非同期ランタイム（tokio）を引き込むため、この用途には過大である。`ureq` は blocking API のみで、rustls を feature で選べる。

`default-features = false` + `features = ["rustls"]` で構成した依存ツリーを確認したところ、`rustls` の crypto provider は `ring` に解決され、`openssl` / `native-tls` / `aws-lc-rs` はいずれも入らなかった。`ring` は少量の C / アセンブリを含むが vendored であり、システムライブラリへのリンクを要求しない。0008 が避けようとしたのはシステムの C ライブラリ依存によるクロスビルドの破綻であり、この構成はその意図を満たす。ルート証明書は `webpki-roots` で同梱され、実行環境の証明書ストアに依存しない。

### 文字コード変換

取得元は Shift_JIS / CP932 系である（0009）。CP932 は Shift_JIS の拡張であり、ベンダ定義文字を含む変換表を自前で持つのは保守コストに見合わない。`encoding_rs` は Web ブラウザで実績のある実装で、pure Rust である。ライセンスは `(Apache-2.0 OR MIT) AND BSD-3-Clause` で、いずれも permissive であり MIT での配布（[0013](0013-license-selection.md)）と両立する。BSD-3-Clause 部分は配布物に著作権表示を含める必要があるため、リリース成果物へ third-party ライセンス表記を含める段で扱う。

### XDG パス解決

`cli-interface.md` は保存先を `$XDG_DATA_HOME/bizdate/holidays/`、未設定時は `~/.local/share/bizdate/holidays/` と定めている。`directories` / `dirs` は macOS で `~/Library/Application Support` を返すため、この仕様と一致しない。仕様どおりに実装するには環境変数 2 つを読むだけで足りるので、クレートを足さない。

## 決定

- edition は `2024` とする
- MSRV は `1.98` とする（候補 A）。`Cargo.toml` の `rust-version`、開発 image の tag、CI の toolchain をこの値で揃える
- 開発用 image は `rust:<MSRV>-trixie` を base にし、`rustup component add rustfmt clippy` を足した image を使う。公式 image は minimal profile のため rustfmt と clippy を含まない
- 採用クレートは次とする

| 系統 | 採用 | pure Rust | ライセンス |
|---|---|---|---|
| CLI パーサ | `clap`（`derive`） | yes | MIT OR Apache-2.0 |
| 日付・タイムゾーン | `jiff` | yes | Unlicense OR MIT |
| CSV | `csv` | yes | Unlicense/MIT |
| HTTP client | `ureq`（`default-features = false`, `features = ["rustls"]`） | yes（`ring` は vendored な C / asm を含む） | MIT OR Apache-2.0 |
| 文字コード変換 | `encoding_rs` | yes | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| XDG パス解決 | クレートを追加せず自前実装 | - | - |

- バージョン指定は caret 要件（`clap = "4.6"` など）とし、`Cargo.lock` を commit して再現性を担保する
- error 処理クレート（`anyhow` / `thiserror` 等）は本ログの対象外とする。必要になった時点で、それを使う Issue で判断する

## 理由

6 系統のうち 5 系統は、仕様が要求する挙動を pure Rust で満たし、かつ保守されている定番クレートが存在するため。XDG パス解決だけは定番クレートの挙動が仕様と食い違うため、依存を増やさず自前実装とした。

MSRV を開発 image と同じ最新 stable に固定したのは、配布がビルド済みバイナリ起点であり、複数 toolchain を保守する利益が現時点で無いためである。

## 影響

- `Cargo.toml` に上記クレートと `rust-version` を記載する
- `Dockerfile` と `compose.yaml` を置き、開発コマンドは Compose 経由を原則とする（`doc/guidelines/development-command-guidelines.md`）
- MSRV の値は `Cargo.toml` の `rust-version`、`Dockerfile` の `FROM`、`compose.yaml` の image tag の 3 箇所に現れる。上げるときは 3 つを同時に更新する（`doc/guidelines/development-command-guidelines.md`）
- 日本語 help を section heading まで揃えるかどうかは Issue #12 で決める。derive の記述だけでは `Usage:` / `Options:` は英語のまま残る
- CI（Issue #8）は本ログの MSRV と同じ toolchain で `fmt` / `clippy` / `test` を回す
- `fetch-holidays`（Issue #13）は `ureq` + `encoding_rs` で取得と UTF-8 変換を行う
- 祝日データの保存先解決（Issue #10）は `XDG_DATA_HOME` と `HOME` を自前で読む
- リリース成果物に third-party ライセンス表記を含める必要がある（`encoding_rs` の BSD-3-Clause 部分、`ring` 等）。配布手段を決める段で扱う

## 後から見直す条件

- `jiff` の破壊的変更や保守停止で、`chrono` + `chrono-tz` への移行が必要になった場合
- MSRV を固定したことでビルドできない環境が実際に問題になった場合
- rustls の crypto provider 既定が変わり、システムライブラリ依存が入る場合
- バイナリサイズや起動時間が実際の制約になった場合
