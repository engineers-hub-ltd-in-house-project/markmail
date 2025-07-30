# AWS開発環境でのSalesforce連携設定（2025-07-30）

## 概要

MarkMailのAWS開発環境でSalesforce連携が動作していない問題を解決し、フォーム送信時のSalesforceリード自動作成機能を有効化しました。

## 重要な発見：Salesforce認証について

### クライアントIDとシークレットが不要だった理由

現在の実装では、以下の理由でSalesforce Connected
AppのクライアントIDとシークレットが不要でした：

1. **アクセストークンベースの認証**

   - `sf org display`コマンドで取得したアクセストークンを直接使用
   - アクセストークンとインスタンスURLがあれば、Salesforce APIを呼び出し可能

2. **コードの実装**

   ```rust
   // backend/src/services/crm_service/salesforce.rs
   let mut client = SalesforceClient::new(
       Some(std::env::var("SALESFORCE_CLIENT_ID")
           .unwrap_or_else(|_| "dummy_client_id".to_string())),
       Some(std::env::var("SALESFORCE_CLIENT_SECRET")
           .unwrap_or_else(|_| "dummy_client_secret".to_string())),
   );

   // アクセストークンを直接設定（重要）
   client.set_access_token(&self.config.access_token);
   client.set_instance_url(&self.config.instance_url);
   ```

3. **実際の動作**
   - 環境変数が「dummy」値でもAPI呼び出しは成功
   - OAuth認証フローを使用しない場合、クライアントIDとシークレットは不要

## 実施した作業

### 1. RDSへのCRM統合設定投入

AWS Systems Manager経由でBastion Hostを使用してRDSにデータを投入：

```sql
-- CRM統合設定を投入
INSERT INTO crm_integrations (
    id,
    user_id,
    provider,
    org_id,
    instance_url,
    credentials,
    settings,
    salesforce_settings,
    field_mappings,
    sync_enabled
) VALUES (
    '7d027cc5-1b3a-48e5-8410-15052fcf9130'::uuid,
    (SELECT id FROM users WHERE email = 'yusuke.sato@engineers-hub.ltd'),
    'salesforce',
    'markmail-org',
    'https://customization-computing-9993.my.salesforce.com',
    '{"access_token": "[YOUR_ACCESS_TOKEN]", "refresh_token": null}'::jsonb,
    '{"batch_size": 200, "sync_enabled": true, "sync_interval_minutes": 60}'::jsonb,
    '{"org_id": "markmail-org", "api_version": "v60.0", "instance_url": "https://customization-computing-9993.my.salesforce.com"}'::jsonb,
    '[]'::jsonb,
    true
)
ON CONFLICT (user_id, provider) DO UPDATE SET
    credentials = EXCLUDED.credentials,
    updated_at = NOW();
```

必要な情報の取得方法：

```bash
# アクセストークンの取得
sf org display --target-org markmail-org --json | jq -r '.result.accessToken'

# Bastion HostのインスタンスID
aws ec2 describe-instances --filters "Name=tag:Name,Values=markmail-dev-bastion" --profile [YOUR_PROFILE]

# RDSエンドポイント
aws rds describe-db-instances --query 'DBInstances[?contains(DBInstanceIdentifier,`markmail-dev`)]' --profile [YOUR_PROFILE]

# DBパスワード
aws secretsmanager get-secret-value --secret-id markmail-dev-db-secret --profile [YOUR_PROFILE]
```

### 2. 問題の特定と解決

#### 発生した問題

```
Salesforceリード作成エラー: INVALID_OR_NULL_FOR_RESTRICTED_PICKLIST
Java: 制限つき選択リスト項目の値が不適切: 実務経験豊富
```

#### 原因

- 開発環境用テストスクリプトで選択リスト値が短縮形になっていた
- ローカル用: `"実務経験豊富（メイン言語として使用）"`（正しい）
- 開発環境用: `"実務経験豊富"`（誤り）

#### 解決

テストスクリプトを修正して正しい選択リスト値を使用するように変更。

### 3. 動作確認

フォーム送信テストを実行し、以下を確認：

- ✅ フォーム送信成功
- ✅ Salesforceリード作成成功
- ✅ カスタムフィールド（Java**c、Python**c等）の値も正しく設定

## 今後の検討事項

### 1. 正式なConnected App設定

現在はアクセストークンベースで動作していますが、以下の理由で正式なConnected
Appの作成を検討：

- **リフレッシュトークンの利用**: アクセストークンの自動更新
- **セキュリティ向上**: IP制限やスコープ制限の設定
- **監査**: Connected App単位でのAPI利用状況の追跡

### 2. アクセストークンの管理

- 現在: 手動で`sf org display`から取得してRDSに保存
- 将来: リフレッシュトークンを使用した自動更新機能の実装

### 3. 環境変数の整理

- 現在の環境変数（SALESFORCE_CLIENT_ID、SALESFORCE_CLIENT_SECRET）は使用されていない
- Dockerfileやインフラコードの見直しを検討

## 使用したコマンド

```bash
# ログの確認
aws logs tail /ecs/markmail-dev --since 3m --profile [YOUR_PROFILE] | grep -i salesforce

# Salesforceでの確認（選択リスト値）
sf sobject describe -s Lead -o markmail-org | jq '.fields[] | select(.name == "Java__c") | {name, type, picklistValues}'

# リード作成の確認
sf data query -q "SELECT Id, FirstName, LastName, Email, Company, State, Java__c, Python__c FROM Lead WHERE Email = '[EMAIL]'" -o markmail-org
```

## スクリプトファイル

スクリプトファイルは以下のように整理されています：

### aws-deployment/

- `insert_crm_integration_to_rds.sql` - RDS投入用SQL
- `insert_crm_integration_rds.sh` - RDS投入実行スクリプト
- `create_salesforce_secret.sh` - AWS Secrets Manager設定（未使用）
- `update_aws_secrets.sh` - AWS Secrets Manager更新（未使用）
- `README_AWS_DEPLOYMENT.md` - AWSデプロイメント手順

### form-management/

- `create_markmail_form.py` - ローカル環境用フォーム作成
- `create_markmail_form_dev.py` - 開発環境用フォーム作成
- `publish_form.py` - ローカル環境用フォーム公開
- `publish_form_dev.py` - 開発環境用フォーム公開

### testing/

- `submit_form_test.py` - ローカル環境用フォーム送信テスト
- `submit_form_test_dev.py` - 開発環境用フォーム送信テスト

### utilities/

- `check_field_permissions.py` - フィールド権限確認
- `check_picklist_values.py` - 選択リスト値確認
- `update_sf_token_docker.py` - Dockerコンテナ内のトークン更新

## まとめ

AWS開発環境でのSalesforce連携が正常に動作するようになりました。重要な発見として、現在の実装ではOAuth認証フローを使用せず、アクセストークンを直接使用しているため、Connected
AppのクライアントIDとシークレットが不要であることが判明しました。

この実装方式により、環境変数の設定なしでもSalesforce
APIの呼び出しが可能となり、デプロイメントがシンプルになっています。
