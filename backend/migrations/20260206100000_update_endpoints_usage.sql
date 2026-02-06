ALTER TABLE provider_endpoints DROP COLUMN weight;
ALTER TABLE provider_endpoints DROP COLUMN priority;
ALTER TABLE providers ADD COLUMN usage_count bigint NOT NULL DEFAULT 0;