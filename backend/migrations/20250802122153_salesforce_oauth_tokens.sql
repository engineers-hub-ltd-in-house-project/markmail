-- OAuth認証情報テーブル
CREATE TABLE IF NOT EXISTS salesforce_oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    instance_url TEXT NOT NULL,
    token_type VARCHAR(50) NOT NULL DEFAULT 'Bearer',
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- トークン更新ログ
CREATE TABLE IF NOT EXISTS salesforce_token_refresh_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failure')),
    error_message TEXT,
    refreshed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_salesforce_oauth_tokens_user_id ON salesforce_oauth_tokens(user_id);
CREATE INDEX idx_salesforce_oauth_tokens_expires_at ON salesforce_oauth_tokens(expires_at);
CREATE INDEX idx_salesforce_token_refresh_logs_user_id ON salesforce_token_refresh_logs(user_id);
CREATE INDEX idx_salesforce_token_refresh_logs_refreshed_at ON salesforce_token_refresh_logs(refreshed_at);

-- Updated_atの自動更新トリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_salesforce_oauth_tokens_updated_at BEFORE UPDATE
    ON salesforce_oauth_tokens FOR EACH ROW EXECUTE FUNCTION 
    update_updated_at_column();
