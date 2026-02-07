-- Remove api_type from alias_targets

-- 1. Drop the unique constraint that includes api_type
ALTER TABLE alias_targets DROP CONSTRAINT IF EXISTS alias_targets_alias_id_provider_id_model_id_api_type_key;

-- 2. Drop the api_type column
ALTER TABLE alias_targets DROP COLUMN api_type;

-- 3. Add new unique constraint
ALTER TABLE alias_targets ADD CONSTRAINT alias_targets_alias_id_provider_id_model_id_key UNIQUE (alias_id, provider_id, model_id);
