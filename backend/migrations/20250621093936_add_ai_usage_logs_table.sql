-- AI使用ログテーブルの作成
CREATE TABLE ai_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    feature_type VARCHAR(50) NOT NULL, -- scenario, content, subject
    prompt TEXT,
    response TEXT,
    tokens_used INTEGER,
    model_used VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- インデックスの作成
CREATE INDEX idx_ai_usage_logs_user_id ON ai_usage_logs(user_id);
CREATE INDEX idx_ai_usage_logs_feature_type ON ai_usage_logs(feature_type);
CREATE INDEX idx_ai_usage_logs_created_at ON ai_usage_logs(created_at);
