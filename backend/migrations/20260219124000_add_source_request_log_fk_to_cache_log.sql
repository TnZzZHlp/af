-- Link cache hit event to the source request_logs row that provided the cached response.
ALTER TABLE cache_log
  ADD COLUMN IF NOT EXISTS source_request_log_id bigint;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'cache_log_source_request_log_id_fkey'
  ) THEN
    ALTER TABLE cache_log
      ADD CONSTRAINT cache_log_source_request_log_id_fkey
      FOREIGN KEY (source_request_log_id)
      REFERENCES request_logs(id)
      ON DELETE SET NULL;
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_cache_log_source_request_log_id
  ON cache_log(source_request_log_id);

COMMENT ON COLUMN cache_log.source_request_log_id IS
  'Foreign key to request_logs.id for the cached response source row.';
