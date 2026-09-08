<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# Dependabot による GitHub Actions の更新

- 状態: decided
- 作成日: 2026-09-09
- 最終更新日: 2026-09-09
- 関連: `.github/dependabot.yml`, `.github/workflows/ci.yml`, `doc/guidelines/development-loop.md`, [Issue #21](https://github.com/kiyohara/bizdate/issues/21)

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
