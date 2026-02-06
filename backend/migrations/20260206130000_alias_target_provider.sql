-- Update alias_targets to bind to provider instead of endpoint
-- and remove circuit breaker fields.

-- Add provider_id column
ALTER TABLE alias_targets ADD COLUMN provider_id uuid REFERENCES providers(id);

-- Populate provider_id from existing data (best effort)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'alias_targets' AND column_name = 'provider_endpoint_id') THEN
        UPDATE alias_targets 
        SET provider_id = (SELECT provider_id FROM provider_endpoints WHERE id = alias_targets.provider_endpoint_id)
        WHERE provider_endpoint_id IS NOT NULL;
    END IF;
END $$;

-- If provider_id is null (orphaned targets or empty table), we can't really enforce NOT NULL without deleting.
-- Let's delete invalid rows to enforce integrity.
DELETE FROM alias_targets WHERE provider_id IS NULL;
ALTER TABLE alias_targets ALTER COLUMN provider_id SET NOT NULL;

-- Drop old columns
ALTER TABLE alias_targets DROP COLUMN provider_endpoint_id;
ALTER TABLE alias_targets DROP COLUMN fail_count;
ALTER TABLE alias_targets DROP COLUMN circuit_open_until;
ALTER TABLE alias_targets DROP COLUMN last_fail_at;

-- Update unique constraint
ALTER TABLE alias_targets DROP CONSTRAINT IF EXISTS alias_targets_alias_id_provider_endpoint_id_model_id_key;
ALTER TABLE alias_targets ADD CONSTRAINT alias_targets_alias_id_provider_id_model_id_key UNIQUE (alias_id, provider_id, model_id);
