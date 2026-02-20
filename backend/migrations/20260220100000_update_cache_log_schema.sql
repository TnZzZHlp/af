-- Remove redundant fields and add client context to cache_log.

-- Drop redundant columns
ALTER TABLE cache_log
  DROP COLUMN api_type,
  DROP COLUMN hit,
  DROP COLUMN request_body_hash,
  DROP COLUMN request_body_size,
  DROP COLUMN response_body_size;

-- Add client context columns
ALTER TABLE cache_log
  ADD COLUMN client_ip inet,
  ADD COLUMN user_agent text;

-- Drop obsolete indices
DROP INDEX IF EXISTS idx_cache_log_api_type;
DROP INDEX IF EXISTS idx_cache_log_hit_layer;

-- Add index for client_ip if needed for filtering/stats (optional but good for consistency with request_logs)
-- request_logs doesn't have an index on client_ip in the init sql, but it might be useful.
-- For now, I'll stick to the requested changes.
