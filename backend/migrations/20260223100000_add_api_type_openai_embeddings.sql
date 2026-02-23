-- Add openai_embeddings to api_type enum
ALTER TYPE api_type ADD VALUE IF NOT EXISTS 'openai_embeddings';
