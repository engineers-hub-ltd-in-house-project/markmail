#!/bin/bash

# RDSにCRM統合設定を投入するスクリプト
# Bastion Host経由で実行

INSTANCE_ID="[YOUR_BASTION_INSTANCE_ID]"
RDS_ENDPOINT="[YOUR_RDS_ENDPOINT]"
DB_PASSWORD="[YOUR_DB_PASSWORD]"
ACCESS_TOKEN="[YOUR_ACCESS_TOKEN]"

echo "RDSにCRM統合設定を投入中..."

aws ssm send-command \
  --instance-ids "$INSTANCE_ID" \
  --document-name "AWS-RunShellScript" \
  --parameters "commands=[
    \"export PGPASSWORD='$DB_PASSWORD'\",
    \"psql -h $RDS_ENDPOINT -U markmail -d markmail <<'EOF'
-- まずユーザーIDを確認
SELECT id, email FROM users WHERE email = 'yusuke.sato@engineers-hub.ltd';

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
    sync_enabled,
    created_at,
    updated_at
) VALUES (
    '7d027cc5-1b3a-48e5-8410-15052fcf9130'::uuid,
    (SELECT id FROM users WHERE email = 'yusuke.sato@engineers-hub.ltd'),
    'salesforce',
    'markmail-org',
    'https://customization-computing-9993.my.salesforce.com',
    '{\\\"access_token\\\": \\\"$ACCESS_TOKEN\\\", \\\"refresh_token\\\": null}'::jsonb,
    '{\\\"batch_size\\\": 200, \\\"sync_enabled\\\": true, \\\"sync_interval_minutes\\\": 60}'::jsonb,
    '{\\\"org_id\\\": \\\"markmail-org\\\", \\\"api_version\\\": \\\"v60.0\\\", \\\"instance_url\\\": \\\"https://customization-computing-9993.my.salesforce.com\\\"}'::jsonb,
    '[]'::jsonb,
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

-- 結果を確認
SELECT id, user_id, provider, sync_enabled, credentials->>'access_token' as token_preview 
FROM crm_integrations 
WHERE provider = 'salesforce';
EOF\"
  ]" \
  --profile yusuke.sato \
  --output json

echo "コマンドを送信しました。実行結果を確認中..."