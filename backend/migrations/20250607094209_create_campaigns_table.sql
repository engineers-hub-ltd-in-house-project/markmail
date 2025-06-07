-- Add migration script here

-- キャンペーンテーブルの作成
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    subject VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    recipient_count INT DEFAULT 0,
    sent_count INT DEFAULT 0,
    opened_count INT DEFAULT 0,
    clicked_count INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- インデックス作成
CREATE INDEX idx_campaigns_user_id ON campaigns(user_id);
CREATE INDEX idx_campaigns_template_id ON campaigns(template_id);
CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_created_at ON campaigns(created_at DESC);
CREATE INDEX idx_campaigns_scheduled_at ON campaigns(scheduled_at);

-- 更新日時の自動更新トリガー
CREATE TRIGGER update_campaigns_updated_at
    BEFORE UPDATE ON campaigns
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- キャンペーンのステータス列にチェック制約を追加
ALTER TABLE campaigns 
    ADD CONSTRAINT campaigns_status_check 
    CHECK (status IN ('draft', 'scheduled', 'sending', 'sent', 'paused', 'cancelled'));

-- コメント追加
COMMENT ON TABLE campaigns IS 'メールキャンペーンテーブル';
COMMENT ON COLUMN campaigns.id IS 'キャンペーンID';
COMMENT ON COLUMN campaigns.user_id IS '作成者のユーザーID';
COMMENT ON COLUMN campaigns.template_id IS 'テンプレートID';
COMMENT ON COLUMN campaigns.name IS 'キャンペーン名';
COMMENT ON COLUMN campaigns.description IS 'キャンペーンの説明';
COMMENT ON COLUMN campaigns.subject IS 'メール件名';
COMMENT ON COLUMN campaigns.status IS 'キャンペーンステータス (draft, scheduled, sending, sent, paused, cancelled)';
COMMENT ON COLUMN campaigns.scheduled_at IS '配信予定日時';
COMMENT ON COLUMN campaigns.sent_at IS '実際の配信開始日時';
COMMENT ON COLUMN campaigns.recipient_count IS '配信対象者数';
COMMENT ON COLUMN campaigns.sent_count IS '配信完了数';
COMMENT ON COLUMN campaigns.opened_count IS '開封数';
COMMENT ON COLUMN campaigns.clicked_count IS 'クリック数';
COMMENT ON COLUMN campaigns.created_at IS '作成日時';
COMMENT ON COLUMN campaigns.updated_at IS '更新日時';