-- Add migration script for creating the subscribers table

-- Create subscriber_status enum type
CREATE TYPE subscriber_status AS ENUM (
    'active',
    'unsubscribed',
    'bounced',
    'complained'
);

-- Create subscribers table
CREATE TABLE subscribers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    status subscriber_status NOT NULL DEFAULT 'active',
    tags TEXT[] NOT NULL DEFAULT '{}',
    custom_fields JSONB NOT NULL DEFAULT '{}'::jsonb,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unsubscribed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_subscribers_user_id ON subscribers(user_id);
CREATE UNIQUE INDEX idx_subscribers_user_email ON subscribers(user_id, email);
CREATE INDEX idx_subscribers_status ON subscribers(status);
CREATE INDEX idx_subscribers_tags ON subscribers USING GIN(tags);

-- Add trigger for updated_at
CREATE TRIGGER set_subscribers_updated_at
BEFORE UPDATE ON subscribers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();