# リリース手順ルール

この文書は、bizdate の version を公開する作業手順の共通正本である。GitHub Release の公開と Homebrew tap の更新、その前後の確認と記録、失敗したときの扱いを定める。AI agent と人間の両方が従う。

- 配布物の仕様（対象環境、成果物、命名、checksum、version と tag の規則、同梱物、tap を更新する条件）は `doc/design/distribution.md` を正とし、この文書に複製しない。
- release workflow の job の構成は `doc/guidelines/development-command-guidelines.md` の「release workflow」にある。
- 決定の経緯は decision log 0016（公開の運用条件）、0022（release workflow）、0025（Homebrew）、0026（この手順）にある。
- agent は `run-release` skill（`.agents/skills/run-release/SKILL.md`）でこの手順を実行する。skill はこの文書を参照し、手順を複製しない。
- release workflow（`.github/workflows/release.yml` と、それが呼ぶ `ci.yml`、`release-verify.yml`、`publish-homebrew.yml`）の job、`dist-workspace.toml` の成果物、手順から呼ぶ `.github/scripts/` の script の引数を変えたら、この文書の job の一覧（「公開前確認」と「監視」）、asset の数、コマンドを同じ変更で揃える。

## 用語

| 用語 | 意味 |
|---|---|
| 公開 | `v<version>` tag の push で release workflow が GitHub Release を作り、tap の Formula を更新すること。PR の merge は公開ではない |
| 公開担当者 | 公開の準備、確認、記録を担う人。人間、またはユーザーの指示を受けた agent |
| 公開後確認 Issue | version ごとに、公開の状態、確認の結果と証拠、未確認の項目を追う Issue |
| リリース準備 PR | 公開する version にするための `Cargo.toml` と `Cargo.lock` の変更だけを入れる PR |
| 候補 SHA | tag を打つ予定の `main` 上の commit |
| 確定 SHA | push された tag が指す commit |

## 役割と権限

| 操作 | 行う人 | 条件 |
|---|---|---|
| `HOMEBREW_TAP_GITHUB_TOKEN` の発行と登録、期限前の作り直し | ユーザー | 手順は `doc/design/distribution.md` の「tap への書き込みの認証」。agent は secrets を操作しない |
| 公開する version の決定 | ユーザー | 公開担当者は候補を提案してよい |
| 公開後確認 Issue の起票と更新、確認と記録、リリース準備 PR と公開後の PR の作成 | 公開担当者 | - |
| PR の merge | ユーザー | agent は merge しない |
| 公開の承認 | ユーザー | 「承認」の提示に対して、文言で明示する |
| tag の作成と push | ユーザー | 承認した候補 SHA に打つ |
| workflow の再実行と cancel、Release の編集と削除 | 公開担当者 | 対象と理由を示し、ユーザーの承認を得てから行う（`doc/guidelines/github-cli-guidelines.md`） |

- agent は tag を push しない。承認と実行を分けるためである。cloud session では push できるのが作業ブランチだけでもある。
- 既存の tag を付け替えず、消さない。失敗した job を確認なしに再実行しない。
- 同じ version を作り直さない。やり直すときは version を上げる（`doc/design/distribution.md` の「tap の更新」）。

## 流れ

1. 公開後確認 Issue を起票し、既にあれば再利用する（状態: 公開待ち）。
2. 公開する version が `Cargo.toml` と異なれば、リリース準備 PR を出し、ユーザーが merge する。
3. 候補 SHA を決め、公開前確認を行う。
4. 公開対象と確認の結果を提示し、ユーザーの承認を受ける。
5. ユーザーが tag を push する。
6. release workflow を見守り、結果を提示する。Release が作られたら状態を公開済みにする。失敗したら「復旧」に従う。
7. 公開後確認を行い、証拠を公開後確認 Issue に残す。承認を得て Release 本文に要約を加える。
8. 全項目の結果を記録し、残る項目を追跡 Issue へ引き継いだら、状態を確認済みにする。
9. 公開後確認 Issue を入力に `run-issue-task` で PR を出す（リリース台帳の行、README の予定表記の切り替え）。merge で Issue が閉じる。

## 公開後確認 Issue

### 起票と再利用

- 公開担当者は、公開の前（リリース準備 PR を出す場合はその前）に、title が `v<version> の公開後確認` の Issue を探す。
  - open の Issue があれば再利用し、本文の記録を更新する。
  - 同じ version の closed の Issue があれば、その version は公開済みか中止である。起票せず、ユーザーに確認する。
  - 無ければ起票する。本文の雛形は `.agents/skills/run-release/references/post-release-issue.md` にある。
- 予行や練習のために起票しない。
- `progress.md` の索引へ自動では登録しない。横断して追跡したい場合は、`register-progress-issue` で独立した PR として登録する。

### 記録するもの

本文の「状態」節に次を置き、変わったら本文を更新する。

| 項目 | 記録する時点 |
|---|---|
| version と tag | 起票時 |
| 状態 | 起票時（公開待ち）と、変わるたび |
| 担当 | 起票時。人間はユーザー名、agent は agent であることと指示したユーザー |
| 再開条件 | 起票時と、変わるたび。次へ進むために要ること（ユーザーの承認と tag の push、secret の登録後の再実行など） |
| 候補 SHA | 公開前確認の時点 |
| 確定 SHA、Release と run の URL | 公開の後 |
| 未確認の追跡先 | 確認済みにする時点 |

確認の証拠はコメントで残す。書き方は「公開後確認」の「証拠の残し方」に従う。

本文の公開後確認の checklist は、結果を記録した項目を check する。失敗した項目と、追跡 Issue へ回した項目も、行末に結果と追跡先を書いて check する。公開後確認の項目がすべて check されていることを、確認済みにする条件とする（公開後の PR の項目は含めない）。

### 状態

| 状態 | 意味 | 次へ進む条件 |
|---|---|---|
| 公開待ち | 公開の準備と公開前確認をしている | Release が作られた |
| 公開済み | Release は公開された。公開後確認をしている | 全項目の結果を記録し、残る項目を追跡 Issue へ引き継いだ |
| 確認済み | 公開後確認を終えた。失敗した項目も結果として記録してある | 公開後の PR が merge された（Issue が閉じる） |
| 中止 | その version を公開しない | 理由とやり直す version を書き、not planned で close する |

- 公開の操作が終わっても close しない。確認済みになる前に close しない。
- `host` が Release を作った後で止まった場合（tap の更新の失敗など）も、公開済みとして扱う。
- 公開後確認で不具合が見つかった場合、修正は別の Issue で行い、次の version で公開する。公開した version の Release と tag はそのまま残す。

## リリース準備 PR

- 公開する version が `Cargo.toml` の `version` と異なる場合にだけ出す。
- 変更は `Cargo.toml` の `version` と、それに伴う `Cargo.lock` の更新（`docker compose run --rm dev cargo update --workspace`）に限る。機能の変更や修正を混ぜない。
- 起点 Issue を作らない運用作業である（`doc/guidelines/development-loop.md`）。description に公開後確認 Issue を `Refs #<番号>` で書き、`Closes` を付けない。merge は公開ではないことも書く。
- working branch note を作り、PR の採番後に `number-working-branch-note` で採番する。
- merge はユーザーが行う。merge の後、`main` を取り直して候補 SHA を決め、公開前確認を行う。

## 公開前確認

候補 SHA について次を確かめ、結果を公開後確認 Issue にコメントで残す。候補 SHA を変えたら、確認をやり直す。

| 確認 | 方法 | 満たす条件 |
|---|---|---|
| 候補 SHA | `git fetch origin main` の後の `origin/main` の先頭を候補にする | 前回の公開から候補 SHA までに入った PR を一覧できる。一覧は release notes の草稿と承認の提示に使う |
| version と tag | 候補 SHA を checkout し、`docker compose run --rm dev .github/scripts/check-release-tag.sh v<version>` | ok になる |
| 既存の tag と Release | `git ls-remote --tags origin v<version>` と GitHub の Release の一覧 | どちらも無い |
| 未解決の Issue と PR | open の Issue と PR の一覧 | 公開を止める不具合や、入れるはずの未 merge の修正が無い。判断に迷うものはユーザーに示す |
| CI | 候補 SHA の check run（`CI` workflow の push の run） | `fmt / clippy` と `test / build (<target>)` の 3 job が success |
| 配布成果物 | 候補 SHA を `main` に入れた PR の、最後の Release workflow の run | 下の job がすべて success で、記録した値が README と一致する |
| release notes | Release 本文へ加える要約の草稿を、公開後確認 Issue に書く | 「Release 本文の要約」を満たす |
| tap | `https://raw.githubusercontent.com/kiyohara/homebrew-tap/HEAD/Formula/bizdate.rb` の `version` | Formula が無いか、公開する version より古い |
| secret | ユーザーに確認する。一覧を読める環境では `gh api repos/kiyohara/bizdate/actions/secrets` の名前と更新日も見る（cloud session では proxy が拒否する） | `HOMEBREW_TAP_GITHUB_TOKEN` が登録済みで、token が tap に書け、期限が切れていない |

配布成果物の run で確かめる job と値:

- `plan`: `release.yml` と dist の設定が一致し、成果物が決まっている。
- `build-local-artifacts (<target>)` の 3 job と `build-global-artifacts`: 3 target の archive と `.sha256`、`sha256.sum`、`bizdate.rb` の 8 件を作っている。Release には `host` が `dist-manifest.json` を加え、9 件になる。
- `custom-release-verify / archive (<target>)` の 3 job: checksum、archive の構成、third-party 表記、起動の確認が通っている。記録した最低 glibc（Linux の 2 target）と最低 macOS が、README の対応環境と一致する。上がっていれば、README を直す Issue を先に済ませ、release notes にも書く。
- `custom-release-verify / homebrew formula` と `custom-release-verify / homebrew (<target>)` の 3 job: Formula の生成、install、`brew test` が通っている。
- `custom-ci / fmt / clippy`: success。PR の run では `custom-ci / test / build (<target>)` が skipped になるのは想定どおりである。

その run の後に `main` へ別の PR が入っていた場合、run が確かめた内容は候補 SHA と一致しない。その旨を承認の提示に書く。tag push の run は `host` の前に同じ検証を通すため、失敗しても公開はされないが、その version は使えなくなる。

## 承認

公開担当者は、公開前確認を終えたら次を提示し、ユーザーの承認を待つ。承認は、この提示に対するユーザーの文言で受ける。ボタンの押下や、無関係な発言を承認として扱わない。

- version と tag、候補 SHA（40 桁）
- 公開されるもの: GitHub Release（asset 9 件）と、tap の `Formula/bizdate.rb` の更新
- 公開前確認の結果と、確かめた run と check の URL
- Release 本文へ加える要約の草稿と、公開後に公開担当者が Release 本文を編集してよいか
- ユーザーが実行するコマンド（「tag の push」）
- 取り消せないこと: Release は `host` の段階で公開され、tap は `custom-publish-homebrew` で更新される。同じ version は作り直さない

提示の後に候補 SHA や公開の内容が変わった場合は、提示と承認をやり直す。

## tag の push

承認の後、ユーザーが次を実行する。`<version>` と `<候補 SHA>` は提示した値に置き換える。

```sh
git fetch origin main
git tag -s v<version> <候補 SHA> -m "bizdate <version>"
git rev-parse "v<version>^{commit}"
git push origin v<version>
```

- `git rev-parse` の出力が候補 SHA と一致することを確かめてから push する。
- 署名は `doc/guidelines/git-operation-guidelines.md` に従う。署名や push が 1Password の承認待ちで失敗した場合は `doc/guidelines/one-password-integration-guidelines.md` に従う。
- `git push --tags` で他の tag をまとめて push しない。
- push した後は、公開担当者に知らせる。

## 公開の監視と復旧

### 監視

公開担当者は、tag の Release workflow の run（`head_branch` が `v<version>` の push の run）を見守り、終わったら結果をユーザーに示す。すべて success であることを期待する job は次のとおりである。

- `plan`、`build-local-artifacts (<target>)` の 3 job、`build-global-artifacts`
- `custom-ci / fmt / clippy` と `custom-ci / test / build (<target>)` の 3 job。後者が skipped でなく success であること（PR の run では確かめられない）
- `custom-release-verify / release tag`、`custom-release-verify / archive (<target>)` の 3 job、`custom-release-verify / homebrew formula`、`custom-release-verify / homebrew (<target>)` の 3 job
- `host`、`custom-publish-homebrew / formula`、`announce`

job の名前は release workflow に従う。workflow を変えたときに揃える範囲は、冒頭のとおりである。

### 復旧

失敗したら、Release の実状（有無、draft か prerelease か、asset の名前と数）と tap の Formula を確かめてから判断する。Release は `host` の段階で公開されるため、未公開と決めつけない。

| 止まった job | Release | tap | 扱い |
|---|---|---|---|
| `host` より前 | 作られない | 変わらない | 原因を調べて示す。runner の喪失や取得の一時的な失敗など、内容と無関係なら、承認を得て失敗した job を再実行する。内容に原因があれば、修正を PR で入れて version を上げる（公開後確認 Issue は中止にする） |
| `host` | 作られないか、asset の一部だけで作られる | 変わらない | Release が無ければ、承認を得て再実行する。有る場合、再実行は作成で失敗する。不足の asset を足すか、version を上げるかをユーザーと決める |
| `custom-publish-homebrew / formula` | 公開済み | 書き込み前に止まれば変わらない | secret の未設定、token の期限切れや権限の不足なら、ユーザーが secret を直した後、承認を得てこの job を再実行する。同じ内容の再実行は何も変えずに成功する。巻き戻し、同じ version の内容違い、Formula 以外の変更で止まった場合は再実行せず、version を上げる |
| `announce` | 公開済み | 更新済み | Release と tap には影響しない。原因を示し、再実行の要否をユーザーと決める |

- 再実行は、失敗の内容、再実行する job、期待する結果を示し、承認を得てから行う。同じ失敗が繰り返したら再実行を重ねず、原因を調べる。
- tag を付け替えない。version を上げてやり直す場合、Release が作られていなければ公開後確認 Issue を中止として閉じる。作られていれば公開済みのまま公開後確認を行い、結果を記録する。新しい version は「流れ」の 1 からやり直す。

### 承認を得て行う操作

承認の提示では、実行するコマンドも示す。`gh` の実行形式は `doc/guidelines/github-cli-guidelines.md` に従う。

- job の再実行: `gh run rerun --job <job-id>`。`<job-id>` は job の数値 ID で、`gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId}'` の `databaseId` である。失敗した job をまとめて再実行する場合は `gh run rerun <run-id> --failed`。組み込みの GitHub tool の `actions_run_trigger` は、見えていても使わない（`doc/guidelines/github-mcp-guidelines.md` の「CI 操作の境界」）。
- Release 本文の編集: 本文は全体が置き換わる。今の本文と Release の `id` を読み（組み込みの GitHub tool の `get_release_by_tag`、または `gh api repos/kiyohara/bizdate/releases/tags/v<version>`）、要約を前に加えた全文を file に書いて、`gh api -X PATCH repos/kiyohara/bizdate/releases/<release-id> -F body=@<file>` で更新する。更新の後に読み戻して確かめる。
- cloud session の proxy がこれらを拒否した場合は、経路を変えて再試行せず、ユーザーに GitHub の UI での操作（run の画面の再実行、Release の編集）を依頼する。

## 公開後確認

公開済みになったら、次をすべて確かめる。

| 項目 | 満たす条件 | 環境 |
|---|---|---|
| Release | draft でも prerelease でもなく、asset が 9 件そろう | どこでもよい |
| tag の run | 「監視」の job がすべて success | どこでもよい |
| 確定 SHA | tag が指す commit が候補 SHA と一致する | どこでもよい |
| 取得と checksum | 9 件を取得でき、各 `.sha256` と `sha256.sum` に一致する | どこでもよい |
| 展開、同梱物、起動、隔離データでの判定 | `verify-release-archive.sh`（macOS は下の手順）が通る | 各 target の native 環境 |
| third-party 表記 | 各 archive の `THIRD-PARTY-LICENSES.md` が `check-third-party-licenses.sh` を通る | Compose の dev service |
| 既定 CSV の取得 | `--source` を付けない `fetch-holidays` が成功し、取得したデータで判定できる | 各 target の native 環境のうち、取得先に届くもの |
| Homebrew | tap の Formula の `version` と各 `sha256` が Release と一致する。install した `bizdate --version` が `bizdate <version>` を出し、`brew test` が通る。前の version から `brew upgrade` できる | macOS。Linux の Homebrew があれば Linux も |
| 対応環境の値 | tag の run が記録した最低 glibc と最低 macOS が README と一致する | どこでもよい |

emulation で動かした結果を native の確認としない。cloud session の sandbox は Linux x86_64 であり、取得先の `www8.cao.go.jp` へは proxy が接続を拒否するため、既定 CSV の取得はできない。macOS と Linux arm64 は、Apple Silicon の Mac と、その上の Docker（Linux arm64 の native）で確かめられる。native の環境を用意できない項目は、「未確認項目の追跡」に従って残す。

### 手順

作業は tag の commit を checkout した作業ツリーで行う（`git fetch origin tag v<version>` の後に `git switch --detach v<version>` など。終わったら元のブランチへ戻る）。script、`Cargo.lock`、比べる `README.md` と `LICENSE` を tag の時点に揃えるためである。asset は repository 直下の一時ディレクトリに置き、Compose の container から見えるようにする。commit せず、終わったら消す。cloud session の container からは GitHub の asset を取得できないため、取得は host で行う。

取得と checksum:

```sh
v=<version>
dir=$(mktemp -d ./release-check.XXXXXX)
(
    cd "$dir" || exit 1
    for f in sha256.sum dist-manifest.json bizdate.rb \
        bizdate-aarch64-apple-darwin.tar.gz bizdate-aarch64-apple-darwin.tar.gz.sha256 \
        bizdate-aarch64-unknown-linux-gnu.tar.gz bizdate-aarch64-unknown-linux-gnu.tar.gz.sha256 \
        bizdate-x86_64-unknown-linux-gnu.tar.gz bizdate-x86_64-unknown-linux-gnu.tar.gz.sha256; do
        curl -fsSLO "https://github.com/kiyohara/bizdate/releases/download/v$v/$f" || exit 1
    done
    sha256sum -c sha256.sum || exit 1
    for a in bizdate-aarch64-apple-darwin.tar.gz bizdate-aarch64-unknown-linux-gnu.tar.gz bizdate-x86_64-unknown-linux-gnu.tar.gz; do
        awk -v a="$a" '{ sub(/^\*/, "", $2) } $2 == a { found = 1 } END { exit !found }' sha256.sum || { echo "$a is not in sha256.sum" >&2; exit 1; }
        sha256sum -c "$a.sha256" || exit 1
    done
) && echo "ok: 9 assets fetched and verified"
```

取得できない asset が 1 件でもあれば、その時点で非 0 で終わる。`sha256.sum` に 3 つの archive の行があることも確かめ、各 archive の `.sha256` は名前を指定して検査するため、asset が欠けていれば通らない。最後に `ok:` の行が出れば、9 件の取得と checksum の確認が済んでいる。macOS では `sha256sum -c` の代わりに `shasum -a 256 -c` を使う。

Linux（container の architecture の target。cloud session では x86_64、Apple Silicon の Mac では arm64）:

```sh
docker compose run --rm dev sh -c 'target=$(rustc -vV | sed -n "s/^host: //p") && .github/scripts/verify-release-archive.sh "$target" "$1" "$1/dist-manifest.json"' sh "$dir"
```

macOS（host で実行する。third-party 表記は Compose で確かめる）:

```sh
tar -tzf "$dir/bizdate-aarch64-apple-darwin.tar.gz"
mkdir "$dir/macos"
tar -xzf "$dir/bizdate-aarch64-apple-darwin.tar.gz" -C "$dir/macos" --strip-components=1
.github/scripts/platform-check.sh "$dir/macos/bizdate"
cmp "$dir/macos/README.md" README.md && cmp "$dir/macos/LICENSE" LICENSE
docker compose run --rm dev .github/scripts/check-third-party-licenses.sh "$dir/macos/THIRD-PARTY-LICENSES.md"
```

`tar -tzf` の一覧は、`bizdate-aarch64-apple-darwin/` の下に `bizdate`、`README.md`、`LICENSE`、`THIRD-PARTY-LICENSES.md` だけが並ぶこと。

既定 CSV の取得（`<bizdate>` は展開した binary。Linux は archive を `"$dir/linux"` に同じ形で展開し、`docker compose run --rm dev sh` の中で実行する）:

```sh
data=$(mktemp -d)
XDG_DATA_HOME="$data" <bizdate> fetch-holidays
XDG_DATA_HOME="$data" <bizdate> first --date 2026-01-01 --timezone Asia/Tokyo   # no、exit 1（元日）
XDG_DATA_HOME="$data" <bizdate> first --date 2026-01-02 --timezone Asia/Tokyo   # yes、exit 0
XDG_DATA_HOME="$data" <bizdate> last --date 2026-09-30 --timezone Asia/Tokyo    # yes、exit 0
rm -rf "$data"
```

Homebrew（macOS）:

```sh
curl -fsSL https://raw.githubusercontent.com/kiyohara/homebrew-tap/HEAD/Formula/bizdate.rb | grep -E '^  version |sha256 '
brew update
brew install kiyohara/tap/bizdate
bizdate --version
brew test kiyohara/tap/bizdate
```

`brew update` を先に実行するのは、手元の tap の clone を最新にするためである。tap は slapex と共用で、既に tap してある Mac では、自動更新（既定で 24 時間に 1 回）を待つ間、古い clone が使われ得る。そのままでは install の失敗や upgrade の空振りが、tap への書き込みの失敗と見分けにくい。前の version を入れた環境では、`brew update` の後に、`brew install` の代わりに `brew upgrade kiyohara/tap/bizdate` を実行する。Release に添付される `bizdate.rb` は dist が生成したままの Formula であり、tap の Formula（検査して `test do` を足したもの）と内容は一致しない。一致を求めるのは `version` と `sha256` である。

### 証拠の残し方

- 結果は公開後確認 Issue にコメントで残す。項目ごとに、実行した環境（OS と version、architecture、Homebrew の version など）、コマンド、結果（exit code と出力の要点）を書く。
- ローカルの絶対 path、ユーザー名、ホスト名は `<tmp>` などに置き換える。asset の取得で redirect された先の URL（署名付き）を貼らない。
- 確かめられなかった項目は、理由と、確かめるために要るもの（環境、次の version の公開など）を書く。

## Release 本文の要約

- dist が生成する本文（install の方法と asset の一覧）の前に、利用者向けの要約を加える。README と同じくですます調で書く。
- 変更点の要約を書く。最低 glibc の実測値が上がった場合と、実行時前提が変わった場合は必ず書く（`doc/design/distribution.md` の「リリースノート」）。
- 草稿は公開前に公開後確認 Issue に置き、承認の提示で示す。Release の編集は公開済みになった後、ユーザーの承認を得てから行う。

## 未確認項目の追跡

- 確認済みにする時点で確かめられない項目（初回公開での `brew upgrade` など）は、項目ごとに追跡 Issue を起こす。title は `v<version> の未確認項目: <項目>` とし、確かめられない理由、それまでの証拠、再開条件、確かめる手順を書く。
- 公開後確認 Issue の「未確認の追跡先」と、リリース台帳の「未確認」に追跡 Issue を書く。
- 公開担当者は、次の version の公開後確認で open の追跡 Issue を確かめ、結果をその Issue に残して close する。
- 追跡 Issue も `progress.md` の索引へ自動では登録しない。

## 公開後の PR

- 公開後確認 Issue が確認済みになったら、その Issue を入力に `run-issue-task`（review まで回す場合は `drive-issue-to-reviewed-pr`）で PR を出す。description に `Closes #<公開後確認 Issue>` を含める。
- PR の内容は、`progress.md` の「リリース履歴」への行の追加と、README に予定表記が残っていればその切り替えである。初回公開では `progress.md` の未公開の記述も直す（「初回公開（v0.1.0）」）。
- README で有効な案内に切り替えるのは、公開後確認で確かめた経路（archive と Homebrew）の節だけとする。確かめられなかった経路の節は予定表記のまま残し、その経路を確かめた version の公開後の PR で切り替える。
- 公開後確認で見つかった、この文書や README の記述の食い違い（job の表示名、対応環境の値など）を公開後の PR で直す場合は、先に公開後確認 Issue の「公開後の PR」に項目を足す。PR の範囲はその項目だけとする。
- 公開の確認を終える前に、README の予定表記を外さない。手順を入れた PR や、リリース準備 PR の merge を、公開の確認の完了として扱わない。

## リリース台帳

- 台帳は `progress.md` の「リリース履歴」である。列と更新の責任は同節に書く。
- 行を足すのは公開担当者であり、「公開後の PR」で足す。公開していない version、中止した version、確認を終えていない version の行を足さない。

## 初回公開（v0.1.0）

初回に固有の扱い:

- `Cargo.toml` の `version` は既に `0.1.0` であり、リリース準備 PR は要らない。候補 SHA は、公開前確認の時点の `main` の先頭とする。
- README のインストール案内は公開予定の表記で置いてある。公開後の PR で、確かめた経路の予定表記を外し、有効な案内に切り替える（「公開後の PR」）。対応環境の値が tag の run の記録と異なれば、「公開後の PR」に従って同じ PR で直す。
- 公開後の PR では、`progress.md` の「リリース履歴」の前文（まだ公開した version は無い）と、「現況」と「次にやること」の初回公開が未実施という記述も直す。
- tag の commit の README が archive に同梱されるため、`v0.1.0` の archive の README は予定表記のままである。予定表記は、公開後に読んでも誤りにならない文言にしてある。
- 前の version が無いため、`brew upgrade` は確かめられない。追跡 Issue を起こし、次の version の公開後確認で確かめる。tap は単一の version しか持たず、次の version の公開の後は `v0.1.0` を入れ直せないため、Homebrew で入れた `v0.1.0` をその確認まで残しておく。追跡 Issue の再開条件にもそう書く。
- tap の README は slapex の Cask だけを説明している（decision log 0025）。Homebrew の install を確かめた後に、ユーザーが tap の README に bizdate の Formula の案内を足す。tap 側の変更であり、この repository の PR では行わない。
- 「リリース履歴」の最初の行は、公開後の PR で足す。

初回公開の前にそろえるもの:

- `HOMEBREW_TAP_GITHUB_TOKEN` の登録（ユーザー）。cloud session からは登録の有無を読めないため、ユーザーが名前、対象の repository、期限を確かめる。

初回公開で初めて確かめるもの（PR の run では確かめられない）:

- tag push の run でだけ行われるもの: `custom-ci / test / build (<target>)` の 3 job、`custom-release-verify / release tag` の tag の検査（PR の run でも job は success になるが、検査の step は走らない）、`host`、`custom-publish-homebrew / formula`、`announce`
- tap への実際の書き込み（secret の名前と権限を含む）
- 公開 asset の取得、checksum、展開、各 target での起動
- 公開 tap からの `brew install` と Formula の version
- 既定 CSV の実際の取得（cloud session からは届かない）。`x86_64-unknown-linux-gnu` は、Apple Silicon の Mac の Docker では emulation になるため、native の x86_64 Linux が要る
- 公開後確認で、Release の `dist-manifest.json` を `verify-release-archive.sh` の plan として使えること。PR の run では `dist plan` の出力でしか動かしていない。通らない場合は、tag の run の artifact `artifacts-plan-dist-manifest` の `plan-dist-manifest.json` を使う（cloud session からは artifact を取得できないため、ユーザーが run の画面から取得する）
- 「監視」の job の表示名が、tag の run の表示と一致すること。PR の run では、`custom-ci / test / build` は matrix が展開されない名前（`custom-ci / test / build (${{ matrix.target }})`）で、`custom-publish-homebrew` は呼び出し側の job の名前で skipped になり、tag の run での表示名は見られない。異なれば、「公開後の PR」に従って「監視」と「公開前確認」の一覧を直す

ユーザーが行う操作（順に）:

1. `HOMEBREW_TAP_GITHUB_TOKEN` を登録する。登録済みなら、名前、対象の repository、期限を確かめる。
2. 公開担当者に v0.1.0 の公開の準備を指示する。公開担当者は公開後確認 Issue を起票し、公開前確認を行い、承認を求める。
3. 提示を確かめ、文言で承認する。
4. 「tag の push」のコマンドで `v0.1.0` を push する。
5. 公開後確認のうち、macOS と Linux arm64 の確認（既定 CSV の取得を含む）と Homebrew を、Mac で行う（公開担当者が agent なら、手順と結果の記録先を示す）。Homebrew で入れた `v0.1.0` は、次の version の公開後確認まで残しておく。
6. `x86_64-unknown-linux-gnu` の既定 CSV の取得を、native の x86_64 Linux で行う。用意できなければ追跡 Issue に残る。
7. Homebrew の install を確かめたら、tap の README に bizdate の Formula の案内を足す。
8. 公開後確認 Issue を入力とする PR（台帳の行、README の切り替え、`progress.md` の未公開の記述）を review し、merge する。
