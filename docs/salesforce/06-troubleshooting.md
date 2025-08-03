# トラブルシューティングガイド

このドキュメントでは、Salesforce統合で発生する一般的な問題と解決方法を説明します。

## OAuth2認証の問題

### 問題: Missing_OAuth_Token エラー

**症状**:

```json
{
  "error": "Missing_OAuth_Token",
  "message": "OAuth token not found for this user and provider"
}
```

**原因と解決方法**:

1. **トークンが保存されていない**

   ```bash
   # データベースを確認
   docker exec -it markmail-postgres-1 psql -U postgres -d markmail \
     -c "SELECT * FROM crm_integrations WHERE user_id = 'your-user-id';"
   ```

2. **instance_urlが保存されていない**

   ```sql
   -- instance_urlがNULLかチェック
   SELECT user_id, provider, instance_url, expires_at
   FROM crm_integrations
   WHERE provider = 'salesforce';
   ```

3. **解決方法**
   - OAuth2認証を再実行
   - `exchange_code`メソッドがinstance_urlを正しく保存しているか確認

### 問題: invalid_client エラー

**症状**:

```json
{
  "error": "invalid_client",
  "error_description": "invalid client credentials"
}
```

**原因と解決方法**:

1. **Consumer Key/Secretの不一致**

   ```bash
   # 環境変数を確認
   echo $SALESFORCE_CLIENT_ID
   echo $SALESFORCE_CLIENT_SECRET

   # AWS Secrets Managerを確認
   aws secretsmanager get-secret-value \
     --secret-id markmail-dev-salesforce-secret \
     --query 'SecretString' \
     --output text | jq '.'
   ```

2. **Sandbox/Production環境の混在**
   - Sandbox用のCredentialsをProductionで使用していないか確認
   - `use_sandbox`フラグが正しく設定されているか確認

### 問題: redirect_uri_mismatch エラー

**症状**:

```
error=redirect_uri_mismatch&error_description=redirect_uri must match configuration
```

**原因と解決方法**:

1. **Callback URLの不一致**

   ```bash
   # Connected AppのCallback URLを確認
   # Setup → App Manager → Connected App → Edit

   # 登録すべきURL（複数可）:
   http://localhost:3000/api/crm/oauth/salesforce/callback
   https://dev.markmail.engineers-hub.ltd/api/crm/oauth/salesforce/callback
   ```

2. **末尾のスラッシュ**
   - URLは完全一致する必要がある
   - 末尾のスラッシュの有無も含めて確認

## API呼び出しエラー

### 問題: INVALID_SESSION_ID

**症状**:

```json
[
  {
    "message": "Session expired or invalid",
    "errorCode": "INVALID_SESSION_ID"
  }
]
```

**原因と解決方法**:

1. **アクセストークンの期限切れ**

   ```rust
   // 自動リフレッシュの実装確認
   if token_expired(&integration.expires_at) {
       let new_tokens = refresh_salesforce_token(
           &client,
           &integration.refresh_token,
           &oauth_settings
       ).await?;

       // 新しいトークンを保存
       update_tokens(&pool, &new_tokens).await?;
   }
   ```

2. **手動でトークンをリフレッシュ**
   ```bash
   # リフレッシュトークンを使用
   curl -X POST https://login.salesforce.com/services/oauth2/token \
     -d "grant_type=refresh_token" \
     -d "refresh_token=YOUR_REFRESH_TOKEN" \
     -d "client_id=YOUR_CLIENT_ID" \
     -d "client_secret=YOUR_CLIENT_SECRET"
   ```

### 問題: フィールドレベルセキュリティエラー

**症状**:

```json
[
  {
    "message": "Field Engineer_Skills__c is not accessible",
    "errorCode": "INVALID_FIELD_FOR_INSERT_UPDATE"
  }
]
```

**解決方法**:

1. **権限確認スクリプトを実行**

   ```bash
   python scripts/salesforce-integration/utilities/check_field_permissions.py
   ```

2. **Salesforceで権限を設定**
   - Setup → Object Manager → Lead
   - Fields & Relationships → カスタムフィールド
   - Field-Level Security → 編集可能に設定

### 問題: Bad_Id エラー

**症状**:

```json
[
  {
    "message": "Bad_Id: Invalid id value",
    "errorCode": "MALFORMED_ID"
  }
]
```

**原因と解決方法**:

1. **IDフォーマットの確認**

   - Salesforce IDは15文字または18文字
   - 例: `00Q5g00000XXXXXX`

2. **オブジェクトタイプの確認**
   - Lead IDは`00Q`で始まる
   - Contact IDは`003`で始まる
   - Account IDは`001`で始まる

## データベース関連の問題

### 問題: SQLxコンパイルエラー

**症状**:

```
error: error returned from database: column "instance_url" does not exist
```

**解決方法**:

1. **マイグレーションの実行**

   ```bash
   cd backend
   sqlx migrate run
   ```

2. **オフラインキャッシュの更新**
   ```bash
   cargo sqlx prepare
   git add .sqlx
   git commit -m "fix: Update SQLx offline cache"
   ```

### 問題: マイグレーション不整合

**症状**:

```
error: migration checksum mismatch
```

**解決方法**:

1. **マイグレーション履歴を確認**

   ```sql
   SELECT version, checksum, executed_at
   FROM _sqlx_migrations
   ORDER BY version;
   ```

2. **不整合の修正**
   ```sql
   -- 特定のマイグレーションを削除（注意が必要）
   DELETE FROM _sqlx_migrations
   WHERE version = 'problematic_version';
   ```

## AWS環境の問題

### 問題: ECSタスクが起動しない

**症状**: タスクが`STOPPED`状態になる

**デバッグ手順**:

1. **停止理由を確認**

   ```bash
   aws ecs describe-tasks \
     --cluster markmail-dev \
     --tasks arn:aws:ecs:region:account:task/task-id \
     --query 'tasks[0].stoppedReason'
   ```

2. **一般的な原因**
   - メモリ不足: タスク定義のメモリを増やす
   - ヘルスチェック失敗: `/health`エンドポイントを確認
   - Secrets Managerアクセスエラー: IAMロールを確認

### 問題: ALBヘルスチェック失敗

**症状**: Target unhealthy

**解決方法**:

1. **ヘルスチェックエンドポイントの確認**

   ```bash
   curl http://localhost:3000/health
   # 期待: {"status": "ok"}
   ```

2. **タイムアウト設定の調整**
   ```typescript
   // infrastructure/lib/stacks/alb-stack.ts
   targetGroup.configureHealthCheck({
     timeout: cdk.Duration.seconds(10), // 増やす
     interval: cdk.Duration.seconds(60), // 増やす
   });
   ```

### 問題: Secrets Managerアクセス拒否

**症状**:

```
AccessDeniedException: User is not authorized to perform: secretsmanager:GetSecretValue
```

**解決方法**:

1. **タスクロールの確認**

   ```bash
   aws iam get-role \
     --role-name markmail-dev-task-role \
     --query 'Role.AssumeRolePolicyDocument'
   ```

2. **ポリシーの追加**
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": [
           "secretsmanager:GetSecretValue",
           "secretsmanager:DescribeSecret"
         ],
         "Resource": "arn:aws:secretsmanager:*:*:secret:markmail-*"
       }
     ]
   }
   ```

## ローカル開発の問題

### 問題: Dockerコンテナが起動しない

**症状**: docker-compose up でエラー

**解決方法**:

1. **ポート競合の確認**

   ```bash
   lsof -i :5432  # PostgreSQL
   lsof -i :6379  # Redis
   lsof -i :8025  # MailHog
   ```

2. **コンテナのクリーンアップ**
   ```bash
   docker-compose down -v
   docker system prune -f
   docker-compose up -d
   ```

### 問題: cargo watchが動作しない

**症状**: ファイル変更が検出されない

**解決方法**:

1. **cargo-watchのインストール**

   ```bash
   cargo install cargo-watch
   ```

2. **手動実行**
   ```bash
   cargo run
   # または
   cargo watch -c -w src -w .env -x run
   ```

## パフォーマンスの問題

### 問題: API応答が遅い

**デバッグ手順**:

1. **ログでタイミングを確認**

   ```rust
   let start = std::time::Instant::now();
   let result = create_lead(&lead_data).await?;
   info!("Lead creation took: {:?}", start.elapsed());
   ```

2. **データベースクエリの確認**

   ```sql
   EXPLAIN ANALYZE
   SELECT * FROM crm_integrations
   WHERE user_id = 'uuid' AND provider = 'salesforce';
   ```

3. **インデックスの追加**
   ```sql
   CREATE INDEX idx_crm_integrations_user_provider
   ON crm_integrations(user_id, provider);
   ```

## ログとデバッグ

### ログレベルの設定

```bash
# 開発環境
RUST_LOG=debug cargo run

# 特定モジュールのみ
RUST_LOG=markmail::crm=debug cargo run
```

### リクエスト/レスポンスのログ

```rust
// backend/src/middleware/logging.rs
use tower_http::trace::TraceLayer;

app.layer(
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = ?request.method(),
                uri = ?request.uri(),
            )
        })
);
```

### Salesforce APIレスポンスの詳細ログ

```rust
match response.status() {
    StatusCode::OK => {
        let body = response.text().await?;
        debug!("Salesforce response: {}", body);
        serde_json::from_str(&body).map_err(|e| {
            error!("Parse error: {}, Body: {}", e, body);
            format!("Failed to parse response: {}", e)
        })
    }
    _ => {
        let status = response.status();
        let body = response.text().await?;
        error!("Salesforce error: Status={}, Body={}", status, body);
        Err(format!("API error: {}", body))
    }
}
```

## 緊急時の対応

### OAuth2トークンの手動設定

```sql
-- 緊急時のトークン手動更新
UPDATE crm_integrations
SET
  access_token = 'new-token',
  refresh_token = 'new-refresh-token',
  expires_at = NOW() + INTERVAL '2 hours',
  updated_at = NOW()
WHERE user_id = 'user-uuid' AND provider = 'salesforce';
```

### サービスの緊急再起動

```bash
# ローカル
docker-compose restart backend

# AWS
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --desired-count 0

aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --desired-count 1
```

## 関連ドキュメント

- [OAuth2認証](./01-authentication.md)
- [開発環境セットアップ](./03-development-setup.md)
- [テストツール](./04-testing-tools.md)
