<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# Homebrew Formula の生成・検証と tap への書き込み

- 状態: decided
- 作成日: 2026-09-24
- 最終更新日: 2026-09-24
- 関連: `doc/design/distribution.md`, `doc/guidelines/development-command-guidelines.md`, `dist-workspace.toml`, `.github/workflows/publish-homebrew.yml`, `.github/workflows/release-verify.yml`, `.github/scripts/prepare-homebrew-formula.sh`, `.github/scripts/verify-homebrew-formula.sh`, `.github/scripts/publish-homebrew-formula.sh`, [Issue #39](https://github.com/kiyohara/bizdate/issues/39), [Issue #80](https://github.com/kiyohara/bizdate/issues/80)

## 背景

[0016](0016-distribution-contract.md) で、Homebrew の Formula を dist で生成し、既存 tap `kiyohara/homebrew-tap` の `Formula/` に置くこと、prerelease で stable の Formula を上書きしないこと、失敗時は同じ version を作り直さず version を上げることを決めた。[0022](0022-release-workflow.md) で release workflow の構成を決め、Homebrew は `host` の後に publish job として加わるとした。

Issue #39 は、これに加えて Formula の起動テスト、既存 Formula を意図せず巻き戻さない条件、失敗時の再開、公開前成果物での install の検証を求めている。

2026-09-24 に tap を read-only で確認した（HEAD `35d25c6`）。`Casks/slapex.rb` と `README.md` だけで、`Formula/` は無い。

## 候補

- tap への書き込み: A dist の builtin の publish job（`publish-jobs = ["homebrew"]`）/ B custom の publish job（`publish-jobs = ["./publish-homebrew"]`）で、dist が生成した Formula を書く
- 起動テストの置き場所: C Formula に `test do` を足す / D Formula は生成のまま、release workflow の検証だけで起動を確かめる
- 公開前の install の検証: E 行わない（Formula の構文と内容の検査だけ）/ F PR でも、この run の archive を指すよう url を差し替えて native runner で install と `brew test` を行う
- tap への認証: G `repo` scope の classic PAT（0016 の記述、dist の文書の既定）/ H `kiyohara/homebrew-tap` だけに絞った fine-grained PAT（Contents の read and write）

## 検討内容

### dist の builtin の publish job（A）

dist 0.32.0 が生成する `publish-homebrew-formula` job を `dist generate` で確かめた。tap を checkout し、全 artifact を `Formula/` へ展開して、`.rb` を `brew style --except-cops FormulaAudit/Homepage,FormulaAudit/Desc,FormulaAuditStrict --fix ... || true` で整えてから commit して push する。

- Formula の template（`installer/homebrew.rb.j2`）は `test do` を持たず、設定で足す口も無い。Issue の起動テストを満たせない。
- `brew style` の失敗を `|| true` で捨てる。
- tap にある Formula の version を見ずに上書きする。dist の文書も、非線形な release（2.0 の後に 1.0 の patch を出すなど）では最後に出したものが残ると注意している。巻き戻しを止める口が無い。
- commit の author は `axo bot` に固定される。
- job 自体は dist が生成し手で直せないため、上の点を変えられない。

### custom の publish job（B）

dist の `publish-jobs` は `./<name>` で reusable workflow を指せる。生成される呼び出し側は `host` の後に走り、prerelease では builtin と同じ条件（`announcement_is_prerelease` かつ `publish_prereleases` が false）で skipped になる。`announce` の条件にも入る。権限は `github-custom-job-permissions` で絞れる。

Formula の生成は dist の installer（`installers = ["homebrew"]`）が引き続き行う。B で変わるのは、tap へ書く前の検査と加工、書き込みの条件だけである。代わりに、dist は `tap` があるのに builtin の publish job が無効だと、実行のたびに WARN を出す。

### 起動テスト（C / D）

Homebrew の `brew test` は Formula の `test do` を実行する。D では Homebrew 経由で入れた利用者の環境で `brew test` が何もしない。C は B を選べば、tap へ書く前に足せる。テストは `bizdate --version` の出力と、third-party 表記が `pkgshare` にあることを確かめる。祝日データを要する `first` / `last` は、install 時に祝日データを取得しない既存仕様に合わせて使わない。

### 公開前の install（E / F）

tag push まで Formula で install しないと、公開の日に初めて Formula と archive の組み合わせが動く（0022 で PR でも archive を作ると決めた理由と同じ）。Homebrew は tap に無い Formula を install しないため、一時的な local tap に置き、url を `file://` でこの run の archive へ差し替える。sha256 は変えないため、Formula の checksum と archive の一致も install で確かめられる。公開 tap からの install / upgrade の確認とは区別する。

GitHub の macOS runner には Homebrew が PATH にあり、Ubuntu の runner（`ubuntu-22.04` と `ubuntu-22.04-arm`）には `/home/linuxbrew` に入っている（PATH には無い）。PR #79 の CI で 3 つの runner とも `brew` を実行できることを確かめた。

cloud session の sandbox では、Homebrew の portable Ruby の取得元（`ghcr.io`）が許可されておらず、ローカルで `brew` を動かせなかった。install の検証は CI に置くしかない。

### 認証（G / H）

tap への push に要るのは、tap の Contents の書き込みだけである。G は PAT の持ち主が書ける全リポジトリに書ける。H なら漏れたときの影響を tap に限れる。secret の名前は dist の慣例どおり `HOMEBREW_TAP_TOKEN` のままでよい。

## 決定

- tap への書き込みは custom の publish job（B）で行う。`dist-workspace.toml` は `installers = ["homebrew"]`、`tap = "kiyohara/homebrew-tap"`、`publish-jobs = ["./publish-homebrew"]` とし、job の `GITHUB_TOKEN` は `contents: read` に絞る。dist の WARN は想定どおりとして受け入れる。
- Formula の検査と `test do` の追加（C）は `.github/scripts/prepare-homebrew-formula.sh` 1 本にまとめ、`release-verify` が PR でも tag push でも通す。検査は、class 名、desc / homepage / license と `Cargo.toml` の一致、version と plan の一致（prerelease は止める）、OS / CPU の分岐が配布対象 3 target と一致すること（Intel Mac の分岐が無いこと）、各 url と sha256 が Release asset と `.sha256` に一致すること、生成物に `test do` が無いことである。
- dist の template は `brew style` の自動修正できる違反（入れ子の `if`、定数の freeze、hash の整列など）を含む（PR #79 の CI で確認）。builtin の job のように失敗を捨てず、`.github/scripts/fix-homebrew-formula-style.sh` で `brew style --fix` をかけた後、違反が残らないことを確かめる。除く cop は利用者が書く desc と homepage の 2 つだけで、builtin の job が除く `FormulaAuditStrict` は除かない。`release-verify` が PR でも tag push でも通す。
- tap へ書くのは、同じ run の `release-verify` が検査・style の修正・install・`brew test` まで通した Formula の artifact そのものとする。publish job で作り直すと、検証した内容と書く内容の一致が保証されず、job の再実行時に runner の Homebrew の version が変わると `brew style --fix` の出力が変わって「同じ version の内容違い」で止まりうるため（PR #79 の review cycle `claude-code-69f95c0-20260924050758` の指摘）。
- prerelease の tag では `release-verify` の Formula の生成と検証を job ごと skipped にする。Formula は prerelease を拒否するため、走らせると `custom-release-verify` が失敗して `host` が止まり、prerelease の GitHub Release まで出せなくなる（同 review cycle の指摘）。
- PR でも Formula で install する（F）。`release-verify` は dist の `build-global-artifacts`（Formula を作る job）と並行して走り、その artifact を待てない。このため `release-verify` の `homebrew-formula` job が、plan job が保存した同じ dist と同じ入力（全 target の archive と checksum）から Formula を作り直して検査する。`homebrew` job が、build と同じ native runner で `brew style`、local tap からの install、`brew test`、配置（bin、doc、`pkgshare` の third-party 表記）と archive の同一性を確かめる。runner に Homebrew が無ければ失敗させる。
- tap へ書く条件は `.github/scripts/publish-homebrew-formula.sh` に置く。prerelease、tap の version のほうが新しい場合（巻き戻し）、同じ version で内容が異なる場合、`Formula/bizdate.rb` 以外の変更が出る場合は書かずに失敗させる。同じ version が同じ内容で既にあれば何もせず成功させ、push 済みの後の再実行を安全にする。
- `HOMEBREW_TAP_TOKEN` は `kiyohara/homebrew-tap` だけに絞った fine-grained PAT（Contents の read and write）を推奨する（H）。classic PAT の `repo` scope でも動く。設定はユーザーが行う。
- `Cargo.toml` に `homepage` を足し、Formula の `homepage` に使う。

## 理由

- 起動テストと巻き戻しの抑止は Issue の完了条件であり、dist の builtin の job では満たせない。Formula の生成は dist に任せたまま、書き込みの前後だけを自前にすれば、dist の template の更新も引き続き受けられる。
- 検査と test の追加を 1 本の script にまとめて PR と tag push で同じものを通し、tag push では検証した artifact をそのまま書く。PR で install した Formula と tap へ書く Formula が同じ手順で作られ、tag push では検証したものと書くものが一致する。
- 失敗を fail-closed にし、同じ内容の再実行だけを成功にすれば、「同じ version を作り直さず version を上げる」（0016）を job 自身が守り、かつ publish の失敗から job の再実行で復旧できる。

## 影響

- `release.yml` に `custom-publish-homebrew` job が加わり、`announce` はその成功か skipped を待つ。PR では `host` が走らないため呼ばれない。
- PR の run の `custom-release-verify` に `homebrew` の 3 job が加わり、`host` の公開条件に入る。Homebrew の install と `brew style` の分、PR の CI 時間が延びる。
- `publish-homebrew` が失敗しても、GitHub Release は `host` で公開済みである（0016）。復旧は、原因（secret の未設定、tap の状態など）を直してからその job を再実行する。巻き戻しや同じ version の作り直しで止まった場合は、version を上げて出し直す。
- `release-tools` を使えない cloud session では、この変更の `dist generate` を、host で取得して `Dockerfile` と同じ固定 sha256 と照合した dist の binary を dev container へ mount して実行した。repo の設定は変えていない。
- tap の `README.md` は slapex の Cask だけを説明している。bizdate の Formula の案内を tap の README に足すかは、tap 側の変更であり、この決定の範囲外とする（#40 の案内整備で扱う）。
- 公開 tap からの `brew install` / `upgrade` と tap への実 push は、初回公開（#40）まで確かめられない。

## 後から見直す条件

- dist の version を上げる場合。Formula の template（分岐、install、`pkgshare` の扱い、`test do` の有無）と、custom publish job の呼び出し条件を読み直す。`prepare-homebrew-formula.sh` は想定と異なる形を見つけたら止まる。
- dist が Formula の test や、tap の version を見た書き込みに対応した場合。builtin の publish job へ戻せるかを検討する。
- GitHub の runner image から Homebrew が外れた場合。runner で Homebrew を入れるか、その target の install の検証をどう扱うかを決め直す。
- tap の運用（slapex 側の自動化や branch 保護）が変わった場合。push の方法と、変更を `Formula/bizdate.rb` に限る検査を読み直す。

## 2026-09-24 追記: secret の名前を slapex と揃える

Issue #80。同じ tap（`kiyohara/homebrew-tap`）へ書く slapex は、GoReleaser の設定（`.goreleaser.yaml` と `.github/workflows/release.yml`）で Actions secret `HOMEBREW_TAP_GITHUB_TOKEN` を参照している（2026-09-24 に HEAD `08e4ba1` を read-only で確認）。同じ tap へ書く token の secret 名がリポジトリごとに違うと、登録と更新の管理が煩雑になる。

### 候補

上の候補（A〜H）と区別するため、I から続ける。

- secret の名前: I dist の慣例の `HOMEBREW_TAP_TOKEN` を続ける / J slapex と同じ `HOMEBREW_TAP_GITHUB_TOKEN` にする

### 検討内容

`HOMEBREW_TAP_TOKEN` を固定で読むのは dist の builtin の publish job（A）であり、上の「決定」で使わないとした。custom の publish job（B）は secret を名前で参照するだけで、名前は bizdate 側で選べる。dist の生成物（`release.yml`）は publish job へ `secrets: inherit` で secret を渡すだけで、名前を持たない。

J にすると、2 つのリポジトリで同じ名前の secret を登録・更新すればよくなる。user account には organization secret が無いため、登録がリポジトリごとに要る点は変わらない。

### 決定

- secret の名前は slapex と同じ `HOMEBREW_TAP_GITHUB_TOKEN` にする（J）。上の「検討内容」の「secret の名前は dist の慣例どおり `HOMEBREW_TAP_TOKEN` のままでよい」と、「決定」の `HOMEBREW_TAP_TOKEN` を、この追記で置き換える。推奨する token の種類（H）は変えない。

### 影響

- `.github/workflows/publish-homebrew.yml` の参照と未設定時のエラー文、`doc/design/distribution.md` の「tap への書き込みの認証」、`dist-workspace.toml` のコメントを新しい名前に揃えた。
- 旧名で登録した secret は publish job から見えない。登録はユーザーが行う。
- [0016](0016-distribution-contract.md) の本文にある `HOMEBREW_TAP_TOKEN`（3 か所）は当時の名前である。
- #40（リリース手順と案内）の Issue 本文を、新しい名前と fine-grained PAT の推奨（H）へ同期し、「依存」に #80 を足した。

### 後から見直す条件

- dist の builtin の publish job へ戻す場合（上の「後から見直す条件」の 2 番目）。builtin の job は `HOMEBREW_TAP_TOKEN` を固定で読むため、secret の名前も戻すか、両方の名前で登録する。
- slapex が secret の名前を変えた場合。
