-- Keep request_body_hash managed by application code so cache-key semantics
-- are consistent between query path and write path.
DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'request_logs'
      AND column_name = 'request_body_hash'
      AND is_generated = 'ALWAYS'
  ) THEN
    ALTER TABLE request_logs
      ALTER COLUMN request_body_hash DROP EXPRESSION;
  END IF;
END $$;

COMMENT ON COLUMN request_logs.request_body_hash IS
  'Application-managed deterministic hash of normalized request JSON used as cache key.';
