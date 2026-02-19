-- Cache middleware event log table.
CREATE TABLE IF NOT EXISTS cache_log (
  id bigserial PRIMARY KEY,
  request_id uuid NOT NULL,
  gateway_key_id uuid,
  api_type api_type NOT NULL,
  cache_layer text NOT NULL CHECK (cache_layer IN ('moka', 'database')),
  hit boolean NOT NULL,
  request_body_hash text,
  request_body_size integer,
  response_body_size integer,
  latency_ms integer,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cache_log_created_at
  ON cache_log(created_at);

CREATE INDEX IF NOT EXISTS idx_cache_log_gateway_key
  ON cache_log(gateway_key_id);

CREATE INDEX IF NOT EXISTS idx_cache_log_api_type
  ON cache_log(api_type);

CREATE INDEX IF NOT EXISTS idx_cache_log_hit_layer
  ON cache_log(hit, cache_layer);

COMMENT ON TABLE cache_log IS 'Cache hit/miss event logs for middleware-level response cache.';
COMMENT ON COLUMN cache_log.id IS 'Primary key sequence.';
COMMENT ON COLUMN cache_log.request_id IS 'Request correlation ID.';
COMMENT ON COLUMN cache_log.gateway_key_id IS 'Gateway key used by the request.';
COMMENT ON COLUMN cache_log.api_type IS 'API variant for this cache event.';
COMMENT ON COLUMN cache_log.cache_layer IS 'Cache layer where event happened: moka or database.';
COMMENT ON COLUMN cache_log.hit IS 'Whether cache lookup resulted in a hit.';
COMMENT ON COLUMN cache_log.request_body_hash IS 'Optional hash of request body for grouping and diagnostics.';
COMMENT ON COLUMN cache_log.request_body_size IS 'Request body size in bytes.';
COMMENT ON COLUMN cache_log.response_body_size IS 'Response body size in bytes when available.';
COMMENT ON COLUMN cache_log.latency_ms IS 'Cache lookup latency in milliseconds.';
COMMENT ON COLUMN cache_log.created_at IS 'Row creation timestamp.';
