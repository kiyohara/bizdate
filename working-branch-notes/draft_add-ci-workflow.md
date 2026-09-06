# 作業ブランチメモ

- ブランチ: `add-ci-workflow`
- PR: 未作成
- 最終更新: 2026-09-06

## 目的

Issue #8 に従い、`.github/workflows/ci.yml` を追加する。以降のすべての実装 PR が `fmt` / `clippy` / `test` / `build` の検査を受けられる状態にする。

0010 で「CI は GitHub Actions とし、少なくとも `fmt` / `clippy` / `test` を実行する」と決めており、Cargo プロジェクトが置かれた（PR #19）ことで導入条件を満たした。CI を実装ロジックより先に入れるのは、後ろへ倒すと複数の PR が無検査で merge されるためである。

## 現在の状況

workflow を書き、ローカル（Compose 経由）で 4 コマンドが通ることと、actionlint が無指摘であることを確認した。CI 上での実際の緑・赤の確認はこれから行う。

## 決定事項

### clippy の警告を失敗として扱う

`cargo clippy --locked --all-targets -- -D warnings` とし、警告を失敗にする。`development-command-guidelines.md` がローカルの基本形として同じ形を挙げており、ローカルで通る状態が CI でも通る関係を保つ。警告を許すと、警告付きの変更が積み上がって後から一括で直すことになる。

### 第三者 action は commit SHA で固定する

`actions/checkout` / `dtolnay/rust-toolchain` / `Swatinem/rust-cache` を tag ではなく commit SHA で参照し、末尾コメントに固定時点の tag を書く。tag は上流で付け替えられるため、tag 参照だと供給元の変更が無警告で `main` の workflow に入る。リポジトリは public であり（0010）、workflow は `main` へ merge される。

更新は Dependabot などを入れるまで手動になる。本 Issue のスコープ外とし、必要になった時点で判断する。

### toolchain は MSRV に固定する

`toolchain: "1.98"` とし、`Cargo.toml` の `rust-version` と `Dockerfile` の `FROM` に揃える（0014）。MSRV は 3 箇所に現れるという `development-command-guidelines.md` の記述に対し、CI が 4 箇所目になる。

### cargo の option

CI では `--locked` を付ける（`development-command-guidelines.md`）。`cargo fmt` は `Cargo.lock` を解決しないため `--locked` を受け付けず、ここだけ付けない。

### Issue に明示が無いが入れたもの

- `permissions: contents: read`。検査のみの workflow であり書き込み権限を必要としない。
- `concurrency` による PR 上の古い run の打ち切り。`main` への push は履歴として残すため `cancel-in-progress` を PR に限定した。
- `env: CARGO_TERM_COLOR: always`。log を読みやすくする。

## 次にやること

- PR を作り、CI が緑になることを確認する。
- fmt 違反を入れた commit で CI が赤になることを確認し、戻す。
- 実行時間を記録する。

## 検証

未実施の項目を含む。実施済みは次のとおり。

| 項目 | 結果 |
|---|---|
| `docker compose run --rm dev cargo fmt --check` | 成功（差分なし） |
| `docker compose run --rm dev cargo clippy --locked --all-targets -- -D warnings` | 成功（警告なし） |
| `docker compose run --rm dev cargo test --locked` | 成功（0 tests） |
| `docker compose run --rm dev cargo build --locked` | 成功 |
| actionlint（`rhysd/actionlint` image） | 無指摘 |
| YAML として parse できる | 確認 |

未実施。

- CI 上で緑になること。
- fmt 違反で赤になること。
- 実行時間の記録。
- `list_workflow_runs` で run が記録されていること。

## リスク・ブロッカー

- 初回 run は cache が空のため、以降より遅くなる。実行時間の判断材料としては 2 回目以降の値も見る。
- action を SHA 固定したため、上流の修正は自動で入らない。更新手段は別途決める必要がある。

## セッションログ

- 2026-09-06: ブランチ作成。Issue #8 の依存 #7 は PR #19 が merge 済みであることを確認した。
- 2026-09-06: workflow を作成。action の SHA は GitHub API で tag から解決した。ローカル検証と actionlint を通した。
