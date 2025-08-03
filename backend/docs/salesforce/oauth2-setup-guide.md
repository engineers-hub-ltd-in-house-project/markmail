# Salesforce OAuth2セットアップガイド

## 概要

MarkMailシステムでSalesforce OAuth2認証を使用するためのセットアップガイドです。

## 1. Salesforce側の設定

### 1.1 接続アプリケーションの作成

1. Salesforceの設定画面にアクセス
2. 「アプリケーションマネージャー」→「新規接続アプリケーション」
3. 以下の設定を行う：
   - **接続アプリケーション名**: MarkMail Integration
   - **API名**: MarkMail_Integration
   - **連絡先メール**: 管理者のメールアドレス
   - **OAuth設定を有効化**: チェック
   - **コールバックURL**:
     - 開発環境: `http://localhost:3000/api/crm/oauth/salesforce/callback`
     - 本番環境: `https://your-domain.com/api/crm/oauth/salesforce/callback`
   - **選択したOAuthスコープ**:
     - フルアクセス (full)
     - APIの有効化 (api)
     - 更新トークンの実行、オフラインアクセス (refresh_token, offline_access)

### 1.2 認証情報の取得

接続アプリケーションを作成後：

1. 「管理」をクリック
2. 「コンシューマーの詳細を参照」
3. 以下の情報をメモ：
   - **コンシューマー鍵** (Client ID)
   - **コンシューマーの秘密** (Client Secret)

## 2. MarkMail側の設定

### 2.1 環境変数の設定

`.env`ファイルに以下を追加：

```env
# Salesforce OAuth2設定
SALESFORCE_CLIENT_ID=your_consumer_key_here
SALESFORCE_CLIENT_SECRET=your_consumer_secret_here
SALESFORCE_REDIRECT_URI=http://localhost:3000/api/crm/oauth/salesforce/callback

# Redis設定（CSRF token保存用）
REDIS_URL=redis://localhost:6379
```

### 2.2 本番環境（AWS）での設定

AWS Secrets Managerで管理する場合：

```bash
# OAuth2設定用のシークレットを作成
aws secretsmanager create-secret \
  --name markmail-prod-salesforce-oauth \
  --secret-string '{
    "SALESFORCE_CLIENT_ID": "your_consumer_key_here",
    "SALESFORCE_CLIENT_SECRET": "your_consumer_secret_here",
    "SALESFORCE_REDIRECT_URI": "https://your-domain.com/api/crm/oauth/salesforce/callback"
  }' \
  --profile your-profile
```

## 3. OAuth2認証フロー

### 3.1 認証の開始

```bash
# 認証状態を確認
curl -X GET http://localhost:3000/api/crm/oauth/salesforce/status \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 認証を開始
curl -X GET http://localhost:3000/api/crm/oauth/salesforce/init \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

レスポンス例：

```json
{
  "auth_url": "https://login.salesforce.com/services/oauth2/authorize?...",
  "state": "random_csrf_token"
}
```

### 3.2 ユーザー認証

1. `auth_url`にブラウザでアクセス
2. Salesforceにログイン
3. アプリケーション認可画面で「許可」
4. コールバックURLにリダイレクトされる

### 3.3 認証完了の確認

```bash
# 認証状態を再確認
curl -X GET http://localhost:3000/api/crm/oauth/salesforce/status \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

成功時のレスポンス：

```json
{
  "is_authenticated": true,
  "expires_at": "2025-08-02T15:30:00Z",
  "instance_url": "https://your-instance.salesforce.com"
}
```

## 4. CRM統合の作成

OAuth2認証完了後、CRM統合を作成：

```bash
curl -X POST http://localhost:3000/api/crm/oauth/integration \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "salesforce",
    "settings": {
      "sync_enabled": true,
      "sync_interval_minutes": 30,
      "batch_size": 100,
      "field_mappings": []
    }
  }'
```

## 5. トラブルシューティング

### 5.1 認証エラー

**エラー**: `Invalid client credentials`

- **原因**: Client IDまたはClient Secretが正しくない
- **解決**: Salesforceの接続アプリケーション設定を確認

**エラー**: `Invalid redirect URI`

- **原因**: コールバックURLが登録されていない
- **解決**: Salesforceの接続アプリケーションにコールバックURLを追加

### 5.2 トークンエラー

**エラー**: `Token expired`

- **原因**: アクセストークンの有効期限切れ
- **解決**: 自動的にリフレッシュトークンで更新される（手動介入不要）

**エラー**: `Invalid refresh token`

- **原因**: リフレッシュトークンが無効化された
- **解決**: 再度OAuth2認証フローを実行

### 5.3 Redis接続エラー

**エラー**: `Failed to save authentication state`

- **原因**: Redisサーバーに接続できない
- **解決**: Redisサーバーが起動していることを確認
  ```bash
  docker-compose ps redis
  ```

## 6. セキュリティ考慮事項

1. **本番環境では必ずHTTPSを使用**
2. **Client SecretはSecrets Managerで管理**
3. **リフレッシュトークンは暗号化して保存（実装済み）**
4. **CSRFトークンは一度のみ使用可能（5分間有効）**
5. **アクセストークンは自動更新（期限5分前）**

## 7. 既存のCLI認証からの移行

既存のCLI認証を使用している場合：

1. OAuth2認証を実行
2. CRM統合設定が自動的にOAuth2を優先して使用
3. CLI認証は必要に応じて削除可能

```bash
# CLI認証の削除（オプション）
sf org logout --target-org your-org-alias
```
