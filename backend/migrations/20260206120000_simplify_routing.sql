ALTER TABLE aliases DROP COLUMN strategy;
DROP TYPE lb_strategy;
ALTER TABLE alias_targets DROP COLUMN weight;
ALTER TABLE alias_targets DROP COLUMN priority;
ALTER TABLE provider_keys ADD COLUMN usage_count bigint NOT NULL DEFAULT 0;
