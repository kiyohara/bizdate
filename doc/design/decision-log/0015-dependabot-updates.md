<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# Dependabot による GitHub Actions と Cargo の更新

- 状態: decided
- 作成日: 2026-09-09
- 最終更新日: 2026-09-23
- 関連: `.github/dependabot.yml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `dist-workspace.toml`, `doc/guidelines/development-loop.md`, [Issue #21](https://github.com/kiyohara/bizdate/issues/21), [Issue #38](https://github.com/kiyohara/bizdate/issues/38), [Issue #63](https://github.com/kiyohara/bizdate/issues/63), [Issue #73](https://github.com/kiyohara/bizdate/issues/73)

## 背景

PR #20 で第三者 action を commit SHA に固定した。手動更新だけでは上流への追従を忘れやすいため、Issue #21 で更新方法を整備する。

## 候補

- 更新頻度: weekly / monthly。
- PR の単位: action ごと / GitHub Actions 全体をグループ化。
- 日付コメント: 維持して手動更新 / 外して SHA と Git 履歴で追跡。
- 対象: GitHub Actions のみ / Cargo も同時に導入。

## 検討内容

月次はレビューの割り込みを減らせるが、上流の修正や runner 変更への追従が遅れる。現在の対象は 3 action であり、週次でもグループ化と open PR 上限でレビュー待ちの本数を抑えられる。major 更新も対象に含めるため、グループの CI が失敗した場合は各 action の差分を確認する必要がある。

Issue 本文は日付が落ちる可能性を挙げていたが、調査時点の [VersionCommenter](https://github.com/dependabot/dependabot-core/blob/main/github_actions/lib/dependabot/github_actions/file_updater/workflow_updater/version_commenter.rb) はコメント末尾が旧 version と一致する場合に置換する。`# v1 (2026-08-05 時点)` はその条件を満たさず、日付が自動更新される保証もない。[上流テスト](https://github.com/dependabot/dependabot-core/blob/main/github_actions/spec/dependabot/github_actions/file_updater_spec.rb) でも、文の途中の version や旧 SHA に tag が無い場合にコメントが更新されないケースを扱っている。これはソース確認であり、本リポジトリの bot PR で確認した結果ではない。

Cargo は製品の依存解決と MSRV に影響する。GitHub Actions の更新運用を先に確認し、Cargo の grouping、互換性、検証負荷は別途決める。

## 決定

- `github-actions` ecosystem、`directory: "/"`、`schedule.interval: "weekly"` を設定する。
- `groups.github-actions.patterns: ["*"]` で更新をまとめ、version update の open PR 上限を 1 本にする。major 更新を除外しない。
- bot の定期更新 PR は Issue 起点の作業と区別し、起点 Issue と note は要求しない。レビューは直列、merge は人間が行う。
- `dtolnay/rust-toolchain` の末尾コメントは `# v1` とし、PR #20 の日付コメント規約を引き継がない。SHA 自体は変更しない。
- Cargo ecosystem は今回は追加しない。

## 理由

SHA 固定を続けながら、更新の発見と PR 作成を自動化できる。週次・グループ化・上限 1 本で追従速度とレビュー負荷を調整する。固定時点は SHA と Git 履歴に残るため、手動の日付更新を重複管理しない。

## 影響

- 通常の Issue 駆動タスクとの境界は `development-loop.md` に記載する。
- tag コメントは補助情報であり、自動更新を保証しない。更新 PR で SHA と tag の対応を確認する。
- 設定は既定ブランチへ入ってから有効になる。初回は設定受理・更新チェックの実行、更新 PR の SHA とコメント、bot PR の CI を確認する。設定追加 PR の CI 成功だけではこの確認を代替しない。
- security updates / alerts の設定、権限追加、自動 merge は本件に含めない。

## 後から見直す条件

- グループ化した更新の切り分けやレビュー負荷が大きくなった場合。
- Cargo の更新運用や release workflow の追加を扱う場合。
- Dependabot のコメント更新方式が変わった場合。

## 参考

- [Dependabot options reference](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference)
- [About the dependabot.yml file](https://docs.github.com/en/code-security/concepts/supply-chain-security/about-the-dependabot-yml-file)

## 2026-09-09 追記: 実動確認の追跡先

PR #29 のレビューを受け、merge 後の設定受理・更新チェック・更新 PR の内容・bot PR の CI は [Issue #30](https://github.com/kiyohara/bizdate/issues/30) で追跡する。PR #29 は実行ルールどおり `Closes #21` を維持するが、#21 の自動 close は実動確認の完了を意味しない。未確認事項が残る間は #30 を open で維持する。

`dtolnay/rust-toolchain` は同じ `v1` tag の移動を追うため、更新 PR の有無だけで判断せず、上流 tag と workflow の SHA を照合する。一致は現時点の差分なし、不一致で PR が無い場合は更新チェックの完了・cooldown・ignore・open PR 上限・grouping・log を調べる。未検知が残る場合は原因と対処を #30 に記録する。上流 tag が動いていない時点で、将来の移動検知を実証したとは扱わない。

## 2026-09-09 追記: 既定 cooldown 3 日

Issue #30 の実動確認で、`.github/dependabot.yml` に `cooldown` を設定していないにもかかわらず、update job の log に cooldown filter の動作が記録されていることを確認した。

```text
Initializing cooldown filter
Days since release : 35 (cooldown days 3)
```

[Dependabot options reference](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference) によれば、`cooldown` が未設定でも version update には既定で 3 日の cooldown が適用される。新しい version は release から 3 日経つまで version update の対象にならない。security update はこの既定の対象外である。既定化は [2026-07-14 の changelog](https://github.blog/changelog/2026-07-14-dependabot-version-updates-introduce-default-package-cooldown/) で告知されている。

このため、上流が新 version を出しても直後は更新 PR が出ない。「更新 PR が出ない」原因を切り分けるときは、更新チェックの完了、ignore、open PR 上限、grouping に加えて既定 cooldown も候補に含める。

`cooldown` は明示設定していない。既定のままとし、window を変える必要が生じた時点で改めて判断する。「決定」の内容は変更しない。観測した log と run URL は [Issue #30 のコメント](https://github.com/kiyohara/bizdate/issues/30#issuecomment-5593296663) にある。既定値は上流の変更で動きうるため、恒久の仕様として扱わない。

## 2026-09-23 追記: release workflow の導入に伴う再判断

「後から見直す条件」の「Cargo の更新運用や release workflow の追加を扱う場合」に当たる。Issue #38 で release workflow と配布 archive を導入した（[0022](0022-release-workflow.md)）。Cargo の version updates、Dependabot alerts（security alerts）、Dependabot security updates を分けて判断し直す。あわせて、GitHub Actions の version updates と、dist が生成する `release.yml` の関係を決める。

### 判断

| 対象 | 採否 | 理由 | 実装 |
|---|---|---|---|
| Cargo の version updates | 採用 | 配布を始めると、依存の修正（TLS を担う `rustls` / `ring`、root store を埋め込む `webpki-roots` を含む）は再ビルドと再リリースでしか利用者に届かない（[0016](0016-distribution-contract.md) の TLS trust）。更新に気付く経路を自動化する価値が上がった。上の「検討内容」で挙げた検証負荷は、4 target の native CI（#37）と、PR ごとに archive と third-party 表記を検証する Release workflow（#38）で受け止められる。許可していないライセンスの依存が入れば build job の cargo-about が止まり、MSRV を超える更新は MSRV の toolchain で回す CI が止める | 別 Issue（[#63](https://github.com/kiyohara/bizdate/issues/63)）で行い、本件には含めない。grouping、open PR 上限、major 更新の扱い、更新 PR のレビュー観点はその Issue で決める |
| Dependabot alerts | 採用 | 依存に既知の脆弱性が公表されたとき、現状は気付く経路が無い。alerts は通知だけで PR を作らないため、運用の負担が小さい | repository settings で有効にする。agent は settings を変えず、有効化はユーザーが行う |
| Dependabot security updates | 採用 | 修正版への更新 PR を、version updates の cooldown と open PR 上限を待たずに作れる。配布 binary は修正を再リリースで届けるしかなく、着手までの時間を縮める | repository settings で有効にする（alerts が前提）。有効化はユーザーが行う。更新 PR は bot PR と同じ流れでレビューし、merge は人間が行う。grouping の要否は Cargo の version updates と同じ Issue で決める |

Cargo の version updates の実装 Issue（#63）は、配布準備（#36〜#40）の依存にしない単発の作業である。`progress.md` の索引には載せない（#30 と同じ扱い）。

### `release.yml` と GitHub Actions の version updates

`release.yml` は dist の生成物で、第三者 action の SHA は `dist-workspace.toml` の `github-action-commits` から入る。Dependabot は `.github/workflows/` にある `release.yml` も更新する。更新 PR が `release.yml` の SHA だけを変えると、Release workflow の `plan` job が生成結果との差分で失敗する。

- 決定: `release.yml` を Dependabot の対象から外さない。更新 PR で Release workflow が差分により失敗したら、`github-action-commits` を同じ SHA と tag へ揃え、`dist generate --check` が通ることを確かめた commit を更新 PR に足してから merge する。手順は `doc/guidelines/development-loop.md` の「Dependabot が作成する更新 PR」に置く。
- 理由: 対象から外すと、`release.yml` の action だけが更新されないまま、公開の日まで気付かない。差分での失敗は、揃える作業が要ることを更新 PR の時点で示す。
- 影響: この場合に限り、bot PR に人間（または依頼を受けた agent）の commit が加わる。

### 見直す条件

- Cargo の更新 PR のレビュー負荷や、MSRV 起因の失敗が大きい場合。grouping、頻度、major 更新の扱いを見直す。
- dist が、生成する workflow の action を他の workflow と揃えて更新する仕組みを持った場合。`release.yml` の扱いを見直す。
- security updates の PR が version updates の PR と衝突し、レビューの手戻りが目立つ場合。

## 2026-09-23 追記: Cargo の version updates の設定（#63）

上の再判断で採用した Cargo の version updates を、[Issue #63](https://github.com/kiyohara/bizdate/issues/63) で `.github/dependabot.yml` に入れる。更新頻度、grouping、open PR 上限、major 更新、MSRV を超える更新、security updates の grouping、実動確認の追跡先を決める。

### 一次資料で確かめたこと

- options（`allow`、`groups` の `update-types` / `applies-to`、`open-pull-requests-limit`、既定 cooldown）は [Dependabot options reference](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference) の source（[github/docs](https://github.com/github/docs) の `content/code-security/reference/supply-chain-security/dependabot-options-reference.md`、commit `7922319`、2026-09-23）で確かめた。
  - version updates は既定で manifest に明示した依存だけを対象にする。`allow` の `dependency-type: all` は、`cargo` では直接の依存の依存（間接の依存）も含める。
  - `groups` は既定で version updates に効く（`applies-to` 未指定時は `version-updates`）。`update-types` は `minor` / `patch` / `major` で group を絞る。group に入らない更新は依存ごとの PR になる。
  - `open-pull-requests-limit` は version updates にだけ効き、security updates の PR は数えない。既定は 5。
  - 既定の cooldown（3 日）は version updates に効き、security updates には効かない（上の 2026-09-09 追記と同じ）。
- Dependabot の Cargo の挙動は [dependabot-core](https://github.com/dependabot/dependabot-core) の source（commit `f8de02f`、2026-09-23）で確かめた。本リポジトリの bot PR で確かめた結果ではない。
  - 更新の分類: `Cargo::Version.update_type` は、1.0 未満では Cargo の SemVer 規則に従い、`0.y` の `y` の変化（`0.0.z` では `z` の変化）を `major` と数える（`cargo/lib/dependabot/cargo/version.rb`）。`jiff` 0.2 → 0.3 や `encoding_rs` 0.8 → 0.9 は major として扱われる。
  - MSRV: 最新 version の探索は `rust-version` で絞らない。`UpdateChecker` は `LatestVersionFinder#latest_version` を `language_version` 無しで呼び、crates.io の release が持つ `rust` の値は使われない（`cargo/lib/dependabot/cargo/update_checker.rb`、`update_checker/latest_version_finder.rb`）。ただし `Cargo.lock` がある場合、更新先はこの最新 version ではなく、`VersionResolver` が決める解決可能な version である。`FilePreparer` が要求を `>= <現行>, <= <最新>` に置き換えた manifest を作り、Dependabot の image の cargo で `cargo update -p <spec>` を実行して、結果の `Cargo.lock` から version を読む（`update_checker/version_resolver.rb`、`update_checker/file_preparer.rb`。調査時点の image は `rust:1.98.0`）。manifest の `edition` と `rust-version` はそのまま渡る。
  - `Cargo.lock` の書き換え: `LockfileUpdater` は manifest の要求を `=<version>` に固定し、`cargo update -p <name>:<previous_version>` を実行する。line が動かなかった場合にだけ `--precise` で再試行する（`file_updater/lockfile_updater.rb`）。
  - manifest の要求は、新しい version を既存の要求が許さない場合にだけ上げる（`RequirementsUpdateStrategy::BumpVersionsIfNecessary`）。互換のある更新は `Cargo.lock` だけの差分になる。

### 判断

| 項目 | 採否 | 理由 |
|---|---|---|
| ecosystem と頻度 | `cargo`、`directory: "/"`、weekly | `github-actions` と揃える。依存は直接 5 crate、`Cargo.lock` 全体でも数十 crate であり、週次でも grouping で本数を抑えられる。修正の利用者への到達は再リリースが律速であり、日次にする利益が小さい |
| 対象 | 間接の依存も含める（`allow: dependency-type: all`） | TLS を担う `rustls` / `ring`、root store の `webpki-roots` は `ureq` 経由の間接の依存である。既定の直接の依存だけでは、`ureq` の更新が出るまで `Cargo.lock` の中で古いまま残る |
| grouping | minor / patch を group `cargo` の 1 本にまとめる。major はまとめない | 互換のある更新は `Cargo.lock` だけの差分が大半で、まとめてもレビューの負担が小さい。major（上記のとおり `0.y` の変化を含む）はコード修正が要ることがあり、まとめると 1 つの失敗で他の更新も止まる。依存ごとの PR に分けて切り分けやすくする |
| major 更新 | 対象に含める（`ignore` しない） | `github-actions` と同じ。除外すると、互換の無い更新に気付く経路が無くなる |
| open PR 上限 | 2 本 | `github-actions` と同じ 1 本では、修正待ちの major の PR が 1 本残るだけで group の PR も出なくなる。group 1 本と major 1 本を並べられる数にする。レビューは直列のまま行う |
| cooldown | 既定（3 日）のまま | `github-actions` と揃える。security updates には効かないため、脆弱性の修正は待たない |
| security updates の grouping | まとめない（`applies-to: security-updates` の group を置かない） | security updates は open PR 上限を受けず、依存ごとに出る。依存が少なく同時に出ることは稀で、1 件ずつなら失敗の切り分けと revert が容易である。まとめると、修正版の 1 つが MSRV などで落ちたときに他の修正まで止まる |

### MSRV を超える更新

更新先は、Dependabot の image の cargo が `rust-version` 付きの manifest を解決した結果で決まる（上記）。edition 2024 の resolver（v3）は `rust-version` を考慮した解決（`incompatible-rust-versions = "fallback"`）を既定にしている。

- 確かめたこと: Compose の cargo 1.98 で、`rust-version = "1.64"`、`resolver = "3"`、要求 `clap = ">= 4.0.0, <= 4.6.0"` の crate を解決すると、`clap` は MSRV に合う 4.3.24 になった。cargo は `Adding clap v4.3.24 (available: v4.6.7, requires Rust 1.85)` と出力した。`resolver = "2"` では 4.6.0 になった。範囲内に MSRV に合う version があれば、直接の依存でもそれが選ばれる。
- 見込み（未確認）: Dependabot でも同じ解決になるなら、MSRV を超える更新は CI の失敗ではなく、「MSRV に合う version までの更新 PR が出る」または「更新 PR が出ない」形で現れる。範囲内に MSRV に合う version が無い場合（`fallback` は合う version が無ければ合わない version を選ぶ）や、security updates で修正版が MSRV を超える場合は、MSRV を超える crate が入った更新 PR が作られ得る。本リポジトリの bot PR では確かめていない（[#73](https://github.com/kiyohara/bizdate/issues/73) で確かめる）。
- 検出: CI は MSRV と同じ toolchain（`ci.yml` の `RUST_TOOLCHAIN`）で `--locked` の clippy / test / build を回す。Release workflow の build も同じ toolchain を使う（`.github/build-setup.yml`）。MSRV を超える crate が `Cargo.lock` に入ると cargo が `rust-version` の不足で失敗する。更新 PR の CI 成功を merge の条件にすることで、MSRV を超える更新は main に入らない。
- 据え置きへの気付き: 更新 PR が出ないまま MSRV のために古い version に留まる場合は、PR からは気付けない。MSRV は最新 stable に揃える方針（[0014](0014-dependency-crates-and-toolchain.md)）であり、Rust の新しい stable が出た時点で MSRV を上げれば据え置きは解ける。据え置きの有無は、Compose で `cargo update --dry-run` を実行したときの `requires Rust` の表示で確かめられる。
- 処置: CI が MSRV の不足で落ちた更新 PR は merge しない。MSRV を上げる Issue を起こし、`doc/guidelines/development-command-guidelines.md` の「MSRV を上げるとき」に従って別 PR で上げる。更新 PR に MSRV の変更を足さない。MSRV の変更は開発 image と cloud session の environment cache にも及び、bot の更新と分けてレビューするためである。MSRV を上げても、Dependabot が更新 PR を rebase するのは次の schedule の実行時か、target branch への push で PR が conflict したときである（options reference の `rebase-strategy`）。急ぐ場合は更新 PR に `@dependabot rebase` をコメントして rebase させ、CI を回し直す。
- 上限を塞ぐ場合: MSRV を上げるまでの間に更新 PR が open PR 上限を塞ぐ場合、または group の PR に MSRV を超える更新が混ざって互換のある他の更新まで止まる場合は、該当の依存を `ignore` へ理由のコメント付きで足す PR を出し、MSRV を上げたら速やかに外す。`ignore` は security updates にも効くため、`versions` で該当 version の範囲に絞り、`update-types` での除外にしない。`@dependabot ignore` のコメントは設定がリポジトリに残らないため使わない（`@dependabot rebase` は設定を残さない一度きりの操作なので使ってよい）。
- 手順は `doc/guidelines/development-loop.md` の「Dependabot が作成する更新 PR」に置く。

### 実動確認の追跡

設定が既定ブランチへ入った後の確認（設定の受理、更新チェックの実行、間接の依存が対象に入ること、最初の Cargo 更新 PR の分かれ方と差分、その PR での CI と Release workflow の結果）は [Issue #73](https://github.com/kiyohara/bizdate/issues/73) で追う。#63 は PR の merge で close されるため、#21 と #30 の分け方に揃える。設定追加 PR の CI 成功だけでは実動確認済みとしない。Dependabot alerts と security updates の有効化の確認も #73 に含める。

### 影響

- `.github/dependabot.yml` に `cargo` の設定を足す。`github-actions` の設定は変えない。
- `doc/guidelines/development-loop.md` の「Dependabot が作成する更新 PR」を、Cargo の更新 PR にも当てはめ、レビュー観点と MSRV を超える更新の処置を足す。
- Dependabot alerts と Dependabot security updates の有効化は、ユーザーが repository settings で行う。agent は settings を変えない。
