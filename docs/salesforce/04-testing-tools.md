# テストツールとスクリプト

このドキュメントでは、Salesforce統合のテストに使用するツールとスクリプトについて説明します。

## スクリプトディレクトリ構成

```
scripts/salesforce-integration/
├── README.md
├── testing/              # テストスクリプト
├── form-management/      # フォーム管理
├── utilities/            # ユーティリティ
└── aws-deployment/       # AWSデプロイ用
```

## テストスクリプト

### OAuth2フローテスト

#### `oauth2_flow.py`

OAuth2認証フロー全体をテストするスクリプトです。

```python
# 使用方法
cd scripts/salesforce-integration/testing
python oauth2_flow.py
```

実行内容:

1. JWT取得（MarkMailログイン）
2. OAuth2認証URL生成
3. ブラウザで認証ページを開く
4. コールバックURLの入力待ち
5. アクセストークン取得

設定ファイル例（`config.json`）:

```json
{
  "api_base_url": "http://localhost:3000/api",
  "form_id": "72406032-3f07-4da2-9621-4018373b857e",
  "admin_email": "admin@example.com",
  "admin_password": "password123"
}
```

#### `test_oauth_curl.sh`

cURLを使用した簡易OAuth2テストスクリプトです。

```bash
#!/bin/bash
# OAuth2認証URLの生成
JWT_TOKEN="your-jwt-token"
curl -X POST http://localhost:3000/api/crm/oauth/salesforce/initiate \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json"
```

### フォーム送信テスト

#### `submit_form_test_dev.py`

開発環境でのフォーム送信とリード作成をテストします。

```python
# 使用方法
python submit_form_test_dev.py
```

テストデータ例:

```python
form_data = {
    "name": "テスト 太郎",
    "email": f"test_{timestamp}@example.com",
    "company": "テスト株式会社",
    "phone": "03-1234-5678",
    "engineer_skills": "Python, AWS, Docker",
    "years_of_experience": "5",
    "message": "Salesforce統合テスト"
}
```

実行結果:

- フォーム送信成功/失敗
- レスポンスデータ
- Salesforceリード作成結果

#### `submit_form_test.py`

本番環境用のフォーム送信テストスクリプトです。

```python
# 設定変更
config = {
    "api_base_url": "https://markmail.engineers-hub.ltd/api",
    "form_id": "production-form-id"
}
```

### 認証補助ツール

#### `complete_oauth_callback.py`

OAuth2コールバック処理を手動で完了させるスクリプトです。

```python
# 使用方法
python complete_oauth_callback.py --code "auth_code_from_salesforce" --state "state_from_url"
```

用途:

- 自動テストでのコールバック処理
- デバッグ時の手動トークン取得

#### `login_and_oauth.py`

ログインからOAuth2認証まで一連の流れを自動化します。

```python
# 使用方法
python login_and_oauth.py --email admin@example.com --password password123
```

実行内容:

1. MarkMailへのログイン
2. JWT取得
3. OAuth2認証開始
4. 認証URLの表示

## フォーム管理スクリプト

### フォーム作成

#### `create_markmail_form_dev.py`

開発環境用のテストフォームを作成します。

```python
# 使用方法
python create_markmail_form_dev.py
```

作成されるフォームフィールド:

- name (必須)
- email (必須)
- company (必須)
- phone
- engineer_skills
- years_of_experience
- message

#### `create_markmail_form.py`

カスタマイズ可能なフォーム作成スクリプトです。

```python
# カスタムフィールドの定義
custom_fields = [
    {
        "name": "project_budget",
        "type": "number",
        "label": "Project Budget",
        "required": False
    }
]
```

### フォーム公開

#### `publish_form_dev.py`

作成したフォームを公開状態にします。

```python
# 使用方法
python publish_form_dev.py --form-id "your-form-id"
```

出力:

```
Form published successfully!
Public URL: http://localhost:5173/forms/72406032-3f07-4da2-9621-4018373b857e/public
```

## ユーティリティツール

### フィールド権限チェック

#### `check_field_permissions.py`

Salesforceのカスタムフィールドの権限を確認します。

```python
# 使用方法
python check_field_permissions.py
```

出力例:

```
Field: Engineer_Skills__c
- Accessible: True
- Creatable: True
- Updateable: True
- Required: False
```

用途:

- フィールドレベルセキュリティの確認
- API経由でのフィールドアクセス可否の検証

### ピックリスト値確認

#### `check_picklist_values.py`

ピックリストフィールドの有効な値を取得します。

```python
# 使用方法
python check_picklist_values.py --field LeadSource
```

出力例:

```
Valid values for LeadSource:
- Web
- Phone Inquiry
- Partner Referral
- Purchased List
- Other
```

### Docker環境トークン更新

#### `update_sf_token_docker.py`

Docker環境でSalesforceトークンを更新します。

```python
# 使用方法
python update_sf_token_docker.py --container markmail-backend-1
```

用途:

- コンテナ再起動なしでトークン更新
- 開発時の認証エラー解決

## AWS環境用スクリプト

### Secrets Manager設定

#### `create_salesforce_secret.sh`

AWS Secrets ManagerにSalesforce認証情報を作成します。

```bash
#!/bin/bash
aws secretsmanager create-secret \
  --name markmail-dev-salesforce-secret \
  --secret-string '{
    "client_id": "your-client-id",
    "client_secret": "your-client-secret",
    "redirect_url": "https://dev.markmail.engineers-hub.ltd/api/crm/oauth/salesforce/callback",
    "use_sandbox": "false"
  }'
```

#### `update_aws_secrets.sh`

既存のシークレットを更新します。

```bash
# 使用方法
./update_aws_secrets.sh dev "your-new-client-id" "your-new-secret"
```

### RDSデータ投入

#### `insert_crm_integration_rds.sh`

RDSに直接CRM統合データを投入します。

```bash
# 使用方法
./insert_crm_integration_rds.sh --env dev --user-id "uuid"
```

SQLファイル（`insert_crm_integration_to_rds.sql`）:

```sql
INSERT INTO crm_integrations (
    id, user_id, provider, access_token,
    refresh_token, expires_at, instance_url
) VALUES (
    gen_random_uuid(),
    :user_id,
    'salesforce',
    :access_token,
    :refresh_token,
    :expires_at,
    :instance_url
);
```

## テストシナリオ

### 1. 新規統合セットアップ

```bash
# 1. フォーム作成
python create_markmail_form_dev.py

# 2. OAuth2認証
python oauth2_flow.py

# 3. フォーム送信テスト
python submit_form_test_dev.py

# 4. 権限確認
python check_field_permissions.py
```

### 2. トークンリフレッシュテスト

```python
# refresh_token_test.py
import time

# 1. 初回認証
initial_token = get_access_token()

# 2. トークン期限切れを待つ（または手動で期限を過去に設定）
time.sleep(7200)  # 2時間

# 3. API呼び出しでリフレッシュが自動実行されることを確認
response = create_lead_with_expired_token()
assert response.status_code == 200
```

### 3. エラーハンドリングテスト

```python
# error_handling_test.py

# 無効なフィールド
invalid_lead = {
    "InvalidField__c": "value"
}

# 必須フィールド不足
incomplete_lead = {
    "Email": "test@example.com"
    # LastNameとCompanyが不足
}

# 権限エラー
restricted_field = {
    "RestrictedField__c": "value"
}
```

## デバッグのヒント

### リクエスト/レスポンスログ

```python
# ログレベルを設定
import logging
logging.basicConfig(level=logging.DEBUG)

# HTTPリクエストの詳細を表示
import http.client
http.client.HTTPConnection.debuglevel = 1
```

### Salesforceエラーの詳細

```python
try:
    response = create_lead(lead_data)
except Exception as e:
    # Salesforceのエラーレスポンスを解析
    error_details = json.loads(str(e))
    print(f"Error Code: {error_details[0]['errorCode']}")
    print(f"Message: {error_details[0]['message']}")
    print(f"Fields: {error_details[0].get('fields', [])}")
```

### データベース状態確認

```bash
# トークン確認
docker exec -it markmail-postgres-1 psql -U postgres -d markmail \
  -c "SELECT user_id, provider, expires_at, instance_url FROM crm_integrations;"

# フォーム送信確認
docker exec -it markmail-postgres-1 psql -U postgres -d markmail \
  -c "SELECT * FROM form_submissions ORDER BY created_at DESC LIMIT 5;"
```

## 関連ドキュメント

- [開発環境セットアップ](./03-development-setup.md)
- [トラブルシューティング](./06-troubleshooting.md)
- [スクリプトREADME](../../scripts/salesforce-integration/README.md)
