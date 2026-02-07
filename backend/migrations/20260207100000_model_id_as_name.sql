-- Migration to change alias_targets.model_id from UUID to TEXT
-- and add api_type to alias_targets.

-- 1. Add api_type column
ALTER TABLE alias_targets ADD COLUMN api_type api_type;

-- 2. Populate api_type from models table
UPDATE alias_targets at
SET api_type = m.api_type
FROM models m
WHERE at.model_id = m.id;

-- If any targets were orphaned, we might want to delete them or handle them.
-- Given the current state, they should all have models.
ALTER TABLE alias_targets ALTER COLUMN api_type SET NOT NULL;

-- 3. Add model_name column (temporary)
ALTER TABLE alias_targets ADD COLUMN model_name TEXT;

-- 4. Populate model_name from models table
UPDATE alias_targets at
SET model_name = m.name
FROM models m
WHERE at.model_id = m.id;

ALTER TABLE alias_targets ALTER COLUMN model_name SET NOT NULL;

-- 5. Drop the old model_id (UUID)
ALTER TABLE alias_targets DROP COLUMN model_id;

-- 6. Rename model_name to model_id
ALTER TABLE alias_targets RENAME COLUMN model_name TO model_id;

-- 7. Update unique constraint
ALTER TABLE alias_targets DROP CONSTRAINT IF EXISTS alias_targets_alias_id_provider_id_model_id_key;
ALTER TABLE alias_targets ADD CONSTRAINT alias_targets_alias_id_provider_id_model_id_api_type_key UNIQUE (alias_id, provider_id, model_id, api_type);
