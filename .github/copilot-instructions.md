# Copilot Review レビューガイドライン

I want to review in Japanese.

## レビューの目的

- 目的はマージ前に修正すべき問題を見つけることであり、改善案の網羅的な列挙ではない。**重要な問題が無ければ、追加コメントを生成しないことを正しい結果とする。**
- 本リポジトリは v1 の CLI 実装を完了している。実装言語は Rust で、`src/` に実装と unit test、`tests/` に実バイナリを起動する統合テストがある。
- CI（GitHub Actions）が PR ごとに、`cargo fmt --check` / `clippy`（warning を error 扱い）/ 第三者 action の SHA 固定を Linux で、`test`（unit / CLI E2E）/ release `build` / release バイナリの起動確認を配布対象 3 target（macOS arm64、Linux arm64 / x86_64）の native runner で検査する。フォーマット違反、clippy の lint、ビルドエラー、対象 OS でのテスト失敗は CI が検出するため指摘しない。
- Release workflow（`.github/workflows/release.yml`）も PR ごとに走り、3 target の配布 archive を作って、checksum、archive の構成、third-party 表記と依存の対応、展開した binary の起動を検証する。公開（GitHub Release の作成）は `v<version>` tag の push でだけ行う。
- Release workflow は `custom-ci` として `ci.yml` を呼ぶ。PR の run では直接の CI と重複する `platform` を省いて `lint` だけを回し、tag push では全 job を回す。build job の cargo-about は、upstream の prebuilt を `.github/scripts/install-cargo-about.sh` に固定した sha256 と照合して入れる。どちらも decision log 0022 で決定済みであり、PR で `custom-ci` の `platform` が skipped になることと、cargo-about をソースからビルドしないことは指摘しない。`ci.yml` の `platform` の条件が、省く場合（PR の run で release workflow から呼ばれたとき）だけを列挙する形から外れ、tag push で省かれうる変更は指摘する。
- dist 0.32.0 が checksum file（`.sha256` と `sha256.sum`）の末尾に足す空行は、`release-verify.yml` が取り除く。`build-global-artifacts` と並行して走るため、この run の artifact の一覧を読んで待ち（`actions: read`）、build の workflow artifact を `overwrite` で置き換えてから、`host` と同じ形で検査する（decision log 0022 の 2026-09-25 追記）。待ち合わせと置き換えの仕組み自体は指摘しない。置き換えが、build の artifact を取得する `archive`、`homebrew-formula`、`homebrew` の job と並行しうる変更と、`checksum files` の検査を外す・緩める変更は指摘する。prerelease では `homebrew-formula` と `homebrew` が skipped になる。このとき `wait for build artifacts`、`normalize checksums`、`checksum files` のいずれかが skipped になる変更（状態関数を含む `if` を外すなど）も指摘する。
- 配布仕様は `doc/design/distribution.md` で決定済みで、配布対象 3 target の CI と release workflow、Homebrew Formula の生成・検証と tap への更新 job は導入済みである。`v0.1.0` を公開し、README のインストール案内は、公開後確認で確かめた経路（archive、Homebrew）を有効な案内としている（`doc/guidelines/release-guidelines.md`）。公開後確認で確かめていない経路を、予定と明示せずに案内する変更は指摘する。
- tap への書き込みは dist の builtin の publish job ではなく custom の `.github/workflows/publish-homebrew.yml` で行い、dist が「Homebrew publish job が無効」と WARN を出すのは想定どおりである（decision log 0025）。`publish-homebrew-formula.sh` の書かない条件（prerelease、巻き戻し、同じ version の内容違い、`Formula/bizdate.rb` 以外の変更）を緩める変更は指摘する。
- macOS は Apple Silicon だけを配布対象とする。Intel Mac（`x86_64-apple-darwin`）は `doc/design/distribution.md` の「対象外」で決定済み（経緯は decision log 0016）であり、その CI、archive、手動ビルドの手順が無いことは指摘しない。

## 原則

- `Pull Request Overview` を含め、すべて日本語で出力する。
- すべてのコメントに、内容に応じて次のいずれかの prefix を必ず付ける。

| prefix | 使う場面 | マージへの影響 |
| --- | --- | --- |
| `[must]` | correctness / security / reliability / 秘密情報混入に影響する問題 | マージ前に修正が必要 |
| `[ask]` | 意図や前提の確認が必要 | 回答次第で要修正 |
| `[imo]` | 保守性向上の提案 | 影響なし（任意） |
| `[nits]` | 軽微な改善提案 | 影響なし（任意） |
| `[fyi]` | 情報共有のみ | 影響なし |

指摘は重要度の高い順に提示する。correctness → security / privacy → reliability → maintainability → guideline 違反。

## 情報統制

本リポジトリは public である。認証情報、API key、access token、秘密鍵、個人情報、認証情報付き URL、開発機のローカル絶対 path の混入は `[must]` で指摘する。secret は 1Password の secret reference（`op://...`）で書き、実値を置かない。

## 原則として指摘しない事項

マージを妨げない限り指摘しない。指摘する場合も `[imo]` / `[nits]` / `[fyi]` に留める。

- 個人の好みに依存する命名、コードスタイル / フォーマット
- 任意のリファクタリング、将来的な改善提案
- `_template.md` などひな形の未記入項目

## ドキュメント構成のレビュー観点

本リポジトリは「共通正本 + 薄い入口」の構成を取る。恒久ルールの本文は `doc/guidelines/` に置き、`.cursor/rules/`、`.claude/rules/`、`AGENTS.md`、`CLAUDE.md` は正本への参照だけを持つ。

- 入口に恒久ルールが直接書かれていたら指摘する。
- rule の追加・削除・rename では、共通正本・Cursor 入口・Claude Code 入口の basename が揃っているか、`AGENTS.md` にリンクがあるかを確認する。**Codex は `AGENTS.md` からしか正本に到達できないため、リンク漏れは `[must]` で指摘する。**
- 存在しないファイルへの参照とリンク切れは指摘する。
- `.claude/settings.json` は SessionStart hook の登録だけを持つ。処理本体や恒久ルールが書かれていたら指摘する。処理本体は `.agents/scripts/` に置く（`doc/guidelines/cloud-session-guidelines.md`）。
- `doc/design/` 直下の spec（`concept.md` / `business-day.md` / `cli-interface.md` / `distribution.md`）が仕様の正本である。spec 間の矛盾や decision log の決定との食い違いを指摘する。decision log を仕様の正本として扱う記述も指摘する。
- `distribution.md` は配布物の仕様だけを扱う。公開の作業手順（誰がいつ何を実行するか）が spec 側に混ざっていたら指摘する。手順の正本は `doc/guidelines/release-guidelines.md` であり、`run-release` skill、spec、decision log に手順を複製していたら指摘する。
- `.github/workflows/release.yml` は `dist` が `dist-workspace.toml` と `.github/build-setup.yml` から生成する。`release.yml` だけを手で直した変更は指摘し、設定を直して `dist generate` で作り直すよう求める。生成物の `THIRD-PARTY-LICENSES.md` は commit しない。
- `doc/design/decision-log/*.md` は 1 テーマ 1 ファイル。背景・候補・検討内容・決定・理由・影響・見直し条件のうち判断に必要なものが欠けていれば指摘する。`index.md` は入口であり、詳細議論の詰め込み、参照漏れ、有効な方針と未決事項の混同を指摘する。
- `progress.md` は横断的な作業状況の一覧である。Issue 本文や skill 手順の複製、内容が混ざって読めなくなる変更を指摘する。「リリース履歴」には公開後確認を終えた version の行だけを置く。公開していない version の行や架空の行を足す変更は指摘する。
- 文末は読者層で分かれる。利用者向け（repo root `README.md`）はですます調、開発者向け（`doc/` 配下、`AGENTS.md`、`progress.md`、`working-branch-notes/`、各ディレクトリの `README.md`）は常体。混在は `[nits]` で指摘する。

## Working Branch Notes のレビュー観点

`working-branch-notes/**/*.md` は作業メモであり最終仕様書ではない。note 内の細かな整合性（stale 表現、実装との 1:1 整合）は指摘しないか `[fyi]` に留める。秘密情報・個人情報の混入は必ず `[must]` で指摘する。
