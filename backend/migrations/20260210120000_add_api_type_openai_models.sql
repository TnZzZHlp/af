-- Add openai_models to api_type enum
ALTER TYPE api_type ADD VALUE IF NOT EXISTS 'openai_models';
