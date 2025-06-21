-- AI使用制限カラムをsubscription_plansテーブルに追加
ALTER TABLE subscription_plans
ADD COLUMN ai_monthly_limit INTEGER,
ADD COLUMN ai_scenario_limit INTEGER,
ADD COLUMN ai_content_limit INTEGER,
ADD COLUMN ai_subject_limit INTEGER;

-- 既存のプランにデフォルト値を設定
UPDATE subscription_plans
SET 
    ai_monthly_limit = CASE 
        WHEN name = 'free' THEN 10
        WHEN name = 'pro' THEN 500
        WHEN name = 'business' THEN NULL  -- NULLは無制限を意味する
    END,
    ai_scenario_limit = CASE 
        WHEN name = 'free' THEN 3
        WHEN name = 'pro' THEN 50
        WHEN name = 'business' THEN NULL
    END,
    ai_content_limit = CASE 
        WHEN name = 'free' THEN 5
        WHEN name = 'pro' THEN 300
        WHEN name = 'business' THEN NULL
    END,
    ai_subject_limit = CASE 
        WHEN name = 'free' THEN 2
        WHEN name = 'pro' THEN 150
        WHEN name = 'business' THEN NULL
    END;