-- Change campaigns status column from VARCHAR to TEXT to match Rust type
ALTER TABLE campaigns ALTER COLUMN status TYPE TEXT;
