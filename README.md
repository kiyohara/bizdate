# bizdate

`bizdate` は、指定した日付（省略時はコマンド実行日）が、その月の最初または最後の業務日かを判定するコマンドラインツールです。曜日に加え、日本の祝日を扱います。

主用途は、cron やシェルから月次処理のゲートとして使うことです。

```sh
bizdate fetch-holidays
bizdate last && monthly-job
```

詳細な利用手順とインストール方法は、実装と配布手段が固まってから追記します。

## ライセンス

[MIT License](LICENSE) で公開しています。

## 開発者向けドキュメント

- ドキュメント配置の入口: [`doc/README.md`](doc/README.md)
- AI agent / 開発者向け共通入口: [`AGENTS.md`](AGENTS.md)
