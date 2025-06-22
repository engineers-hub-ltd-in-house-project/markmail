-- キャンペーンステータスのCHECK制約を更新してerrorステータスを追加
ALTER TABLE campaigns DROP CONSTRAINT IF EXISTS campaigns_status_check;
ALTER TABLE campaigns ADD CONSTRAINT campaigns_status_check 
    CHECK (status IN ('draft', 'scheduled', 'sending', 'sent', 'paused', 'cancelled', 'error'));
