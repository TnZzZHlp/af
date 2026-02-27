ALTER TABLE alias_targets ADD COLUMN extra_fields jsonb NOT NULL DEFAULT '{}';
ALTER TABLE aliases DROP COLUMN extra_fields;