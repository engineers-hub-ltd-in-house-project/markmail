-- シーケンステーブル（既に存在する場合はスキップ）
CREATE TABLE IF NOT EXISTS sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL,
    trigger_config JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    active_subscribers INTEGER NOT NULL DEFAULT 0,
    completed_subscribers INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- シーケンスステップテーブル（既に存在する場合はスキップ）
CREATE TABLE IF NOT EXISTS sequence_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id UUID NOT NULL REFERENCES sequences(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    step_order INTEGER NOT NULL,
    step_type VARCHAR(50) NOT NULL,
    delay_value INTEGER NOT NULL DEFAULT 0,
    delay_unit VARCHAR(20) NOT NULL DEFAULT 'hours',
    template_id UUID REFERENCES templates(id) ON DELETE SET NULL,
    subject VARCHAR(255),
    conditions JSONB NOT NULL DEFAULT '{}',
    action_config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sequence_id, step_order)
);

-- シーケンス実行状態テーブル（既に存在する場合はスキップ）
CREATE TABLE IF NOT EXISTS sequence_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id UUID NOT NULL REFERENCES sequences(id) ON DELETE CASCADE,
    subscriber_id UUID NOT NULL REFERENCES subscribers(id) ON DELETE CASCADE,
    current_step_id UUID REFERENCES sequence_steps(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    next_step_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sequence_id, subscriber_id)
);

-- シーケンスステップ実行履歴テーブル（既に存在する場合はスキップ）
CREATE TABLE IF NOT EXISTS sequence_step_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID NOT NULL REFERENCES sequence_enrollments(id) ON DELETE CASCADE,
    step_id UUID NOT NULL REFERENCES sequence_steps(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,
    executed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス（存在しない場合のみ作成）
CREATE INDEX IF NOT EXISTS idx_sequences_user_id ON sequences(user_id);
CREATE INDEX IF NOT EXISTS idx_sequences_status ON sequences(status);
CREATE INDEX IF NOT EXISTS idx_sequences_trigger_type ON sequences(trigger_type);
CREATE INDEX IF NOT EXISTS idx_sequences_created_at ON sequences(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_sequence_steps_sequence_id ON sequence_steps(sequence_id);
CREATE INDEX IF NOT EXISTS idx_sequence_steps_step_order ON sequence_steps(step_order);
CREATE INDEX IF NOT EXISTS idx_sequence_steps_template_id ON sequence_steps(template_id);

CREATE INDEX IF NOT EXISTS idx_sequence_enrollments_sequence_id ON sequence_enrollments(sequence_id);
CREATE INDEX IF NOT EXISTS idx_sequence_enrollments_subscriber_id ON sequence_enrollments(subscriber_id);
CREATE INDEX IF NOT EXISTS idx_sequence_enrollments_status ON sequence_enrollments(status);
CREATE INDEX IF NOT EXISTS idx_sequence_enrollments_next_step_at ON sequence_enrollments(next_step_at);

CREATE INDEX IF NOT EXISTS idx_sequence_step_logs_enrollment_id ON sequence_step_logs(enrollment_id);
CREATE INDEX IF NOT EXISTS idx_sequence_step_logs_step_id ON sequence_step_logs(step_id);
CREATE INDEX IF NOT EXISTS idx_sequence_step_logs_status ON sequence_step_logs(status);