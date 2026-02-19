-- Add a deterministic request-body hash key to speed up cache lookups.
-- The hash is generated from request_body and kept in sync automatically.
ALTER TABLE request_logs
  ADD COLUMN IF NOT EXISTS request_body_hash text
  GENERATED ALWAYS AS (
    CASE
      WHEN request_body IS NULL THEN NULL
      ELSE md5(encode(request_body, 'hex'))
    END
  ) STORED;

-- Targeted index for cache lookup path:
-- api_type + request_body_hash + newest-first, only for successful rows with response body.
CREATE INDEX IF NOT EXISTS idx_request_logs_cache_lookup
  ON request_logs (api_type, request_body_hash, created_at DESC)
  WHERE status_code BETWEEN 200 AND 299
    AND response_body IS NOT NULL
    AND request_body_hash IS NOT NULL;

COMMENT ON COLUMN request_logs.request_body_hash IS
  'Deterministic MD5 hash of request_body used as cache lookup key.';
