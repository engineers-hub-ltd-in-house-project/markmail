-- CRM統合の基盤テーブル
CREATE TABLE crm_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL CHECK (provider IN ('salesforce')), -- 将来的に 'hubspot', 'pipedrive' など追加可能
    org_id VARCHAR(255),                        -- Salesforce Organization ID
    instance_url VARCHAR(500),                  -- Salesforce Instance URL
    credentials JSONB NOT NULL,                 -- 暗号化された認証情報
    settings JSONB NOT NULL DEFAULT '{}',       -- 統合設定（sync_enabled, sync_contacts等）
    salesforce_settings JSONB,                  -- Salesforce固有の設定
    field_mappings JSONB NOT NULL DEFAULT '[]', -- フィールドマッピング設定
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- ユーザーごとに1つのCRMプロバイダーのみ許可
    UNIQUE(user_id, provider)
);

-- 同期状態追跡テーブル
CREATE TABLE crm_sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES crm_integrations(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('contact', 'campaign', 'list', 'activity')),
    markmail_id UUID NOT NULL,                  -- MarkMail側のID（subscriber_id, campaign_id等）
    crm_id VARCHAR(255),                        -- CRM側のID
    last_sync_hash VARCHAR(64),                 -- 最後に同期したデータのハッシュ値
    sync_status VARCHAR(20) NOT NULL CHECK (sync_status IN ('synced', 'pending', 'error', 'skipped')),
    sync_direction VARCHAR(20) CHECK (sync_direction IN ('to_crm', 'from_crm', 'bidirectional')),
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 統合とエンティティの組み合わせで一意
    UNIQUE(integration_id, entity_type, markmail_id)
);

-- 同期アクティビティログ
CREATE TABLE crm_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES crm_integrations(id) ON DELETE CASCADE,
    sync_type VARCHAR(50) NOT NULL CHECK (sync_type IN ('manual', 'scheduled', 'webhook', 'initial')),
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('contact', 'campaign', 'list', 'activity', 'all')),
    entity_count INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_details JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_crm_integrations_user_id ON crm_integrations(user_id);
CREATE INDEX idx_crm_integrations_provider ON crm_integrations(provider);
CREATE INDEX idx_crm_integrations_sync_enabled ON crm_integrations(sync_enabled) WHERE sync_enabled = true;

CREATE INDEX idx_crm_sync_status_integration_id ON crm_sync_status(integration_id);
CREATE INDEX idx_crm_sync_status_entity_type ON crm_sync_status(entity_type);
CREATE INDEX idx_crm_sync_status_markmail_id ON crm_sync_status(markmail_id);
CREATE INDEX idx_crm_sync_status_crm_id ON crm_sync_status(crm_id);
CREATE INDEX idx_crm_sync_status_sync_status ON crm_sync_status(sync_status);

CREATE INDEX idx_crm_sync_logs_integration_id ON crm_sync_logs(integration_id);
CREATE INDEX idx_crm_sync_logs_started_at ON crm_sync_logs(started_at);

-- 更新時刻を自動更新するトリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_crm_integrations_updated_at BEFORE UPDATE
    ON crm_integrations FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_crm_sync_status_updated_at BEFORE UPDATE
    ON crm_sync_status FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
