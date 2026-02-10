
-- Add usage tracking columns to request_logs
ALTER TABLE request_logs
ADD COLUMN prompt_tokens integer,
ADD COLUMN completion_tokens integer,
ADD COLUMN total_tokens integer;

COMMENT ON COLUMN request_logs.prompt_tokens IS 'Number of tokens in the prompt.';
COMMENT ON COLUMN request_logs.completion_tokens IS 'Number of tokens in the completion.';
COMMENT ON COLUMN request_logs.total_tokens IS 'Total number of tokens used.';
