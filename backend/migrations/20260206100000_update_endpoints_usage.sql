ALTER TABLE provider_endpoints DROP COLUMN weight;
ALTER TABLE provider_endpoints DROP COLUMN priority;
ALTER TABLE provider_endpoints ADD COLUMN usage_count bigint NOT NULL DEFAULT 0;
