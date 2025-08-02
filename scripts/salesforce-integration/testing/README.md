# Salesforce Integration Testing Scripts

このディレクトリには、Salesforce
OAuth2認証とリード作成をテストするためのスクリプトが含まれています。

## セキュリティに関する重要な変更

パスワードのハードコーディングを削除しました。すべての認証情報は環境変数から読み込まれます。

## 使用方法

### 環境変数の設定

以下のスクリプトを実行する前に、環境変数を設定してください：

```bash
export MARKMAIL_TEST_EMAIL="yusuke.sato@engineers-hub.ltd"
export MARKMAIL_TEST_PASSWORD="your_password_here"
```

または、スクリプト実行時に直接指定：

```bash
MARKMAIL_TEST_PASSWORD=your_password python3 login_and_oauth.py
```

### 各スクリプトの説明

1. **login_and_oauth.py**

   - ログインしてOAuth2認証状態を確認します
   - 認証されていない場合は認証URLを表示します

   ```bash
   MARKMAIL_TEST_PASSWORD=your_password python3 login_and_oauth.py
   ```

2. **oauth2_flow.py**

   - OAuth2認証フロー全体をテストします
   - 認証URLを生成し、ブラウザで開くように案内します

   ```bash
   MARKMAIL_TEST_PASSWORD=your_password python3 oauth2_flow.py
   ```

3. **complete_oauth_callback.py**

   - コールバックURLを手動で処理します（通常は不要）

   ```bash
   MARKMAIL_TEST_PASSWORD=your_password python3 complete_oauth_callback.py 'callback_url'
   ```

4. **submit_form_test.py**

   - ローカル環境でフォーム送信→Salesforceリード作成をテストします
   - 認証は不要（公開フォームのため）

   ```bash
   python3 submit_form_test.py
   ```

5. **submit_form_test_dev.py**

   - 開発環境（dev.markmail.engineers-hub.ltd）でフォーム送信をテストします

   ```bash
   python3 submit_form_test_dev.py
   ```

6. **test_oauth_curl.sh**
   - OAuth2認証URLを生成するシェルスクリプト
   - 注意：このスクリプト内のClient IDは古いもので、現在は使用されていません
   ```bash
   ./test_oauth_curl.sh
   ```

## 注意事項

- パスワードは決してコミットしないでください
- 本番環境のパスワードは特に慎重に扱ってください
- 現在のSalesforce OAuth設定は `backend/.env` ファイルにあります
