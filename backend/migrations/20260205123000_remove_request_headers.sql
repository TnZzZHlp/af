ALTER TABLE request_logs
  DROP COLUMN IF EXISTS request_headers;

ALTER TABLE request_logs
  DROP COLUMN IF EXISTS response_headers;
