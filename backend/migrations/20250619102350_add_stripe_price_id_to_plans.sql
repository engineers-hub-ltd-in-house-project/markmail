-- Add stripe_price_id to subscription_plans table
ALTER TABLE subscription_plans ADD COLUMN IF NOT EXISTS stripe_price_id VARCHAR(255) UNIQUE;
ALTER TABLE subscription_plans ADD COLUMN IF NOT EXISTS stripe_product_id VARCHAR(255) UNIQUE;

-- Update existing plans with placeholder Stripe IDs (to be replaced with actual IDs)
UPDATE subscription_plans SET stripe_price_id = 'price_free_placeholder' WHERE name = 'free';
UPDATE subscription_plans SET stripe_price_id = 'price_pro_placeholder' WHERE name = 'pro';
UPDATE subscription_plans SET stripe_price_id = 'price_business_placeholder' WHERE name = 'business';

UPDATE subscription_plans SET stripe_product_id = 'prod_free_placeholder' WHERE name = 'free';
UPDATE subscription_plans SET stripe_product_id = 'prod_pro_placeholder' WHERE name = 'pro';
UPDATE subscription_plans SET stripe_product_id = 'prod_business_placeholder' WHERE name = 'business';