# 開発コマンド実行

- 詳細は `doc/guidelines/development-command-guidelines.md`
- `cargo` などの開発コマンドは host で直接実行せず、`docker compose run --rm dev cargo ...` で実行する。
- `target/` は named volume にあるため、ビルドしたバイナリも container 内で実行する。
- host の toolchain で得た結果を、検証結果として報告しない。
- cloud session（Claude Code on the web）では SessionStart hook が Docker daemon を起動する。起動していなければ `doc/guidelines/cloud-session-guidelines.md` に従い、sandbox の `cargo` で代替しない。
