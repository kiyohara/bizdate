<!-- 記入前に doc/guidelines/decision-log-guidelines.md の「正本と参照の関係」を確認する。関連は decision log → spec の向きで書く。本ログは仕様の正本ではない。状態は proposed / decided / superseded / rejected のいずれか。index 未決表の open は使わない。 -->

# ライセンス選定

- 状態: decided
- 作成日: 2026-09-06
- 最終更新日: 2026-09-06
- 関連: `README.md`, `LICENSE`, `doc/design/decision-log/0008-language-and-distribution.md`, `doc/design/decision-log/0010-hosting-and-ci-platform.md`

## 背景

リポジトリを public として公開する（[0010](0010-hosting-and-ci-platform.md)）ため、公開時点でライセンスを確定する必要がある。[0008](0008-language-and-distribution.md) では「候補としては MIT が自然」としたが確定していなかった。

ライセンス未指定のまま公開すると法的には全権利留保となり、利用者が安心して使えない。

## 候補

- MIT License
- Apache License 2.0
- BSD-3-Clause
- ライセンス未指定

## 検討内容

- MIT: 最も簡潔で広く理解されている。許諾条件は著作権表示とライセンス文の保持のみ。特許条項は持たない。
- Apache-2.0: 明示的な特許ライセンス付与を備え企業利用で好まれるが、本文が長く NOTICE 運用の手間が増える。`bizdate` は日付判定と CSV 取得のみで、特許リスクが顕在化する領域ではない。
- BSD-3-Clause: MIT に近いが endorsement 禁止条項が加わる。MIT で足りる。
- 未指定: 公開方針と矛盾する。
- 依存予定のクレート（clap / chrono / csv / rustls 系）はいずれも寛容型であり、MIT 採用と矛盾しない。

同一の作者が公開している slapex も MIT を採用しており、揃えることで利用者から見た許諾条件が一貫する。

## 決定

- ライセンスは **MIT License** とする。
- copyright 表記は `2026 Tomokazu Kiyohara` とする。
- repo root に `LICENSE` を置き、`README.md` から参照する。

## 理由

単一バイナリとソースを公開し、手軽に使ってもらうことを重視する方針に対して、許諾条件が最小の寛容型ライセンスが最も整合するため。特許ライセンスの明示付与が必要となる利用形態は現時点で想定されない。

## 影響

- repo root に `LICENSE`（MIT、copyright `2026 Tomokazu Kiyohara`）を置く。
- `README.md` にライセンスの記載を追加する。
- ソースにライセンスヘッダや SPDX 識別子を付す場合は `MIT` を使う。
- `index.md` の未決事項から公開ライセンスの行を外す。

## 後から見直す条件

- 特許ライセンスの明示付与が求められる利用形態が出てきた場合に Apache-2.0 への変更を検討する。
- 取り込む第三者コードがコピーレフトなど別ライセンスを要求する場合。
