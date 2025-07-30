# Salesforce統合のAWSデプロイメント手順

## 1. RDSへのデータ投入

Bastion Host経由でRDSにCRM統合設定を投入します：

```bash
# Bastion Hostインスタンスを取得
INSTANCE_ID=$(aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  --query 'Reservations[*].Instances[*].[InstanceId]' \
  --output text \
  --profile your-profile)

# SQLファイルの内容を実行
aws ssm send-command \
  --instance-ids "$INSTANCE_ID" \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-rds-password\"",
    "psql -h your-rds-endpoint.rds.amazonaws.com -U markmail -d markmail <<EOF
-- CRM統合設定を挿入
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
    sync_enabled,
    created_at,
    updated_at
) VALUES (
    '"'"'7d027cc5-1b3a-48e5-8410-15052fcf9130'"'"'::uuid,
    (SELECT id FROM users WHERE email = '"'"'yusuke.sato@engineers-hub.ltd'"'"'),
    '"'"'salesforce'"'"',
    '"'"'markmail-org'"'"',
    '"'"'https://customization-computing-9993.my.salesforce.com'"'"',
    '"'"'{\"access_token\": \"00DIR000001cWPD!AQwAQJS4qLKbsWVJ7pBVNGD2tC4im.Sjk.uXVSElzam60HsT2dvOst1ZGuX5wUKoTSpL5lIKBuXUKw2F8nYw30.RHHAsC5ch\", \"refresh_token\": null}'"'"'::jsonb,
    '"'"'{\"batch_size\": 200, \"sync_enabled\": true, \"sync_interval_minutes\": 60}'"'"'::jsonb,
    '"'"'{\"org_id\": \"markmail-org\", \"api_version\": \"v60.0\", \"instance_url\": \"https://customization-computing-9993.my.salesforce.com\"}'"'"'::jsonb,
    '"'"'[]'"'"'::jsonb,
    true,
    NOW(),
    NOW()
)
ON CONFLICT (user_id, provider) DO UPDATE SET
    org_id = EXCLUDED.org_id,
    instance_url = EXCLUDED.instance_url,
    credentials = EXCLUDED.credentials,
    settings = EXCLUDED.settings,
    salesforce_settings = EXCLUDED.salesforce_settings,
    field_mappings = EXCLUDED.field_mappings,
    sync_enabled = EXCLUDED.sync_enabled,
    updated_at = NOW();
EOF"
  ]' \
  --profile your-profile
```

## 2. AWS Secrets Managerの更新

`update_aws_secrets.sh`を使用して環境変数を追加：

```bash
# スクリプトを編集して実際の値を設定
vi scripts/salesforce-integration/update_aws_secrets.sh

# 実行
./scripts/salesforce-integration/update_aws_secrets.sh your-profile
```

## 3. Salesforce CLIの対応について

現在のコードは`sf`コマンドを前提としていますが、AWSのコンテナ環境では以下の対応が必要です：

### オプション1: 環境変数方式

現在のコードで既に対応済み：

- `SALESFORCE_CLIENT_ID`と`SALESFORCE_CLIENT_SECRET`を環境変数で設定
- アクセストークンはDBに保存されているものを使用
- リフレッシュトークンで自動更新（実装が必要）

### オプション2: Dockerfileでsfコマンドをインストール

```dockerfile
# 実行ステージに追加
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    && npm install -g @salesforce/cli \
    && rm -rf /var/lib/apt/lists/*
```

ただし、この方法は：

- コンテナサイズが大きくなる
- 認証情報の管理が複雑
- AWS環境では適さない場合がある

### オプション3: 直接API呼び出しに変更

`salesforce_auth.rs`を修正して、`sf`コマンドではなく直接OAuth2フローを実装。

## 4. アクセストークンの更新

現在のアクセストークンは期限切れの可能性があるため、新しいトークンの取得が必要です：

1. ローカルで`sf org display --target-org markmail-org --json`を実行
2. 新しいアクセストークンを取得
3. RDSのcrm_integrationsテーブルを更新

## 注意事項

- Salesforce Connected Appの設定が必要
- IP制限がある場合はAWSのIPを許可リストに追加
- アクセストークンは定期的に更新が必要（リフレッシュトークンの実装が必要）
