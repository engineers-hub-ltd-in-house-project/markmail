-- sequence_step_logsテーブルのexecuted_atカラムを修正
-- AWS環境での無限メール送信エラーを解決

-- executed_atカラムにデフォルト値を追加
ALTER TABLE sequence_step_logs 
ALTER COLUMN executed_at SET DEFAULT NOW();

-- 既存のNULL値を現在時刻で更新
UPDATE sequence_step_logs 
SET executed_at = NOW()
WHERE executed_at IS NULL;

-- executed_atをNOT NULLに設定
ALTER TABLE sequence_step_logs 
ALTER COLUMN executed_at SET NOT NULL;