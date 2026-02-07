-- Add content type columns to request_logs
ALTER TABLE request_logs
ADD COLUMN request_content_type text,
ADD COLUMN response_content_type text;

COMMENT ON COLUMN request_logs.request_content_type IS 'MIME type of the request body.';
COMMENT ON COLUMN request_logs.response_content_type IS 'MIME type of the response body.';
