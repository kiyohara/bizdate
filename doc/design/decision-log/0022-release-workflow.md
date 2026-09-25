<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# release workflow の構成と公開条件

- 状態: decided
- 作成日: 2026-09-23
- 最終更新日: 2026-09-25
- 関連: `doc/design/distribution.md`, `doc/guidelines/development-command-guidelines.md`, `doc/guidelines/release-guidelines.md`, `dist-workspace.toml`, `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `.github/workflows/release-verify.yml`, `.github/scripts/install-cargo-about.sh`, [Issue #38](https://github.com/kiyohara/bizdate/issues/38), [Issue #65](https://github.com/kiyohara/bizdate/issues/65), [Issue #86](https://github.com/kiyohara/bizdate/issues/86)

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

## 2026-09-24 追記: cargo-about を prebuilt で入れ、PR での CI の二重実行をやめる

Issue #65。上の「後から見直す条件」の「PR ごとの CI 時間やレビュー待ちが問題になった場合」に当たる。あわせて、候補 D で cargo-about を `cargo install` で入れた前提（`x86_64-apple-darwin` の prebuilt が無い）は、#64 で配布対象から `x86_64-apple-darwin` を外したことで失われた（[0016](0016-distribution-contract.md) の 2026-09-23 追記）。

### 候補

上の候補（A〜N）と区別するため、O から続ける。

- cargo-about の導入方法: O `cargo install` を続ける / P upstream の prebuilt を、repository に固定した sha256 と照合して入れる / Q prebuilt を、同じ release の `.sha256` と照合して入れる / R `cargo install` の結果を cache する
- PR での CI の二重実行: S 維持する / T PR の run では、`custom-ci` から `platform` を省く / U PR の run では、`custom-ci` の全 job を省く

### 検討内容

#64 の PR（#67）の初回 push（head `ee5d285`、[run 35873618982](https://github.com/kiyohara/bizdate/actions/runs/35873618982)）で、Release workflow は 3 分 37 秒だった。3 つの build job（2 分 17 秒〜2 分 48 秒）のうち 93〜109 秒を、cargo-about の `cargo install` が占めた。

残る 3 target の build runner には、cargo-about 0.9.2 の upstream の prebuilt がすべてある。`macos-15` は `aarch64-apple-darwin`、`ubuntu-22.04` は `x86_64-unknown-linux-musl`、`ubuntu-22.04-arm` は `aarch64-unknown-linux-musl` を使う。Linux の 2 つは musl の静的 binary で、runner の glibc に依存しない。Compose の `release-tools`（Linux の x86_64 / arm64）も同じ asset で賄える。

P の prebuilt と、ソースからビルドした同じ version（O）は、同じ `THIRD-PARTY-LICENSES.md` を生成する。Compose の `release-tools`（Linux arm64）で両方を実行し、出力が byte 単位で一致することを確かめた。

Q の `.sha256` は asset と同じ release に置かれ、asset と一緒に差し替えられうる。改竄を検知するには、repository に固定した値と照合する（P）必要がある。`Dockerfile` が dist を入れる方式と同じである。

R の cache は、公開する tag の run で効く保証が無く、上の「決定」の「公開する build を cache に依存させない」方針とも合わない。P は cache を使わずに、tag の run も同じだけ短くする。

信頼する範囲も変わる。O では、`GH_TOKEN` を持つ build job で cargo-about の依存木をビルドし、その build script と proc-macro を実行していた。P ではそれらを実行せず、upstream が release に添付した binary 1 つを実行する。信頼の対象は、crates.io 上の依存木のソースから、upstream の release の成果物へ移る。sha256 を固定するため、固定した時点の binary 以外は実行しない。

PR の run では、直接の CI と Release workflow の `custom-ci` が、同じ commit で `lint` と `platform` を 2 回ずつ回す（上の E で受け入れた二重実行）。push 1 回あたりの macOS job 4 本のうち 2 本は `platform` である。一方 tag push では、`custom-ci` が公開する commit の CI を示す唯一の経路であり、省けない。

`custom-ci` の結果は、`build-global-artifacts` と `custom-release-verify` の `needs` に入る。`custom-ci` が skipped になると両者も skipped になり、PR で archive を検証しなくなる。dist が生成する `custom-ci` の条件と `needs` は手で変えられないため、呼ばれる側の `ci.yml` で job を選ぶ。U では `custom-ci` の job がすべて skipped になり、`custom-ci` が success で終わることを当てにできない。T は `lint`（1 分未満）を残すため、`custom-ci` は success で終わる。

呼ばれた workflow の `github` context は呼び出し側のものであり、`github.event_name` は PR の run で `pull_request`、tag push で `push` になる。dist は custom job へ `plan` を渡すため、`plan` の有無で直接の CI と区別できる。

### 決定

- cargo-about は upstream の prebuilt を入れる（P）。version は 0.9.2 のままとし、`.github/scripts/install-cargo-about.sh` を正とする。実行環境の OS / architecture から asset を選び、script に固定した sha256 と照合する。対応外の platform と sha256 の不一致では止める（fail-closed）。`Dockerfile` の `release-tools` stage も同じ script を `--root` 付きで使う。上の「決定」の「cargo-about は `cargo install --locked` で入れる」を、この追記で置き換える。
- PR の run では、`custom-ci` から `platform` を省き、`lint` だけを回す（T）。tag push では `ci.yml` の全 job を回す。`ci.yml` の `platform` の条件は、省く場合（呼び出し側の event が `pull_request` で、`plan` が渡されている）だけを列挙する形とし、それ以外の run では必ず回す。上の「理由」の「PR での二重実行は、公開経路を PR ごとに検証できることと引き換えに受け入れる」を、この追記で置き換える。
- cache は使わない（R は採らない）。`cache-builds = false` と `pr-run-mode = "upload"` は維持する。

### 理由

- prebuilt にすれば、3 つの build job から cargo-about のビルド（93〜109 秒）が消える。cache を使わないため、公開する tag の run も同じだけ短くなり、公開する build を cache に依存させない方針と両立する。
- sha256 を repository に固定すれば、固定した後に asset が差し替えられても、実行する前に止まる。
- PR で `custom-ci` の `platform` が回す検査は、直接の CI の `platform` と同じ commit の同じ検査である。tag push では、公開する commit の CI を同じ run で示すという E の理由がそのまま残るため、全 job を回す。
- 条件を、省く場合だけを列挙する形にするのは、tag push や後から足す trigger で `platform` が黙って省かれる構成を避けるためである。

### 影響

- 上の「影響」のうち、build job で cargo-about を `cargo install` し、その依存の build script と proc-macro が動くという記述は、upstream の prebuilt を固定した sha256 と照合して実行する形に変わる。
- cargo-about の version を上げるときは、3 つの asset の sha256 を取り直す（`doc/guidelines/development-command-guidelines.md` の「tool と action の version を上げるとき」）。upstream が build runner に合う asset を配らなくなると、その runner の build job は止まる。
- PR の run では、`custom-ci` の `platform`（macOS 1 本を含む 3 job）が skipped になる。push 1 回あたりの macOS job は 3 本（CI 1 本、Release workflow 2 本）になる。
- PR #76 の初回 push（head `dbcf7b9`、[run 35936143855](https://github.com/kiyohara/bizdate/actions/runs/35936143855)）で、Release workflow は 1 分 44 秒になった（上の #67 の 3 分 37 秒から）。3 つの build job は 48〜58 秒で、cargo-about の step は 1 秒以下だった。
- `doc/design/distribution.md`、`doc/guidelines/development-command-guidelines.md`、`.github/build-setup.yml` と `.github/workflows/ci.yml` のコメント、`.github/copilot-instructions.md` を改めた。
- tag push の経路は PR では走らない。tag push で `platform` が回ることは条件式の review で確かめた。`platform` が省かれても `custom-ci` は `lint` だけで success になり、`host` は公開へ進むため、失敗として表に出ない。実動は、初回公開の tag push の run で `custom-ci / test / build (<target>)` の 3 job が skipped でなく success で走ったことで確かめる。この確認は #40 の公開後チェックへ引き継いだ。
- 上の「後から見直す条件」のうち、「cargo-about が `x86_64-apple-darwin` の prebuilt を出した場合」は、#64 で対象から外したため適用しない。「PR ごとの CI 時間やレビュー待ちが問題になった場合」は、CI の二重実行をこの追記で見直した。`pr-run-mode` は維持した。

### 後から見直す条件

- cargo-about の version を上げる場合、または build runner を変える場合。runner に合う prebuilt があるかを確かめ、無ければ導入方法を見直す。
- dist の version を上げる場合（上の条件）。custom job へ `plan` を渡す形が変わっていないかも確かめる。`platform` を省く条件がこれを前提にしている。

## 2026-09-25 追記: checksum file の末尾の空行を取り除いてから公開する

Issue #86。`v0.1.0` の公開後確認（#84）で、macOS の `shasum -a 256 -c` が `sha256.sum` と各 `.sha256` に対して `WARNING: 1 line is improperly formatted` を出した。照合はすべて `OK` で、exit 0 である。README は、この警告が出ることと、`--strict` を付けると失敗することを注記している。

dist 0.32.0 の `write_checksum_file`（crates.io の `cargo-dist-0.32.0` の `src/lib.rs:714-738`）は、各行を `writeln!` で書いた後に `contents.push('\n')` で改行をもう 1 つ足す。archive ごとの `.sha256` と、全 archive をまとめた `sha256.sum` の両方がこの関数を通るため、どちらも末尾に空行を 1 行含む。

### 候補

上の候補（A〜U）と区別するため、V から続ける。

- V: dist の version を上げ、直った版を使う
- W: `release-verify` の中で `build-global-artifacts` の artifact を待ち、空行を取り除いた checksum file で build の workflow artifact を置き換えてから `host` へ渡す
- X: `host` の後の publish job で、Release の checksum file の asset を差し替える
- Y: `github-release = "announce"` にして Release の作成を publish job の後へ移し、その前に置き換える
- Z: 直さずに上流の修正を待ち、README の注記を残す

### 検討内容

2026-09-25 の時点で、crates.io の cargo-dist の最新は 0.32.0 である。上流の main の `write_checksum_file` も改行を足したままであり、V は採れない。

dist 0.32.0 の release workflow テンプレートには、`sha256.sum` ができた後、`host` の前に、`host` と競合せずに走る custom job の差し込み口が無い。

- `global-artifacts-jobs`（`release-verify`）は `build-global-artifacts` と同じ `needs` を持ち、並行して走る。`sha256.sum` はまだ無く、`needs` で待てない。
- `host-jobs` は `build-global-artifacts` の後に走るが、`host` と並行する。`host` が Release に上げる artifact を取得している間に置き換えると競合する。
- `extra-artifacts` の build は、global の build の中で unified checksum の生成より前に走る。
- `github-build-setup` は build の前にだけ差し込める。

W では、`release-verify` の job が、この run の artifact の一覧を API で読んで `artifacts-build-global` を待つ。`host` は `release-verify` の完了を待つため、置き換えは `host` より前に終わる。置き換えには `actions/upload-artifact` の `overwrite` を使い、同じ名前の artifact を消してから上げ直す。上げ直しに失敗すれば `release-verify` が失敗し、`host` は走らない。一覧を読むために、`release-verify` に `actions: read` が要る。

置き換えている間に同じ artifact を取得する job があると競合する。`release-verify` の中で build の artifact を取得する job（`archive`、`homebrew-formula`、`homebrew`）の後に置けばよい。`build-global-artifacts` の取得は、それが artifact を上げた時点で終わっている。PR の run（`pr-run-mode = "upload"`）も `host` の直前まで同じ経路を通るため、置き換えと検査を PR ごとに確かめられる。

prerelease では `homebrew-formula` と `homebrew` が skipped になる。`if` の無い job は暗黙の `success()` で評価され、依存の連鎖の上流に skipped があると、間の job が success でも skipped になる（GitHub Actions の `needs` の仕様）。reusable workflow の中の skipped は `host` の条件で成功扱いになるため、置き換えと検査が skipped のまま `host` が公開する。待ち合わせの job だけでなく、置き換えと検査の job にも状態関数を含む条件を置く必要がある。PR の run は prerelease ではないため、この経路は `homebrew-formula` の条件を一時的に反転した PR の run で確かめた（PR #89）。

re-run では、前の attempt の artifact が一覧に残る。同じ名前の artifact が複数あると、`actions/download-artifact` の取得と、`actions/upload-artifact` の `overwrite` の削除は、ID が最大のものを選ぶ（`@actions/artifact` の実装）。ID は作成順とは限らない。PR #89 の run でも、後から作った artifact の ID が、先に作った artifact の ID より小さいことがあった。このため、置き換えがこの attempt の artifact を選ぶとは限らない。

待ち合わせは、この attempt の `artifacts-build-global` が上がったことを確かめるためにある。上がった後は、`host` の取得までに `artifacts-*` を上げる job が無い。`checksum files` の job は `host` と同じ取り方で同じ集合を検査するため、置き換えが別の attempt の artifact を選んで整え損ねれば、`host` の前で失敗する。この attempt の artifact かどうかは、最新の `artifacts-plan-dist-manifest`（`plan` job が上げる）より後に作られたかどうかで、作成時刻で判断する。`plan` が走り直さない re-run（失敗した job だけの再実行）では、前の attempt の `plan` の artifact が基準になる。

例外は、`build-global-artifacts` が artifact を上げた後に失敗し、その job を再実行する場合である。待ち合わせは前の attempt の `artifacts-build-global` で抜けうる（`custom-release-verify` が成功していれば、走り直しもしない）ため、走り直した `build-global-artifacts` が後から上げる file は、置き換えと検査を通らずに `host` に渡りうる。この場合は再実行せず、version を上げる（`doc/guidelines/release-guidelines.md` の「復旧」）。

X では、差し替えるまでの間、空行つきの file が公開される。`host` は PR では走らないため、この経路は公開の日まで一度も動かない（上の H で避けた状態）。publish job に `contents: write` が要り、GitHub の immutable release を有効にすると差し替えられなくなる。

Y では、Release を作るのが `announce` の job になり、tap の Formula を書く publish job（[0025](0025-homebrew-formula-publishing.md)）が Release より先に走る。Formula が指す archive が存在しない間ができる。

Z では、警告と README の注記が残る。上流の修正の時期は分からない。

### 決定

- W を採る。`release-verify` に次の job を足す。
  - `wait for build artifacts`（`checksum-artifacts`）: `archive`、`homebrew-formula`、`homebrew` の後に走り、`.github/scripts/wait-for-build-artifacts.sh` で `artifacts-build-global` を待つ。checksum file を含む build の artifact（`artifacts-build-*`）の名前を出す。
  - `normalize checksums (<artifact>)`（`checksum-normalize`）: artifact ごとに、`.github/scripts/normalize-checksum-files.sh` で改行 2 つで終わる checksum file から最後の改行を 1 つだけ取り除き、同じ名前の artifact を置き換える。空行で終わらない file は変えない。
  - `checksum files`（`checksum`）: `host` と同じ形（`artifacts-*` を 1 つの dir にまとめる）で取得し、`.github/scripts/check-checksum-files.sh` で検査する。plan が挙げる checksum file がそろい、空行と CR が無く、各行が `<sha256> *<archive 名>` で plan の archive と一致し、`sha256sum -c --strict` と `shasum -a 256 -c --strict` が警告なしで通ることを確かめる。
  - 3 job はどれも状態関数を含む条件を持ち、prerelease で `homebrew-formula` と `homebrew` が skipped でも走る。`checksum-artifacts` は needs に失敗と cancel が無いとき、後の 2 job は前の job が success のときに走る。
- `dist-workspace.toml` の `github-custom-job-permissions` で、`release-verify` に `actions: read` を足す。`release-verify` の中で `actions: read` を持つのは `checksum-artifacts` の job だけにする。
- checksum の形式（`*` の binary mode 表記）と algorithm は変えない。公開済みの `v0.1.0` の asset は差し替えない。
- README の注記（`shasum` の警告と `--strict`）は、空行の無い checksum file を初めて公開する version の公開後の PR で外す。外すのは、その version の公開後確認で、公開した checksum file が `check-checksum-files.sh` を通ったことを確かめた後とする。担当はその version の公開担当者であり、公開後確認 Issue の雛形の「公開後の PR」に項目を置く。`releases/latest` が指す asset が空行の無いものに変わるまで、注記を残すためである。

### 理由

- 公開の時点で、Release の checksum file に空行が無い。公開後の差し替えが要らない。
- 置き換えと検査は PR の run でも動く。公開の日に初めて動く経路を増やさない。
- 検査は、置き換えた後の file を `host` と同じ取り方で確かめる。prerelease を含め、置き換えが漏れても、空行が戻っても、`host` の前で止まる。
- 空行の判定は tool に頼らず直接行う。`shasum` と coreutils 8.x の `sha256sum`（`ubuntu-22.04` の runner）は空行を警告するが、coreutils 9.x の `sha256sum` は空行を黙って読み飛ばす。
- 取り除くのは dist が足した最後の改行 1 つに限り、それ以外の崩れは検査で止める。dist が直った後は、何も変えずに通る。

### 影響

- `release-verify` は、検証に加えて、`host` が Release に上げる build の workflow artifact を変える。変えるのは checksum file の末尾の改行 1 つだけで、archive、Formula、manifest は同じ内容のまま上げ直す。
- `release-verify` の token に `actions: read` が加わった。使うのは、`checksum-artifacts` の job が、この run の artifact の一覧を読むときだけである。
- release workflow の job が 3 つ増える（matrix を展開すると 6 job）。`homebrew` の job の後に直列で走るため、Release workflow の所要時間が延びる。
- `build-global-artifacts` が失敗した場合、`host` は走らない。artifact を上げる前に失敗していれば、`checksum-artifacts` の job は待つ上限（900 秒）まで待ってから失敗する。上げた後に失敗していれば待ち合わせを抜けるが、その run は再実行しない（上の例外）。
- `doc/design/distribution.md` の「checksum」、`doc/guidelines/development-command-guidelines.md` の「release workflow」、`doc/guidelines/release-guidelines.md`（job の一覧、復旧、公開後確認、公開後の PR）、`run-release` の公開後確認 Issue の雛形、`.github/copilot-instructions.md` を改めた。
- Release の checksum file に空行が無いことは、tag push の run でだけ確かめられる。PR の run では、置き換えた workflow artifact を `checksum files` の job が検査する。公開した asset は、次の version の公開後確認で `check-checksum-files.sh` に通す。

### 後から見直す条件

- dist の version を上げる場合。checksum file に空行を足さなくなっていれば、`checksum-artifacts` と `checksum-normalize` の job と `actions: read` を外し、`checksum` の検査は残す。`build-global-artifacts` の後、`host` の前に走る差し込み口ができていれば、待ち合わせをやめてそこへ移す。
- `actions/upload-artifact` の `overwrite`、または `actions/download-artifact` が同じ名前の artifact から選ぶ規則が変わった場合。
- re-run で前の attempt の artifact の扱いが変わった場合。`wait-for-build-artifacts.sh` の待つ条件を読み直す。
