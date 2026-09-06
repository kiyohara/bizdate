# Working Branch Notes 情報統制ルール

この文書は、`working-branch-notes/**/*.md` に書いてはいけない情報を定義する共通正本である。

`working-branch-notes/` の note は PR に含まれ、通常のリポジトリファイルとしてリモートへ送られる。本リポジトリは public であるため（`doc/design/decision-log/0010-hosting-and-ci-platform.md`）、作業メモであっても公開される前提で扱う。

## 基本方針

- 秘密情報、個人情報、非公開の第三者情報を書かない。
- 必要な文脈は、実値ではなくダミー値、プレースホルダ、変数名、抽象化した説明で残す。
- ログ、URL、設定値を貼る場合は、不要な識別子や認証情報を削ってから最小限だけ引用する。
- 判断に迷う情報は note に書かず、適切な管理場所やマネージドシークレットを参照する。

## 書いてはいけないもの

- 認証情報、API key、access token、refresh token、cookie、session id、秘密鍵
- `.env`、credentials、CI secrets、クラウド設定、管理画面などに置かれる実値
- `BEGIN ... PRIVATE KEY` 形式の鍵、証明書、署名用秘密値
- password、secret、token などの名前に続く実値
- 個人情報、個人を直接識別できる情報
- 非公開 URL、認証情報付き URL、署名付き parameter が付いた URL
- token、secret、cookie、session id、個人情報を含む可能性があるログ全文
- 長いランダム文字列など、資格情報や署名値に見える未確認の値
- 開発機のホスト名、ユーザー名、ローカル絶対 path など、公開する必要のない環境情報

## 代わりに書くもの

- `API_TOKEN_PLACEHOLDER`、`<redacted>`、`example.com` などのダミー値
- `ENV_VAR_NAME` のような変数名
- 個人名や組織名を出さない抽象的な説明
- エラー原因の理解に必要な stack trace の最小部分
- URL は host や path だけにし、query parameter は削除または redact したもの

## 編集時の確認

`working-branch-notes/**/*.md` を編集したら、push 前に少なくとも次を確認する。

- `password`、`secret`、`token`、`cookie`、`session`、`PRIVATE KEY` に続く実値がないか。
- 長いランダム文字列や署名値に見える文字列が残っていないか。
- URL に認証情報、署名、token、個人情報が含まれていないか。
- ログを必要以上に貼っていないか。

問題が見つかった場合は、実値を削除し、必要なら placeholder に置き換える。
