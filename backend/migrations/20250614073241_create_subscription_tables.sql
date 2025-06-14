-- Create subscription plans table
CREATE TABLE IF NOT EXISTS subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    price INTEGER NOT NULL DEFAULT 0,
    billing_interval VARCHAR(20) NOT NULL DEFAULT 'monthly',
    -- Limits
    contact_limit INTEGER NOT NULL DEFAULT -1,
    monthly_email_limit INTEGER NOT NULL DEFAULT -1,
    campaign_limit INTEGER NOT NULL DEFAULT -1,
    template_limit INTEGER NOT NULL DEFAULT -1,
    sequence_limit INTEGER NOT NULL DEFAULT -1,
    sequence_step_limit INTEGER NOT NULL DEFAULT -1,
    form_limit INTEGER NOT NULL DEFAULT -1,
    form_submission_limit INTEGER NOT NULL DEFAULT -1,
    user_limit INTEGER NOT NULL DEFAULT 1,
    webhook_limit INTEGER NOT NULL DEFAULT 0,
    -- Features
    custom_markdown_components BOOLEAN NOT NULL DEFAULT false,
    ai_features BOOLEAN NOT NULL DEFAULT false,
    advanced_analytics BOOLEAN NOT NULL DEFAULT false,
    ab_testing BOOLEAN NOT NULL DEFAULT false,
    api_access BOOLEAN NOT NULL DEFAULT false,
    priority_support BOOLEAN NOT NULL DEFAULT false,
    custom_domain BOOLEAN NOT NULL DEFAULT false,
    white_label BOOLEAN NOT NULL DEFAULT false,
    -- Metadata
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    features JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create user subscriptions table
CREATE TABLE IF NOT EXISTS user_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'canceled', 'past_due', 'trialing')),
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    current_period_end TIMESTAMPTZ NOT NULL,
    cancel_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    metadata JSONB,
    stripe_subscription_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- Create usage records table
CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, metric_type, period_start, period_end)
);

-- Create payment history table
CREATE TABLE IF NOT EXISTS payment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES user_subscriptions(id),
    amount INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'JPY',
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('succeeded', 'failed', 'pending', 'refunded')),
    description TEXT,
    stripe_payment_intent_id VARCHAR(255),
    stripe_invoice_id VARCHAR(255),
    metadata JSONB,
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create usage alerts table
CREATE TABLE IF NOT EXISTS usage_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL,
    threshold_percentage INTEGER NOT NULL DEFAULT 80,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    last_alerted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, metric_type)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_user_id ON user_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_status ON user_subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_usage_records_user_id ON usage_records(user_id);
CREATE INDEX IF NOT EXISTS idx_usage_records_metric_type ON usage_records(metric_type);
CREATE INDEX IF NOT EXISTS idx_payment_history_user_id ON payment_history(user_id);
CREATE INDEX IF NOT EXISTS idx_payment_history_status ON payment_history(status);
CREATE INDEX IF NOT EXISTS idx_usage_alerts_user_id ON usage_alerts(user_id);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_subscription_tables_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_subscription_plans_updated_at BEFORE UPDATE ON subscription_plans
    FOR EACH ROW EXECUTE FUNCTION update_subscription_tables_updated_at();

CREATE TRIGGER update_user_subscriptions_updated_at BEFORE UPDATE ON user_subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_subscription_tables_updated_at();

CREATE TRIGGER update_usage_records_updated_at BEFORE UPDATE ON usage_records
    FOR EACH ROW EXECUTE FUNCTION update_subscription_tables_updated_at();

CREATE TRIGGER update_usage_alerts_updated_at BEFORE UPDATE ON usage_alerts
    FOR EACH ROW EXECUTE FUNCTION update_subscription_tables_updated_at();

-- Insert default plans
INSERT INTO subscription_plans (
    name, display_name, description, price, billing_interval,
    contact_limit, monthly_email_limit, campaign_limit, template_limit,
    sequence_limit, sequence_step_limit, form_limit, form_submission_limit,
    user_limit, webhook_limit, custom_markdown_components, ai_features,
    advanced_analytics, ab_testing, api_access, priority_support,
    custom_domain, white_label, sort_order
) VALUES
    -- Free Plan
    ('free', 'Free', '個人・スタートアップ向け', 0, 'monthly',
     100, 1000, 10, 10, 3, 5, 3, 100, 1, 0, false, false,
     false, false, false, false, false, false, 1),
    -- Pro Plan
    ('pro', 'Pro', '成長企業向け', 4980, 'monthly',
     10000, 100000, 100, 100, 20, 20, 20, 10000, 5, 10, true, true,
     true, false, true, false, false, false, 2),
    -- Business Plan
    ('business', 'Business', 'エンタープライズ向け', 19800, 'monthly',
     100000, 1000000, -1, -1, -1, -1, -1, -1, -1, -1, true, true,
     true, true, true, true, true, true, 3)
ON CONFLICT (name) DO NOTHING;

-- Migrate existing users to free plan
INSERT INTO user_subscriptions (user_id, plan_id, status, current_period_start, current_period_end)
SELECT 
    u.id,
    p.id,
    'active',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP + INTERVAL '30 days'
FROM users u
CROSS JOIN subscription_plans p
WHERE p.name = 'free'
  AND NOT EXISTS (
    SELECT 1 FROM user_subscriptions us WHERE us.user_id = u.id
  );

-- Create function to assign free plan to new users
CREATE OR REPLACE FUNCTION assign_free_plan_to_new_user()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_subscriptions (user_id, plan_id, status, current_period_start, current_period_end)
    SELECT 
        NEW.id,
        p.id,
        'active',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP + INTERVAL '30 days'
    FROM subscription_plans p
    WHERE p.name = 'free';
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically assign free plan when user registers
CREATE TRIGGER assign_free_plan_on_user_create
AFTER INSERT ON users
FOR EACH ROW EXECUTE FUNCTION assign_free_plan_to_new_user();