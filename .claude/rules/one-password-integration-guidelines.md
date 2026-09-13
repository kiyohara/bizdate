# 1Password 連携

- 詳細は `doc/guidelines/one-password-integration-guidelines.md`
- 1Password 連携の有無を事前に判定せず、preflight やゲートも置かず、各 guideline のコマンドと tool をそのまま実行する。
- `git` / `gh` / `op` の実行や MCP server の起動が失敗し、出力に 1Password や承認の文言がある、または原因を判断できないときは、承認ダイアログへの応答が間に合わなかった可能性を疑う。再試行や回避策へ進まず、正本に従って中断し、失敗した操作・エラー要旨・未反映の変更を報告して選択肢を提示する。
- 別の 1Password 連携経路への自動切り替え、署名の無効化、鍵・remote・credential helper の変更はしない。
