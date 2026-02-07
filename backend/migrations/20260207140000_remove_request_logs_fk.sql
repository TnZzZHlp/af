-- Remove foreign key constraint from request_logs table
ALTER TABLE request_logs DROP CONSTRAINT IF EXISTS request_logs_gateway_key_id_fkey;
