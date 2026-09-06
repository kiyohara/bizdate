<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 実装言語と配布

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `doc/design/concept.md`, `doc/design/business-day.md`, `doc/design/cli-interface.md`

## 背景

プロダクト契約（スコープ、日付、業務日、祝日、CLI）が固まったので、言語・依存・対象プラットフォーム・配布・公開 API の有無を決める。当初案は Go だったが、新しい取り組みとして Rust を試したい意向があり、Docker ビルド、Homebrew 連携、十分な CI テストを要件とした。Go と比べた Rust エコシステムの成熟度を調査したうえで判断する。

## 候補

- A: Go。stdlib-first。HTTP 取得とローカル保存。最初の目標はローカルで動く CLI。Releases / Homebrew は後続
- B: Rust。条件付き（pure Rust 依存、Docker でビルド再現、CI を厚く、Homebrew は後続視野）。Releases / Homebrew は動作後
- C: Python / TypeScript。実装は速いが、単一バイナリ配布と依存固定が弱い
- D: 最初から GitHub Releases と Homebrew まで v1 必須にする

## 検討内容

調査の結果、Docker・Homebrew・CI の三点について現行 Rust エコシステムは要件を満たせる。

- CI: `dtolnay/rust-toolchain`、`Swatinem/rust-cache`、`cargo fmt` / `clippy` / `test` が定石として確立している
- Docker: 公式 `rust` イメージと `cargo-chef` による依存レイヤキャッシュが定番。クロスビルドは `cross` など Docker 前提の道具が強い
- Homebrew: `dist`（旧 cargo-dist）で GitHub Releases と個人 tap への formula 更新が自動化できる。slapex と同型の配布が可能

Go の方が楽な点は、クロスコンパイルの手軽さとビルド速度、学習コストである。一方「未熟で要件を満たせない」わけではない。未熟に見える主因は C 依存付きクロスビルドであり、`bizdate` は clap / chrono / csv / HTTP（rustls）で pure Rust に寄せられる。

ライブラリ API を公開すると semver と互換約束が増える。v1 は CLI が成果物なので公開 API は持たない。

最初から Releases / Homebrew を必須にすると、判定ロジックより配布整備が先に来る。ローカルと CI で正しく動く CLI を先に作る。Homebrew は視野に入れ、動作確認後に着手する。

公開配布の自動化（`dist`）は GitHub Releases 前提が強い。開発ホスティングと CI 基盤は GitHub とするため（[0010](0010-hosting-and-ci-platform.md)）、この前提はそのまま満たせる。

## 決定

- 実装言語は Rust とする
- 次の条件を守る
  1. 依存は pure Rust に寄せる。HTTP の TLS は rustls を使い、OpenSSL 等の C 依存を引き込まない
  2. ビルドとテストの再現は Docker（または Compose）を正とする
  3. CI では少なくとも `fmt`、`clippy`、`test` を行い、必要なら OS matrix を足す
  4. Homebrew 連携は視野に入れるが、最初の実装目標には含めない。後続で `dist` と個人 tap を想定する
- 対象プラットフォームは macOS と Linux の amd64 / arm64 とする。Windows は初期対象外とする
- 公開ライブラリクレートとしては提供しない。実装モジュールはクレート内に閉じる
- 最初の実装目標は、ローカルと CI で正しく動く CLI とする。GitHub Releases および Homebrew による配布は、動作確認後の後続作業とする
- 配布は GitHub Releases を起点とし、Homebrew 連携は `dist` に委ねる（ホスティングの決定は [0010](0010-hosting-and-ci-platform.md)）
- 公開時のライセンスは MIT とする（[0013](0013-license-selection.md)）

## 理由

利用者の「Rust で新しく取り組む」意向と、Docker / Homebrew / CI の要件を、現行エコシステムで満たせるため。条件付きにすることで、Go 比で痛い Cros 依存の罠を避ける。

## 影響

- リポジトリに Cargo プロジェクトを置く
- Dockerfile / Compose と CI workflow を初期から用意する
- 利用者向けのインストール手順は、配布手段が固まってから `README.md` / 将来の `doc/help/` に書く
- 当初の Go 仮説（素案 A）は採用しない

## 後から見直す条件

- pure Rust を保てず C 依存が不可避になり、クロスビルドコストが過大になった場合
- ビルド時間や学習コストが開発速度を明らかに阻害する場合
- Windows 対応や Homebrew が初期公開の必須条件になった場合
