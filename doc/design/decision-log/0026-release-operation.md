<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 公開の作業手順と公開後の追跡

- 状態: decided
- 作成日: 2026-09-24
- 最終更新日: 2026-09-24
- 関連: `doc/design/distribution.md`, `doc/guidelines/release-guidelines.md`, `.agents/skills/run-release/SKILL.md`, `doc/guidelines/development-loop.md`, `progress.md`, `README.md`, [0012-development-loop.md](0012-development-loop.md), [0016-distribution-contract.md](0016-distribution-contract.md), [0025-homebrew-formula-publishing.md](0025-homebrew-formula-publishing.md), [Issue #40](https://github.com/kiyohara/bizdate/issues/40)

## 背景

[0016](0016-distribution-contract.md) で公開の運用条件（`v<version>` tag の push が trigger、secret の登録と tag の push と公開の承認はユーザー、失敗したら version を上げる、Release の実状を確かめてから復旧する）を決め、[0022](0022-release-workflow.md) と [0025](0025-homebrew-formula-publishing.md) で release workflow と tap への書き込みを置いた。作業手順そのもの（誰がいつ何をするか、公開の前後に何を確かめるか、失敗したらどうするか、確認をどこに残すか）は #40 のリリース guideline を正本とすることになっていたが、まだ無かった。README にはインストールの案内が無く、`progress.md` のリリース台帳も列が決まっていなかった。`development-loop.md` は、リリース手順を担う skill を索引登録と進捗整理と同種の運用作業として扱うとしていたが、例外の範囲は決まっていなかった。

公開は取り消しが効きにくい。公開の操作（tag の push）と、公開した成果物の確認は、時点も実行環境も分かれる。macOS の確認にはユーザーの Mac が要り、既定の祝日 CSV の取得先へは cloud session から届かない。公開の操作を終えた時点で完了とすると、確かめていない項目が追跡されずに残る。

## 候補

公開後の確認を追う場所:

- A. version ごとの公開後確認 Issue に、状態、担当、再開条件、証拠を残す。
- B. `progress.md` の台帳の行に状態を持たせる。
- C. 手順を入れた PR や working branch note に残す。

version を変える PR の扱い:

- D. リリース準備 PR を、起点 Issue を持たない運用作業の例外とする。公開後確認 Issue を参照し、`Closes` を付けない。
- E. version を上げるための Issue を毎回起こし、通常の Issue 駆動タスクとする。

tag の push:

- F. ユーザーが行う（0016 のまま）。
- G. 承認を受けた agent が行う。

README のインストール案内:

- H. 公開予定の手順と明示して先に置き、公開の確認後に、公開後確認 Issue を入力とする PR で有効な案内へ切り替える。
- I. 公開の確認後に初めて書く。

## 検討内容

- B は台帳に公開前や確認中の状態が混ざり、公開していない version の行が台帳に載る。C は PR の merge で追跡が閉じ、手順の導入と実際の公開の確認が混ざる。A は Issue の open と close で未完了が見え、証拠をコメントで時系列に残せる。公開後の PR をその Issue を入力とする Issue 駆動タスクにでき、merge で閉じる流れが既存のループに乗る。
- E の「version を上げるための Issue」は、索引登録や進捗整理のための Issue と同じく、指示書として意味を持たない。公開後確認 Issue を起点にすると、公開後の PR と合わせて 1 Issue に 2 PR となり、1 Issue = 1 PR を崩す。D は公開後確認 Issue を参照で辿れ、1 Issue = 1 PR を保てる。
- G は、cloud session では push できるのが作業ブランチだけであり、実行できない。承認した人と実行する人を分けないと、提示と異なる commit に tag を打つ誤りを止める人がいない。F は 0016 と一致する。
- I は、#40 を終えても README に案内が無く、公開の直後に案内の PR を急ぐことになる。H は予定と明示すれば事実と食い違わず、公開の後は予定表記を外す差分だけを review すればよい。ただし tag の commit の README が archive に同梱されるため、初回（`v0.1.0`）の archive には予定表記の README が入る。予定表記は、公開後に読んでも誤りにならない文言にする。0016 の「実際に Release を公開してから切り替える」は、有効な案内への切り替えを公開の後に行う点で維持される。
- 予定表記を version の公開ごとに一括で外すと、Homebrew だけが失敗した version（Release は公開済みで、tap への書き込みが内容の問題で止まった場合など）で、使えない Homebrew の案内が有効になる。公開後確認で確かめた経路（archive、Homebrew）の節だけを切り替えれば、archive の案内は先に有効にでき、残りの経路はそれを確かめた version の公開後の PR で切り替えられる。
- tap の README は slapex の Cask だけを説明しており、[0025](0025-homebrew-formula-publishing.md) は bizdate の Formula の案内を足すかを #40 に委ねた。そのままでは、tap を見た人が bizdate の Formula を知る手段が無い。tap の変更はこの repository の PR では行えず、tap への書き込みに当たる。公開の前に足すと、まだ入れられない Formula を案内することになる。
- release notes は、`CHANGELOG.md` を導入しない（0016）ため repository の file ではない。Release 本文は `host` の段階で dist の生成物として作られ、要約は公開の後にしか加えられない。そこで、要約の草稿を公開前に公開後確認 Issue に置き、承認の提示で示す。

## 決定

- 公開は、version ごとの公開後確認 Issue で追う（A）。公開担当者が公開の前に起票し、既にあれば再利用する。状態は公開待ち、公開済み、確認済み、中止とし、公開の操作と Issue の完了を分ける。全項目の結果を記録し、残る項目を追跡 Issue へ引き継ぐまで close しない。予行のためには起票しない。
- リリース準備 PR（`Cargo.toml` の version と `Cargo.lock` の更新だけ）は、起点 Issue を作らない運用作業の例外とする（D）。公開後確認 Issue を参照し、`Closes` を付けない。公開後確認 Issue を入力とする PR（リリース台帳の行、README の予定表記の切り替え）は通常の Issue 駆動タスクとする。恒常的な例外は、索引登録、進捗整理と合わせて 3 件になる。
- tag の push はユーザーが行う（F）。公開担当者は公開対象と公開前確認の結果を提示し、ユーザーの文言での承認を受ける。既存の tag を付け替えず、失敗した job を確認なしに再実行しない。
- README のインストール案内は公開予定の手順として置き、公開の確認後に有効な案内へ切り替える（H）。切り替えるのは、公開後確認で確かめた経路（archive、Homebrew）の節だけとし、確かめられなかった経路の節は予定表記のまま残す。
- tap の README への bizdate の Formula の案内は、初回公開で Homebrew の install を確かめた後に、ユーザーが tap 側で足す。この repository の PR では行わない。
- Release 本文の要約は、草稿を公開後確認 Issue に置いて承認の提示で示し、公開の後に承認を得て加える。
- リリース台帳は `progress.md` の「リリース履歴」とし、列は version、公開日、Release、検証、未確認とする。行は公開後確認を終えた version についてだけ、公開後確認 Issue を入力とする PR で足す。
- 確かめられない項目（初回公開での `brew upgrade` など）は、項目ごとに追跡 Issue を起こし、次の version の公開後確認で確かめる。
- 手順の正本は `doc/guidelines/release-guidelines.md`、実行の順序は `run-release` skill とする。spec、decision log、skill に手順を複製しない。

## 理由

- 公開の操作と確認の完了を別の単位にすると、確認が終わるまで未完了が Issue の open として見え続ける。
- 例外を運用作業に限る既存の方針（[0012](0012-development-loop.md) と `development-loop.md`）と、1 Issue = 1 PR を両立できる。
- 承認と tag の push をユーザーに残すと、取り消せない操作の直前に人が必ず介在する。
- 予定の案内を先に置くと、公開の後に必要な作業が予定表記を外す差分に縮む。

## 影響

- `doc/guidelines/release-guidelines.md` を追加し、`AGENTS.md`、guideline の一覧、Cursor と Claude Code の入口、`run-release` skill と公開後確認 Issue の雛形を置いた。
- `development-loop.md` の恒常的な例外を 3 件にし、リリース準備 PR の範囲を書いた。`maintain-progress` の観点 5 を台帳の定義へ揃えた。
- README に、公開予定のインストール案内を置いた。`v0.1.0` の archive には予定表記の README が入る。
- 初回公開のユーザーの操作に、tap の README への案内の追加を入れた。
- `progress.md` に「リリース履歴」の節を置いた。行はまだ無い。
- `.github/copilot-instructions.md` の README と手順の正本の記述を揃えた。

## 後から見直す条件

- tag の push を agent に任せる運用へ変える場合。承認と実行の分離を別の手段で担保する必要がある。
- 公開後の確認を workflow で自動化できるようになった場合（公開 asset を取得して検証する job など）。
- `CHANGELOG.md` を導入する場合。release notes の置き場が変わる。
- 公開の頻度が上がり、version ごとの Issue の運用が負担になった場合。
