DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'provider_endpoints'
      AND column_name = 'url'
  ) THEN
    ALTER TABLE provider_endpoints ADD COLUMN url text;
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'provider_endpoints'
      AND column_name = 'base_url'
  ) AND EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'provider_endpoints'
      AND column_name = 'path'
  ) THEN
    EXECUTE $sql$
      UPDATE provider_endpoints
      SET url = rtrim(base_url, '/') || '/' || ltrim(path, '/')
      WHERE url IS NULL
    $sql$;
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'provider_endpoints'
      AND column_name = 'url'
  ) AND NOT EXISTS (
    SELECT 1
    FROM provider_endpoints
    WHERE url IS NULL
  ) THEN
    ALTER TABLE provider_endpoints ALTER COLUMN url SET NOT NULL;
  END IF;
END $$;

ALTER TABLE provider_endpoints
  DROP CONSTRAINT IF EXISTS provider_endpoints_provider_id_api_type_base_url_path_key;

ALTER TABLE provider_endpoints
  DROP COLUMN IF EXISTS base_url;

ALTER TABLE provider_endpoints
  DROP COLUMN IF EXISTS path;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'provider_endpoints_provider_id_api_type_url_key'
      AND conrelid = 'provider_endpoints'::regclass
  ) THEN
    ALTER TABLE provider_endpoints
      ADD CONSTRAINT provider_endpoints_provider_id_api_type_url_key
      UNIQUE (provider_id, api_type, url);
  END IF;
END $$;

COMMENT ON COLUMN provider_endpoints.url IS 'Full endpoint URL including path.';
