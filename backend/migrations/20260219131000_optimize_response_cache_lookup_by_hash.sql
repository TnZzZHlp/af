-- Optimize response cache lookup now that query matches by request_body_hash only.
CREATE INDEX IF NOT EXISTS idx_request_logs_cache_lookup_by_hash
  ON request_logs (request_body_hash, created_at DESC)
  WHERE status_code BETWEEN 200 AND 299
    AND response_body IS NOT NULL
    AND request_body_hash IS NOT NULL;
