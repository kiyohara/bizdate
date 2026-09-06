<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# 開発ホスティングと CI 基盤

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `doc/design/concept.md`, `doc/design/cli-interface.md`

## 背景

仕様（0003–0009）が固まり、実装に進む段階に入った。コードのホスティング、タスク管理、PR レビュー、CI、配布自動化をどこで行うかを決めないと、開発ループ（[0012](0012-development-loop.md)）も配布方式（[0008](0008-language-and-distribution.md)）も具体化できない。

`bizdate` は公開を前提とした CLI ツールであり、単一バイナリの配布と Homebrew 連携を視野に入れている。

## 候補

- A: GitHub。Issue / PR / Actions / Releases が揃い、`dist` と Homebrew tap の自動化が前提としている環境
- B: GitLab。CI は強力だが、Rust 配布ツールチェーンの既定の連携先ではない
- C: セルフホスト（Gitea 等）。運用コストを自分で持つ
- D: ホスティングを持たず、ローカルリポジトリのみで進める

## 検討内容

配布自動化の観点が決定的である。[0008](0008-language-and-distribution.md) で採用した `dist` は GitHub Releases を前提とし、Homebrew tap への formula 更新も GitHub 上のリポジトリへ push する形で自動化される。個人 tap `kiyohara/homebrew-tap` は既に GitHub 上にあり、slapex がこれを使っている。同じ tap を共用できる。

CI は GitHub Actions で `fmt` / `clippy` / `test` を回す構成が Rust エコシステムの定石であり（0008 の調査）、`dtolnay/rust-toolchain` と `Swatinem/rust-cache` がそのまま使える。

レビューについては、GitHub Copilot code review をリポジトリ単位で効かせられる。AI agent によるレビューと組み合わせると、merge 前の確認を二重化できる。

タスク管理は GitHub Issue を使う。Issue 駆動の開発ループ（[0012](0012-development-loop.md)）は Issue 番号を作業単位の識別子にするため、ホスティングと分離できない。

ローカルのみで進める案は、CI が回らず、レビュー記録も残らない。公開を前提とする以上いずれ移すことになり、先送りの利点がない。

## 決定

- 開発ホスティングは GitHub とする。リポジトリは `kiyohara/bizdate`。
- リポジトリは **public** とする。ライセンスは [0013](0013-license-selection.md) で定める。
- タスク管理は GitHub Issue を使う。運用は [0012](0012-development-loop.md) に従う。
- CI は GitHub Actions とし、少なくとも `fmt` / `clippy` / `test` を実行する。
- 配布は GitHub Releases を起点とし、Homebrew tap は `kiyohara/homebrew-tap` を slapex と共用する。
- CI とリリースの workflow は、実装がある程度形になった段階で導入する。テスト対象のコードが無い状態では CI を先に置いても機能しないためである。

## 理由

`dist` と Homebrew による配布自動化が GitHub Releases を前提としており、既存の個人 tap もそこにある。Issue / PR / Actions / Releases が 1 か所に揃うことで、開発ループとレビュー体制を同じ場所に載せられる。

## 影響

- リポジトリ設定として Issues を有効化し、label を整備する。
- CI / リリースの workflow ファイルは `.github/workflows/` に置く。
- Copilot code review 用の指示は `.github/copilot-instructions.md` に置く（[0011](0011-ai-agent-lineup.md)）。
- public 前提のため、秘密情報をリポジトリに置かない。MCP server の token などは 1Password の secret reference で扱う。

## 後から見直す条件

- 非公開での開発が必要になった場合。
- GitHub Actions の実行枠や制約が開発速度を明らかに阻害する場合。
- 配布方式が変わり、GitHub Releases を前提としない手段に移る場合。
