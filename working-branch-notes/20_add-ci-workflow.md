# 作業ブランチメモ

- ブランチ: `add-ci-workflow`
- PR: #20
- 最終更新: 2026-09-06

## 目的

Issue #8 に従い、`.github/workflows/ci.yml` を追加する。以降のすべての実装 PR が `fmt` / `clippy` / `test` / `build` の検査を受けられる状態にする。

0010 で「CI は GitHub Actions とし、少なくとも `fmt` / `clippy` / `test` を実行する」と決めており、Cargo プロジェクトが置かれた（PR #19）ことで導入条件を満たした。CI を実装ロジックより先に入れるのは、後ろへ倒すと複数の PR が無検査で merge されるためである。

## 現在の状況

Issue #8 の検証項目をすべて実施し、PR #20 を作成した。レビュー待ち。

CI が緑になること、意図的な fmt 違反で赤になることの両方を実際の run で確認した。

## 決定事項

### clippy の警告を失敗として扱う

`cargo clippy --locked --all-targets -- -D warnings` とし、警告を失敗にする。`development-command-guidelines.md` がローカルの基本形として同じ形を挙げており、ローカルで通る状態が CI でも通る関係を保つ。警告を許すと、警告付きの変更が積み上がって後から一括で直すことになる。

### 第三者 action は commit SHA で固定する

`actions/checkout` / `dtolnay/rust-toolchain` / `Swatinem/rust-cache` を tag ではなく commit SHA で参照し、末尾コメントに固定時点の tag を書く。tag は上流で付け替えられるため、tag 参照だと供給元の変更が無警告で `main` の workflow に入る。リポジトリは public であり（0010）、workflow は `main` へ merge される。

更新は Dependabot などを入れるまで手動になる。本 Issue のスコープ外とし、必要になった時点で判断する。

### toolchain は MSRV に固定する

`toolchain: "1.98"` とし、`Cargo.toml` の `rust-version` と `Dockerfile` の `FROM` に揃える（0014）。CI が MSRV の 4 箇所目になるため、review 対応で `development-command-guidelines.md` の表と「やらないこと」を 4 箇所へ更新した。

### cargo の option

CI では `--locked` を付ける（`development-command-guidelines.md`）。`cargo fmt` は `Cargo.lock` を解決しないため `--locked` を受け付けず、ここだけ付けない。

### Issue に明示が無いが入れたもの

- `permissions: contents: read`。検査のみの workflow であり書き込み権限を必要としない。
- `concurrency` による PR 上の古い run の打ち切り。`main` への push は履歴として残すため `cancel-in-progress` を PR に限定した。
- `env: CARGO_TERM_COLOR: always`。log を読みやすくする。
- `timeout-minutes: 15`（review 対応で追加）。未指定だと GitHub-hosted runner の上限 6 時間まで走る。#13 の HTTP 取得が入った後に応答の返らない処理が紛れ込むと、赤になるまで runner を占有する。

## 次にやること

- レビュー対応。merge はユーザーが行う。
- merge 後、branch protection の required status check を有効にするかはユーザーが判断する（Issue のスコープ外）。

## 検証

Issue #8 の検証項目をすべて実施した。

| 項目 | 結果 |
|---|---|
| PR 自体で CI が緑になる | 成功。run #1（cold cache）と run #3（warm cache）の 2 回 |
| fmt 違反を入れた commit で CI が赤になる | 成功。run #2 が `cargo fmt` で失敗し、後続の clippy / test / build は skip された |
| 違反 commit を戻す | revert commit で戻し、run #3 が緑になることを確認 |
| workflow の実行時間 | cold cache 約 33 秒、warm cache 約 17 秒 |
| run が記録されている | `list_workflow_runs` で run #1〜#3 を確認 |

run ごとの内訳。

| run | commit | 結果 | job 時間 |
|---|---|---|---|
| #1 | workflow 追加 | success | 33 秒（cache 復元なし） |
| #2 | fmt 違反（意図的） | failure | 18 秒（`cargo fmt` で停止） |
| #3 | #2 の revert | success | 17 秒 |

step 別の所要時間（run #1 → run #3）。

| step | run #1 | run #3 |
|---|---|---|
| Install Rust toolchain | 8 秒 | 9 秒 |
| Cache restore | 0 秒 | 3 秒 |
| cargo fmt | 1 秒 | 0 秒 |
| cargo clippy | 8 秒 | 1 秒 |
| cargo test | 8 秒 | 0 秒 |
| cargo build | 0 秒 | 0 秒 |

toolchain の導入が約 9 秒で、warm cache 時は実行時間の過半を占める。現状の実行時間は待てる範囲にある。

事前のローカル確認（Compose 経由）。

| 項目 | 結果 |
|---|---|
| `docker compose run --rm dev cargo fmt --check` | 成功（差分なし） |
| `docker compose run --rm dev cargo clippy --locked --all-targets -- -D warnings` | 成功（警告なし） |
| `docker compose run --rm dev cargo test --locked` | 成功（0 tests） |
| `docker compose run --rm dev cargo build --locked` | 成功 |
| actionlint（`rhysd/actionlint` image） | 無指摘 |
| YAML として parse できる | 確認 |

## リスク・ブロッカー

- action を SHA 固定したため、上流の修正は自動で入らない。更新手段（Dependabot など）は別途決める必要がある。
- decision log 0014 の「影響」は MSRV を 3 箇所と書いているが、当時の記録として残す（`decision-log-guidelines.md` は本文を書き換えない方針）。運用上の正本は `development-command-guidelines.md` 側であり、そちらは 4 箇所へ更新済み。
- 実装が最小のため、`-D warnings` の実際の負荷はまだ測れていない。以降の実装 PR で判断材料が出る。

## セッションログ

- 2026-09-06: ブランチ作成。Issue #8 の依存 #7 は PR #19 が merge 済みであることを確認した。
- 2026-09-06: workflow を作成。action の SHA は GitHub API で tag から解決した。ローカル検証と actionlint を通した。
- 2026-09-06: `.github/workflows/` への push が token の scope 不足で拒否された。`workflow` scope の付与はユーザーが実施。
- 2026-09-06: PR #20 を作成。run #1 が緑。fmt 違反を入れた run #2 が赤になることを確認し、revert して run #3 が緑に戻ることを確認した。
- 2026-09-06: review comment 5 件へ対応。`[must]` の MSRV 表を 4 箇所へ更新し、`[imo]` の `timeout-minutes` 追加と `# v1` の日付追記を採用した。Dependabot は別 Issue、`-D warnings` は現状維持とした。
