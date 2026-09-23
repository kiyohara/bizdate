<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# release workflow の構成と公開条件

- 状態: decided
- 作成日: 2026-09-23
- 最終更新日: 2026-09-23
- 関連: `doc/design/distribution.md`, `doc/guidelines/development-command-guidelines.md`, `dist-workspace.toml`, `.github/workflows/release.yml`, [Issue #38](https://github.com/kiyohara/bizdate/issues/38)

## 背景

Issue #38 で、`dist` 0.32.0 による配布成果物と GitHub Releases 向けの release workflow を導入する。配布物の仕様（対象 4 target、`.tar.gz` と `sha256`、同梱物、version と tag、公開 trigger、権限、失敗時の扱い）は [0016](0016-distribution-contract.md) と `doc/design/distribution.md` で決まっている。

決まっていないのは実装の置き方である。dist の設定の置き場所、PR での検証範囲、公開条件（tag と version の一致、対象 commit のテスト成功、全成果物の完成）の実装方法、第三者 action の固定、ローカルでの実行環境を決める必要がある。

release workflow（`.github/workflows/release.yml`）は dist が設定から生成する。dist の `plan` / `build` / `host` は実行のたびに生成結果との差分を検査し（`dist generate --check` 相当）、差分があれば止まる。手編集はできず、調整できるのは dist の設定項目と custom job の差し込み口に限られる。

## 候補

- 検証 job の差し込み位置: A `plan-jobs` / B `local-artifacts-jobs` と `global-artifacts-jobs`
- third-party 表記の生成場所: C 事前の job で 1 回生成して build job へ渡す / D 各 build job で生成する
- 対象 commit のテスト成功の確かめ方: E release workflow から CI（`ci.yml`）を呼ぶ / F CI の結果を API で確かめる
- PR での検証範囲: G `pr-run-mode = "plan"`（dist の既定）/ H `pr-run-mode = "upload"`
- dist の build profile: I `dist init` の既定（`release` + thin LTO）/ J `release` と同じ
- build の toolchain: K runner の stable（dist の既定）/ L CI と同じ toolchain を build 前に選ぶ
- ローカルの実行環境: M dev image に dist と cargo-about を入れる / N 別の Dockerfile stage と Compose service に分ける

## 検討内容

### custom job と host の条件（A / B）

dist 0.32.0 の release workflow テンプレートを読んだ。`plan-jobs` は `plan` と並行して走り、`build-local-artifacts` がその完了を待つ。GitHub Release を作る `host` の条件は、`always()` のうえで build 系と `local-artifacts-jobs` / `global-artifacts-jobs` の結果が success か skipped であることを確かめる。`plan-jobs` の結果は条件に入っていない。

`plan-jobs` が失敗すると、それを待つ `build-local-artifacts` 以降は skipped になる。skipped を許す `host` の条件は成立する。`dist host` は GitHub hosting では成果物の有無を確かめないため、tag push では archive の無い Release が作られうる。A は公開条件として使えない。

`local-artifacts-jobs` は `build-local-artifacts` と並行して走り、`global-artifacts-jobs` は `build-local-artifacts` の後に走る。どちらの結果も `host` の条件に入り、失敗すれば `host` は走らない。

`host` は `global-artifacts-jobs` の skipped も許す。tag push で `plan` が成功した場合に `global-artifacts-jobs` が skipped になるのは、build の matrix が空で `build-local-artifacts` が skipped になるときだけである。4 target を組む設定では起こらない。

### third-party 表記の生成場所（C / D）

`include` に書いたファイルは `dist build` の時点で存在する必要がある。C は生成 job を `build-local-artifacts` より前に置くことになり、A の穴に当たる。

D は build job ごとに cargo-about を入れる。cargo-about 0.9.2 の prebuilt は `x86_64-apple-darwin` を持たないため、runner をまたいで同じ手順にするなら `cargo install` になる。Compose の dev container（14 CPU）では 35 秒で入った。生成は `targets` に 4 target を書いた設定を使うため、どの runner で生成しても 4 target を合わせた集合になる。

### 対象 commit のテスト成功（E / F）

F は、tag push の時点で main の CI が終わっていない場合の待ち合わせと、tag を打った commit が main に無い場合の扱いを自前で書く必要がある。その処理は PR では走らないため、公開の日まで検証されない。

E は `ci.yml` を reusable workflow として呼ぶだけで、公開する commit で同じ検査を通したことを同じ run の中で示せる。代わりに PR では CI が 2 回走る（直接の `CI` と、Release workflow の `custom-ci`）。

### PR での検証範囲（G / H）

G は `dist plan` だけを走らせる。build job への差し込み（toolchain、third-party 表記）と archive の中身は、tag push まで一度も検証されない。H は PR ごとに 4 target の archive を作り、`host` 以外の工程を通す。PR の検証時間は増える。

### build profile（I / J）

CI の `platform` job は `--release` で CLI E2E（`fetch-holidays` の loopback 取得を含む）を通す。I は配る binary の最適化がそれと異なり、E2E を通した構成を配ったことにならない。J なら E2E を通した profile と同じ構成の binary を配る。

### toolchain（K / L）

dist は build の toolchain を指定しない。`rust-toolchain-version` の設定は非推奨である。K では runner に入っている stable が使われ、CI が test した toolchain と異なる binary を配る。最低 glibc などの実行時前提も変わりうる。

`rust-toolchain.toml` は Compose と CI を含むすべての環境の toolchain 解決に効き、Compose の image と CI の指定（`1.98`）と別に管理する値が増える。L は build の直前の step で `rustup default` を切り替えるだけで済む。

### ローカルの実行環境（M / N）

`dist generate` は dist の設定を変えるたびに要り、ローカルで実行できる必要がある。cloud session の SessionStart hook は dev image を同期的に build する（`doc/guidelines/cloud-session-guidelines.md`）。M では dist と cargo-about の取得が session 開始に乗り、取得先への到達可否が session の起動を左右する。N は Dockerfile に stage を足し、profile を付けた Compose service から使う。hook は dev だけを build するため影響しない。

### その他の判断材料

- `dist build` は `cargo build` に `--locked` を付けない。
- dist は version が一致すれば、`v` の無い tag（`0.1.0`）や package 名付きの tag（`bizdate-v0.1.0`）も受け付ける。version が一致しない tag（`v0.2.0`）は `dist plan` が `This workspace doesn't have anything for dist to Release!` で止める。
- dist は既定で `source.tar.gz` と、全 archive の checksum をまとめた `sha256.sum` を作る。`host` は `dist-manifest.json` とともにこれらを Release へ添付する。`sha256.sum` は checksum を有効にしている限り外せない。
- dist が生成する job は、workflow 全体の `contents: write` を継ぎ、build job にも `GH_TOKEN` が渡る。dist の設定で変えられるのは custom job の権限だけである。
- 第三者 action の SHA は `github-action-commits` で指定できる。値は `uses:` の行末へそのまま出るため、`"<SHA> # <tag>"` と書けば他の workflow と同じ末尾 tag コメントになる。`github-build-setup` の値は引用符付きで出るため、tag コメントを含められない。

## 決定

- dist の設定は `dist-workspace.toml`（`dist init` の既定の置き場所）に置く。`publish = false` を上書きする `dist = true` だけを `Cargo.toml` の `[package.metadata.dist]` に置く。
- 公開を止める検査は `host` が結果を確かめる段階に置き、`plan-jobs` は使わない（B）。
  - `local-artifacts-jobs`: `ci.yml` を呼び、公開する commit で CI を通す（E）。
  - `global-artifacts-jobs`: `release-verify.yml`。PR 以外の run では tag が `v<Cargo.toml の version>` であることを確かめ（tag でない ref では失敗させる）、4 target の native runner で archive を展開して検証する。検証は checksum、archive 名と構成（dist の plan と照合）、`README.md` / `LICENSE` の同一性、third-party 表記と依存の対応、binary の起動と実行時前提（`platform-check.sh`）である。
- third-party 表記は各 build job で `dist build` の直前に生成し、配布対象の依存と対応していることを確かめる（D）。cargo-about は `cargo install --locked` で入れる。
- PR では `pr-run-mode = "upload"` とし、`host` 以外の工程を通す（H）。
- `[profile.dist]` は `release` を継ぐだけにする（J）。
- build の直前に toolchain を CI の `RUST_TOOLCHAIN` と同じ値にし（L）、`cargo fetch --locked` で `Cargo.lock` の一致を確かめる。
- `cache-builds = false` とし、公開する build を cache に依存させない。`source-tarball = false` とし、配る成果物を archive と checksum に限る。
- release workflow の第三者 action は `github-action-commits` で SHA に固定する。すべての workflow の `uses:` が commit SHA を指すことを、CI の `lint` job で確かめる。
- ローカルの実行環境は Dockerfile の `release-tools` stage と Compose の `release-tools` service とする（N）。dist は公開 binary を checksum と照合して入れ、cargo-about は build job と同じ script で入れる。
- third-party 表記の生成設定は `about.toml`（許可するライセンス、対象 4 target、依存の範囲）と `about.hbs`（書式）に置く。dev-dependencies は除き、build 依存と proc-macro は残す。bizdate 自身は含めない。

## 理由

公開を止める検査を `plan-jobs` に置くと、失敗が `host` の条件を素通りして archive の無い Release を作りうる。dist のテンプレートでこれを防げるのは、`host` が結果を見る段階だけである。

third-party 表記を事前の job で作ると同じ穴に当たるため、生成を build job 自身に置いた。build job の失敗は `host` を止める。

CI を release workflow から呼ぶのは、公開する commit のテスト成功を、別 workflow の結果の待ち合わせなしに同じ run で示せるためである。PR での二重実行は、公開経路を PR ごとに検証できることと引き換えに受け入れる。

PR で archive まで作るのは、公開の日に初めて build job の差し込みと archive の中身が動く状態を避けるためである。

profile と toolchain を CI に揃えるのは、配る binary を CI が E2E を通した構成に近づけるためである。archive の binary 自体も `release-verify` で起動して確かめる。

## 影響

- `doc/guidelines/development-command-guidelines.md` に「配布成果物の生成と確認」を加え、MSRV の表に `.github/build-setup.yml` を加える。
- Dependabot が `release.yml` の action を更新した PR では、Release workflow の `plan` が生成結果との差分で失敗する。`dist-workspace.toml` の `github-action-commits` を揃える運用を [0015](0015-dependabot-updates.md) と `doc/guidelines/development-loop.md` に置く。
- PR ごとの CI 時間が増える。Release workflow は 4 target の build と検証、CI の再実行を含む。
- dist が生成する job の `contents: write` と、build job に渡る `GH_TOKEN` は dist の制約として残る。追加した custom job は `contents: read` に絞った。
- D を選んだため、`GH_TOKEN` を持つ build job で cargo-about 0.9.2 を `cargo install` して実行する。その依存の build script と proc-macro も同じ job で動き、信頼する範囲は、配る binary の依存に cargo-about の依存木を加えたものになる。version の固定と `--locked` で再現性は保つ。
- dist が生成する `release.yml` は、`github.ref_name`（tag 名）を `run:` の shell へ直接展開する（`plan` の `dist host --steps=create --tag=...`、build と host の `--tag`、`gh release create`）。tag 名には `$(...)` や `;` を含められ、trigger の tag pattern もそれを通す。このため tag を push できる者は、`plan` 以降の job でコマンドを実行できる。`release-verify` の tag 検査は build の後に走るため、これを防がない。tag の push には contents の write 権限が要り、権限の拡大にはならない。生成物は手で直せないため、dist の制約として受け入れる。
- `Dockerfile` と `compose.yaml` を変えたため、cloud session の environment cache は stub の貼り直しが要る。`compose.cloud.yaml` の override は `dev` にだけ置いており、cloud session では `release-tools` を使えない。`release-tools` を要する作業（`dist generate` など）はローカルで行う。
- Homebrew の installer と tap の更新（Issue #39）は、この構成の `host` の後に dist の publish job として加わる。

## 後から見直す条件

- dist が `plan-jobs` の結果を `host` の条件に含めるようになった場合。C へ戻し、third-party 表記の生成を 1 回にできる。
- dist が job ごとの権限を設定できるようになった場合、または tag 名を `run:` へ直接展開しなくなった場合。build job の `contents: write` を外し、残るリスクを読み直す。
- dist の trigger 系の設定（`dispatch-releases` / `release-branch`）を変える場合。tag の push 以外で公開が走るため、`host` の条件、`release-verify` の tag 検査、`global-artifacts-jobs` の skipped を許す前提を読み直す。
- cloud session で `release-tools` が必要になった場合。`compose.cloud.yaml` に `dev` と同じ override を足し、cloud session で確かめる。
- dist の version を上げる場合。`release.yml` の差分に加え、custom job の `needs` と `host` の条件を読み直す。
- PR ごとの CI 時間やレビュー待ちが問題になった場合。`pr-run-mode` と CI の二重実行を見直す。
- cargo-about が `x86_64-apple-darwin` の prebuilt を出した場合。build job での install の時間を見直す。
- MSRV を上げる場合。`.github/build-setup.yml` の toolchain も同時に上げる。
