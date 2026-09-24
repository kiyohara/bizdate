# 作業ブランチメモ

- ブランチ: `claude/project-thread-0ebbe4`
- PR: #83
- 最終更新: 2026-09-24

## 目的

Issue #40。version を公開する作業手順（公開後確認 Issue、リリース準備 PR、公開前確認、承認と tag の push、監視と復旧、公開後確認、未確認項目の追跡）の共通正本と、それを参照する実行 skill を置く。README に公開予定のインストール案内を、`progress.md` にリリース台帳を置く。実際の初回公開、tag の push、secrets と settings の変更はスコープ外である。

## 現在の状況

- cloud session（Claude Code on the web）で、`drive-issue-to-reviewed-pr` で進めている。P1（実装と PR 作成）を終えた。
- 依存の #39（PR #79）と #80（PR #81）は merge 済み。レビュー待ちの自分の PR は無い。
- 手順の正本 `doc/guidelines/release-guidelines.md`、skill `run-release`（雛形 `references/post-release-issue.md`）、Cursor と Claude Code の入口、decision log 0026 を置き、AGENTS.md、guideline の一覧、`development-loop.md`、`maintain-progress`、`run-issue-task`、README、`progress.md`、`distribution.md` の参照、Copilot 用の指示を揃えた。

## 決定事項

- 公開は version ごとの公開後確認 Issue（title `v<version> の公開後確認`）で追う。状態は公開待ち、公開済み、確認済み、中止。確認済みになる前に close しない。公開後の PR（台帳の行、初回の README の切り替え）はその Issue を入力とする Issue 駆動タスクとし、merge で閉じる。
- リリース準備 PR（`Cargo.toml` の version と `Cargo.lock` だけ）を、起点 Issue を持たない運用作業の例外にした。公開後確認 Issue を `Refs` で参照し、`Closes` を付けない。公開後確認 Issue から出すと 1 Issue = 1 PR が崩れるため。`development-loop.md` の恒常的な例外は 3 件になった。
- tag の push は 0016 のとおりユーザーが行う。agent は公開対象と確認の結果を提示し、文言での承認を待つ。tag は `git tag -s`（署名付きの annotated tag）を既定にした。commit の署名の方針に揃えるため。
- release notes は `CHANGELOG.md` を置かない方針（0016）のまま、要約の草稿を公開後確認 Issue に置き、承認の提示で示す。Release の編集は公開後に承認を得て行う。
- README の案内は公開予定と明示して先に置いた。`v0.1.0` の archive に同梱されるため、公開後に読んでも誤りにならない文言にした（「最初のリリースを公開する前に用意した予定の手順」「公開を確認するまで使えない」）。
- 台帳は `progress.md` の「リリース履歴」に、列の説明と見出しだけの表を置いた。行は無い。
- 公開後確認は tag の commit を checkout して行う。script、`Cargo.lock`、比べる `README.md` と `LICENSE` を tag の時点に揃えるため。Linux は Compose の dev service で `verify-release-archive.sh` を、macOS は host で `platform-check.sh` を使い、third-party 表記は Compose で確かめる（`check-third-party-licenses.sh` が cargo を要するため）。
- 方針は decision log 0026 を新規に作って記録した。0016 の運用判断の具体化と、0012 の例外の拡張にまたがるため、既存ログへの追記にしなかった。
- 初回公開の未設定・未検証の項目とユーザーの操作の順は、guideline の「初回公開（v0.1.0）」に置き、`progress.md` の「次にやること」から参照した。

## 次にやること

- [x] guideline、skill、入口、README、`progress.md`、decision log を書く
- [x] 検証と予行を行い、結果を記録する
- [x] PR を作成し、note を採番する
- [ ] review cycle を回す

## 検証

| 項目 | 結果 |
|---|---|
| `git diff --check` | OK |
| 配置と入口 | rule の basename が 3 箇所で一致、`AGENTS.md` の共通正本と「AI Agent 向けルール」、guideline の一覧に追加済み。`test -f .claude/skills/run-release/SKILL.md` は OK、`find -L .claude/skills -maxdepth 1 -type l` で壊れた symlink は 0 件 |
| 参照先の存在 | 新規・変更した文書の repo 内 path を抽出して確かめた。存在しないのは生成物（`THIRD-PARTY-LICENSES.md`）、導入しない `CHANGELOG.md`、文中の file 名の略記だけ |
| 文体 | 開発者向け（guideline、skill、雛形、decision log、`progress.md`）に「です・ます」の文末は無い。README の追加部分に常体の文末は無い |
| shell の code block | placeholder を置き換えて `sh -n` で構文を確かめた。guideline 6 件、README 10 件とも OK |
| 予行: tag と version | Compose の dev service で `check-release-tag.sh v0.1.0` は ok、`v0.1.1` は exit 1 |
| 予行: 既存の tag と Release | GitHub の一覧で tag 0 件、Release 0 件 |
| 予行: tap | raw の URL で `Formula/bizdate.rb` は 404（未公開）。tap の README は 200 で、cloud session から読める。tap の API は proxy が 403 を返す |
| 予行: ruleset | branch 対象の "main protection" だけで、tag の push を止める rule は無い |
| 予行: CI（候補の見本） | main の先頭 `201bda4` の CI（push）は `fmt / clippy` と 3 target の `test / build` が success |
| 予行: 配布成果物（候補の見本） | PR #81 の Release workflow の run 35999200425（head `162a983`）は、`host` 以降の skipped を除き success。`build-global-artifacts` の log で `bizdate.rb` と `sha256.sum` の生成を確かめた。run の時点の main は `37e7005` で、`201bda4` の first parent と一致する（run は `201bda4` と同じ内容を確かめた） |
| 予行: job 名 | PR の run の job 名と `release.yml`、`ci.yml`、`release-verify.yml`、`publish-homebrew.yml` の定義から、tag push の run の 20 job を guideline の「監視」に書いた。tag push の run での表示名は未検証 |
| 予行: 公開後確認のコマンド | 模擬の archive（dist と同じ構成、`.sha256` の末尾に空行）で、README と guideline の checksum、展開、配置の手順が通った（coreutils 9.4 は警告なし）。Compose の dev service への引数の渡し方と target の解決（`x86_64-unknown-linux-gnu`）を確かめた。実際の Release asset では未検証 |
| cloud session からの到達性 | host から GitHub の Release asset と raw.githubusercontent.com に届く。container からは github.com の asset に届かない（TLS の検証で失敗）。`www8.cao.go.jp` は host（403）と container の両方から届かない |
| skill の検出 | `run-release` は追加の直後に cloud session の skill 一覧に載った。description での発火は未検証（新しい session で確かめられる） |
| Rust の検証（`cargo fmt` / `clippy` / `test`） | 省略。Rust のコードと Cargo の設定を変えていないため。PR CI では回る |
| 初回公開、tap の更新、実際の install | 未実施（スコープ外）。公開 asset の取得、公開 tap からの install と upgrade、既定 CSV の取得は未検証 |
| note の情報統制 | 確認済み。秘密情報、個人情報、ローカルの絶対 path、署名付き URL を書いていない |

## リスク・ブロッカー

- `HOMEBREW_TAP_GITHUB_TOKEN` の登録の有無は cloud session から読めない。初回公開の前にユーザーが確かめる。
- `verify-release-archive.sh` に Release の `dist-manifest.json` を渡す使い方は、公開前には確かめられない（plan の manifest と同じ形式である前提）。初回公開で確かめる。
- `x86_64-unknown-linux-gnu` の既定 CSV の取得には native の環境が要る。cloud session からは届かず、Apple Silicon の Mac の Docker では emulation になる。用意できなければ追跡 Issue に残る。
- `v0.1.0` の archive の README は予定表記のままになる。

## セッションログ

- 2026-09-24: #40 に着手した。依存の完了を確かめ、配布仕様、release workflow、既存の skill と guideline を読み、PR #81 の run と main の CI で生成物と job を照合した。guideline、skill、入口、README、`progress.md`、decision log 0026 を置き、上の検証と予行を行った。
- 2026-09-24: PR #83 を draft で作成し、`number-working-branch-note` の手順で note を採番した（完了として書き換えたタスク行: note の「PR を作成し、note を採番する」の 1 行、PR description は 0 行。触らなかった stale 表現・タスク行: 0 件）。`progress.md` の DIST-05 の PR 列に #83 を記入した。reviewer の指定は、PR の作成者と同じ account のため GitHub が受け付けなかった（assignee は設定済み）。
