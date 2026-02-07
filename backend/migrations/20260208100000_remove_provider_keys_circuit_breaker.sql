-- Remove circuit breaker fields from provider_keys
ALTER TABLE provider_keys DROP COLUMN fail_count;
ALTER TABLE provider_keys DROP COLUMN circuit_open_until;
ALTER TABLE provider_keys DROP COLUMN last_fail_at;
