# 公開後確認 Issue の雛形

`run-release` の手順 2 で起票するときに使う。記録する項目、状態の意味、close する条件は `doc/guidelines/release-guidelines.md` の「公開後確認 Issue」を正とする。

- title: `v<version> の公開後確認`
- 本文: 下の雛形の `<...>` を埋める。公開後の PR を `run-issue-task` で出せるよう、Issue 駆動タスクと同じ節（背景、作業内容・完了条件、検証、スコープ外、依存、実行）を持たせる。
- README に予定表記が残っていなければ、README の切り替えの行を消す。初回公開でなければ、`progress.md` の未公開の記述の行を消す。

````markdown
## 状態

- version: `<version>`（tag: `v<version>`）
- 状態: 公開待ち
- 担当: <人間はユーザー名。agent は agent であることと指示したユーザー>
- 再開条件: <次へ進むために要ること>
- 候補 SHA: 未定
- 確定 SHA: 未定
- Release: 未作成
- run: 未実行
- 未確認の追跡先: なし

## 背景

`v<version>` の公開と、公開後の確認を追う。手順は `doc/guidelines/release-guidelines.md` に従う。公開の操作と、この Issue の完了は分ける。全項目の結果を記録し、残る項目を追跡 Issue へ引き継ぐまで close しない。

## 作業内容・完了条件

この Issue の作業は 2 段に分かれる。公開後確認は `run-release` で行い、結果をこの Issue に記録する。公開後の PR の範囲は、下の「公開後の PR」の項目だけである。

公開後確認（証拠はコメントで残す。結果を記録した項目は、失敗した項目や追跡 Issue へ回した項目も、行末に結果と追跡先を書いて check する）:

- [ ] Release が draft でも prerelease でもなく、asset が 9 件そろう
- [ ] tag の run の job がすべて success（`custom-ci / test / build (<target>)` の 3 job を含む）
- [ ] 確定 SHA が候補 SHA と一致する
- [ ] 9 件を取得でき、各 `.sha256` と `sha256.sum` に一致する
- [ ] `aarch64-apple-darwin`: 展開、同梱物、起動、隔離データでの判定
- [ ] `aarch64-unknown-linux-gnu`: 展開、同梱物、起動、隔離データでの判定
- [ ] `x86_64-unknown-linux-gnu`: 展開、同梱物、起動、隔離データでの判定
- [ ] third-party 表記（3 つの archive）
- [ ] 既定 CSV の取得と、取得したデータでの判定（3 target）
- [ ] Homebrew: tap の Formula の version と sha256、install、`bizdate --version`、`brew test`
- [ ] Homebrew: 前の version からの upgrade（前の version が無ければ追跡 Issue へ）
- [ ] 対応環境の値（最低 glibc、最低 macOS）が README と一致する
- [ ] Release 本文に要約を加えた
- [ ] 前の version の未確認項目の追跡 Issue（あれば）

公開後の PR（確認済みになってから、この Issue を入力に `run-issue-task` で出す）:

- [ ] `progress.md` の「リリース履歴」に行を足す
- [ ] README の予定表記のうち、公開後確認で確かめた経路（archive、Homebrew）の節を有効な案内へ切り替える
- [ ] `progress.md` の「リリース履歴」の前文、「現況」、「次にやること」にある初回公開が未実施という記述を直す（初回公開のみ）

## Release 本文へ加える要約（草稿）

<草稿。公開前確認で書き、承認の提示で示す>

## 検証

- 上の各項目の証拠が、この Issue のコメントにある。
- 確かめられなかった項目は、追跡 Issue に理由、証拠、再開条件がある。

## スコープ外

- 機能の変更と不具合の修正（別の Issue で行い、次の version で公開する）
- 同じ version の作り直しと tag の付け替え

## 依存

なし

## 実行

公開と公開後確認は `run-release` で行う。公開後の PR は、この Issue の状態が確認済みになってから `run-issue-task` で出す。その PR の merge でこの Issue が閉じる。
````
