# 1Password 承認待ち失敗

- 詳細は `doc/guidelines/one-password-approval-failure.md`
- 1Password 連携操作の前にゲートや preflight（`op whoami`、`ssh-add -l` など）を置かない。通常どおり実行する。
- 承認待ちの失敗（タイムアウト / 拒否 / lock / プロンプト未到達など）を検知したら中断し、失敗した操作・エラー要旨・未反映の変更を報告して選択肢を提示する。
- 別の 1Password 連携経路へ自動 fallback しない。署名の無効化や鍵・remote・credential helper の無断変更もしない。
