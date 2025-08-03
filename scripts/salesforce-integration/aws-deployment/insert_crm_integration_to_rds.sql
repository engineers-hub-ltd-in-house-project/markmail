-- CRM統合設定をRDSに挿入するSQL
-- yusuke.sato@engineers-hub.ltdユーザーのSalesforce統合設定を挿入

-- まず既存のユーザーIDを確認
-- SELECT id FROM users WHERE email = 'yusuke.sato@engineers-hub.ltd';

-- CRM統合設定を挿入（既存の場合は更新）
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
    '{"access_token": "[YOUR_ACCESS_TOKEN]", "refresh_token": null}'::jsonb,
    '{"batch_size": 200, "sync_enabled": true, "sync_interval_minutes": 60}'::jsonb,
    '{"org_id": "markmail-org", "api_version": "v60.0", "instance_url": "https://customization-computing-9993.my.salesforce.com"}'::jsonb,
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