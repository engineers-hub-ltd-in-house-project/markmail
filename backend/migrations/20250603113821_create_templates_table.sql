-- Add migration script here

-- テンプレートテーブルの作成
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    subject_template TEXT NOT NULL,
    markdown_content TEXT NOT NULL,
    html_content TEXT,
    variables JSONB DEFAULT '{}',
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- インデックス作成
CREATE INDEX idx_templates_user_id ON templates(user_id);
CREATE INDEX idx_templates_created_at ON templates(created_at DESC);
CREATE INDEX idx_templates_name ON templates(name);

-- 更新日時の自動更新トリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- コメント追加
COMMENT ON TABLE templates IS 'メールテンプレートテーブル';
COMMENT ON COLUMN templates.id IS 'テンプレートID';
COMMENT ON COLUMN templates.user_id IS '作成者のユーザーID';
COMMENT ON COLUMN templates.name IS 'テンプレート名';
COMMENT ON COLUMN templates.subject_template IS 'メール件名テンプレート';
COMMENT ON COLUMN templates.markdown_content IS 'マークダウンコンテンツ';
COMMENT ON COLUMN templates.html_content IS '変換されたHTMLコンテンツ（キャッシュ用）';
COMMENT ON COLUMN templates.variables IS 'テンプレート変数のJSON';
COMMENT ON COLUMN templates.is_public IS '公開テンプレートかどうか';
COMMENT ON COLUMN templates.created_at IS '作成日時';
COMMENT ON COLUMN templates.updated_at IS '更新日時';
