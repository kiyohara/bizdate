# 作業ブランチメモ

- ブランチ: `claude/drive-next-issue-5ndbgx`
- PR: #79
- 最終更新: 2026-09-24

## 目的

Issue #39。dist の Homebrew installer を有効にし、既存 tap `kiyohara/homebrew-tap` の `Formula/bizdate.rb` を tag push の release workflow から更新できるようにする。Formula の起動テスト、prerelease と巻き戻しの抑止、失敗時の再開、tap 書き込みの認証と設定手順を整える。実際の tap への公開は行わない（スコープ外）。

## 現在の状況

- cloud session（Claude Code on the web）で、`drive-issue-to-reviewed-pr` の P1 として進めている。ユーザーの指示は「次に実行できそうな issue を特定して、一気通貫で drive して」。
- Issue の選定: open Issue は #39 / #40 / #73、open PR は無い。#39 は依存の #65（PR #76）が merge 済みで、`progress.md` の配布準備の索引順でも次に当たり、前提は #36 で確定済み。#40 は #39 に依存し、#73 は Dependabot の更新 PR 待ちのため外した。
- tap の現状（2026-09-24、read-only の clone で確認。HEAD `35d25c6`、2026-09-03）: `Casks/slapex.rb` と `README.md` だけで、`Formula/` は無い。README は slapex の Cask だけを説明している。

## 決定事項

- tap への書き込みは dist の builtin の publish job（`publish-jobs = ["homebrew"]`）を使わず、custom の publish job（`.github/workflows/publish-homebrew.yml`）で行う。builtin は Formula に test を足せず、`brew style --fix ... || true` で失敗を隠し、tap の Formula の version を見ずに上書きする（非線形な release で巻き戻る）ため。Formula 自体は dist の installer が生成したものを使う。経緯は decision log 0025。
- PR でも tag push でも、Formula は `prepare-homebrew-formula.sh` で同じく検査し、`test do` を足す。PR では `release-verify` の `homebrew` job が、url をこの run の archive へ差し替えて native runner で install と `brew test` を行う。
- `Cargo.toml` に `homepage` を足した（dist が Formula の `homepage` に使う。無いと WARN）。
- cloud session で `release-tools` service は使えない（`compose.cloud.yaml` に override が無く、container 内の curl が proxy の CA を検証できない）。代わりに、host で dist 0.32.0 と cargo-about 0.9.2 の公開 binary を取得し、`Dockerfile` / `install-cargo-about.sh` と同じ固定 sha256 と照合してから dev container へ read-only で mount して実行した。repo の設定は変えていない。

## 次にやること

- [x] dist の Homebrew installer と custom publish job を設定し、`release.yml` を作り直す
- [x] Formula の検査と test 追加、PR での install 検証、tap への publish の script を書く
- [ ] PR CI で、3 target の Formula の検査と install / `brew test` / 配置を確かめる
- [ ] `distribution.md`、decision log 0025 と index、`development-command-guidelines.md`、Copilot 指示を更新する
- [ ] `progress.md` の DIST-04 を更新する
- [x] PR を作成し、note を採番する

## 検証

（CI の結果とあわせて記録する）

## リスク・ブロッカー

- 公開 tap からの `brew install` / `upgrade`、tap への実 push、`HOMEBREW_TAP_TOKEN` の設定は未実施（スコープ外。#40 の初回公開で確かめる）。

## セッションログ

- 2026-09-24: #39 に着手。dist の Homebrew 設定と custom publish job、Formula の検査・test 追加・install 検証・publish の script を追加した。
