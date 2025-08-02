# Salesforce OAuth2 トラブルシューティングガイド

## 概要

Salesforce OAuth2認証実装時に遭遇した問題と解決方法をまとめたドキュメントです。

## 1. 遭遇した問題と解決方法

### 1.1 OAUTH_APPROVAL_ERROR_GENERIC エラー

**問題**

```
OAuth エラーのため、あなたを認証できません。詳細は、Salesforce システム管理者にお問い合わせください。
OAUTH_APPROVAL_ERROR_GENERIC : 認証中、予期せぬエラーが発生しました。もう一度お試しください。
```

**原因**

1. 接続アプリケーションが異なる組織（Production/Sandbox）に作成されていた
2. Client IDが正しい組織に存在しない

**解決方法**

1. 現在接続している組織を確認
   ```bash
   sf org display --target-org <org-alias> --json | jq -r '.result.instanceUrl'
   ```
2. 正しい組織に接続アプリケーションを作成
3. Sandbox環境の場合は、認証URLを調整

### 1.2 missing required code challenge エラー

**問題**

```
http://localhost:3000/api/crm/oauth/salesforce/callback?error=invalid_request&error_description=missing+required+code+challenge
```

**原因** 接続アプリケーションで「サポートされる認証フローに Proof Key for Code
Exchange (PKCE) 拡張を要求」が有効になっていた

**解決方法**

1. Salesforce設定 → アプリケーションマネージャー → 接続アプリケーション編集
2. 「サポートされる認証フローに Proof Key for Code Exchange
   (PKCE) 拡張を要求」のチェックを外す
3. 保存

### 1.3 認証ヘッダーがありません エラー

**問題** コールバックURLにアクセスすると「認証ヘッダーがありません」エラーが発生

**原因**
コールバックエンドポイントが認証が必要な保護されたルートに配置されていた

**解決方法**

```rust
// backend/src/api/mod.rs
// コールバックエンドポイントを公開ルートに移動
let public_routes = Router::new()
    // ... 他のルート
    .route(
        "/api/crm/oauth/salesforce/callback",
        get(crm_oauth::salesforce_auth_callback),
    );
```

### 1.4 Production/Sandbox環境の混在

**問題**

- `login.salesforce.com`でアクセスしようとしたが、実際はSandbox環境だった
- 逆に、Sandbox URLでProduction環境にアクセスしようとした

**原因** 環境設定の不一致

**解決方法**

1. 組織のインスタンスURLを確認
2. 環境変数を適切に設定

   ```env
   # Sandbox環境の場合
   SALESFORCE_IS_SANDBOX=true
   SALESFORCE_AUTH_URL=https://<instance>.my.salesforce.com/services/oauth2/authorize
   SALESFORCE_TOKEN_URL=https://<instance>.my.salesforce.com/services/oauth2/token

   # Production環境の場合
   SALESFORCE_IS_SANDBOX=false
   SALESFORCE_AUTH_URL=https://login.salesforce.com/services/oauth2/authorize
   SALESFORCE_TOKEN_URL=https://login.salesforce.com/services/oauth2/token
   ```

## 2. 作成したテストツール

### 2.1 フォーム送信テストツール

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/submit_form_test.py`

**用途**: フォーム送信→Salesforceリード作成のテスト

### 2.2 OAuth2認証フローテストツール

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/oauth2_flow.py`

**用途**: OAuth2認証フローの実行と状態確認

### 2.3 OAuth2コールバック処理ツール

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/complete_oauth_callback.py`

**用途**: コールバックURLを手動で処理（未使用）

### 2.4 ログイン・認証確認ツール

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/login_and_oauth.py`

**用途**: ログインとOAuth2認証状態の確認

### 2.5 トークン更新スクリプト

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/backend/refresh_sf_token.sh`

**用途**: Salesforce CLIトークンの更新と.envファイルへの反映

### 2.6 OAuth URLテストスクリプト

**パス**:
`/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/test_oauth_curl.sh`

**用途**: OAuth2認証URLの生成

### 2.7 生成されたOAuth URL保存ファイル

- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/oauth_url.txt`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/oauth_url_sandbox.txt`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/oauth_test_url.txt`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/scripts/salesforce-integration/testing/oauth_new_url.txt`

## 3. 重要な設定ポイント

### 3.1 Salesforce接続アプリケーション設定

1. **基本情報**

   - 接続アプリケーション名: MarkMail Integration Sandbox
   - API名: MarkMail_Integration_Sandbox

2. **OAuth設定**

   - ✅ OAuth設定を有効化
   - コールバックURL: `http://localhost:3000/api/crm/oauth/salesforce/callback`

3. **OAuthスコープ**

   - ✅ API を使用してユーザーデータを管理 (api)
   - ✅ いつでも要求を実行 (refresh_token, offline_access)

4. **フローの有効化**

   - ✅ 認証コードおよびログイン情報フローを有効化
   - ❌ PKCEを要求しない

5. **セキュリティ**
   - ✅ Web サーバーフローの秘密が必要
   - ✅ 更新トークンフローの秘密が必要

### 3.2 環境変数設定

```env
# Salesforce OAuth設定
SALESFORCE_CLIENT_ID=<Your_Client_ID>
SALESFORCE_CLIENT_SECRET=<Your_Client_Secret>
SALESFORCE_REDIRECT_URI=http://localhost:3000/api/crm/oauth/salesforce/callback
SALESFORCE_IS_SANDBOX=true
SALESFORCE_AUTH_URL=https://<instance>.my.salesforce.com/services/oauth2/authorize
SALESFORCE_TOKEN_URL=https://<instance>.my.salesforce.com/services/oauth2/token
```

## 4. デバッグコマンド

### 組織情報確認

```bash
# 現在の組織情報を確認
sf org display --target-org <org-alias> --json | jq -r '.result | {instanceUrl: .instanceUrl, username: .username, orgId: .id}'

# すべての組織を確認
sf org list --all
```

### データベーストークン更新

```bash
# CRM統合のトークンを確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT id, user_id, provider, credentials->>'access_token' as access_token FROM crm_integrations;"

# トークンを更新
NEW_TOKEN=$(cat /tmp/sf_token.txt)
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "UPDATE crm_integrations SET credentials = jsonb_set(credentials, '{access_token}', '\"$NEW_TOKEN\"') WHERE user_id = '<user_id>';"
```

## 5. 今後の検討事項

1. PKCEサポートの実装（セキュリティ向上のため）
2. エラーメッセージの詳細化
3. 環境自動判定機能の実装（Production/Sandbox）

## 6. 参考リンク

- [Salesforce OAuth 2.0 Web Server Flow](https://help.salesforce.com/s/articleView?id=sf.remoteaccess_oauth_web_server_flow.htm)
- [Connected Apps](https://help.salesforce.com/s/articleView?id=sf.connected_app_overview.htm)
