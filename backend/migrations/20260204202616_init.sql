-- Database bootstrap for AI gateway service.
-- Uses PostgreSQL 18 built-in uuidv7() generation and defines enums for API types
-- and load-balancing strategy.

-- API surface variants supported by the gateway.
DO $$
BEGIN
  CREATE TYPE api_type AS ENUM (
    'openai_chat_completions',
    'openai_responses',
    'anthropic_messages'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- Load-balancing strategy for alias routing.
DO $$
BEGIN
  CREATE TYPE lb_strategy AS ENUM (
    'weighted_round_robin',
    'round_robin',
    'priority'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- Providers are logical vendors (OpenAI, Anthropic, etc.).
CREATE TABLE IF NOT EXISTS providers (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  name text NOT NULL UNIQUE,
  description text,
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

-- Provider endpoints are concrete URLs for a specific api_type.
CREATE TABLE IF NOT EXISTS provider_endpoints (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  provider_id uuid NOT NULL REFERENCES providers(id),
  api_type api_type NOT NULL,
  url text NOT NULL,
  weight integer NOT NULL DEFAULT 1 CHECK (weight > 0),
  priority integer NOT NULL DEFAULT 0,
  timeout_ms integer NOT NULL DEFAULT 60000 CHECK (timeout_ms > 0),
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_id, api_type, url)
);

-- Speed up endpoint lookup by provider and api_type.
CREATE INDEX IF NOT EXISTS idx_provider_endpoints_provider
  ON provider_endpoints(provider_id);

CREATE INDEX IF NOT EXISTS idx_provider_endpoints_api_type
  ON provider_endpoints(api_type);

-- Provider keys are bound to a provider and shared across endpoints.
-- Circuit breaker state is tracked per key.
CREATE TABLE IF NOT EXISTS provider_keys (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  provider_id uuid NOT NULL REFERENCES providers(id),
  name text,
  key text NOT NULL UNIQUE,
  weight integer NOT NULL DEFAULT 1 CHECK (weight > 0),
  enabled boolean NOT NULL DEFAULT true,
  fail_count integer NOT NULL DEFAULT 0 CHECK (fail_count >= 0),
  circuit_open_until timestamptz,
  last_fail_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_provider_keys_provider
  ON provider_keys(provider_id);

CREATE INDEX IF NOT EXISTS idx_provider_keys_enabled
  ON provider_keys(enabled);

-- Models are provider-specific and constrained by api_type.
CREATE TABLE IF NOT EXISTS models (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  provider_id uuid NOT NULL REFERENCES providers(id),
  api_type api_type NOT NULL,
  name text NOT NULL,
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_id, api_type, name)
);

CREATE INDEX IF NOT EXISTS idx_models_provider
  ON models(provider_id);

CREATE INDEX IF NOT EXISTS idx_models_api_type
  ON models(api_type);

-- Aliases are user-facing model names with routing strategy per api_type.
CREATE TABLE IF NOT EXISTS aliases (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  name text NOT NULL UNIQUE,
  api_type api_type NOT NULL,
  strategy lb_strategy NOT NULL DEFAULT 'weighted_round_robin',
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_aliases_api_type
  ON aliases(api_type);

-- Alias targets map alias -> provider endpoint -> model.
-- Circuit breaker state is tracked per target.
CREATE TABLE IF NOT EXISTS alias_targets (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  alias_id uuid NOT NULL REFERENCES aliases(id),
  provider_endpoint_id uuid NOT NULL REFERENCES provider_endpoints(id),
  model_id uuid NOT NULL REFERENCES models(id),
  weight integer NOT NULL DEFAULT 1 CHECK (weight > 0),
  priority integer NOT NULL DEFAULT 0,
  enabled boolean NOT NULL DEFAULT true,
  fail_count integer NOT NULL DEFAULT 0 CHECK (fail_count >= 0),
  circuit_open_until timestamptz,
  last_fail_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (alias_id, provider_endpoint_id, model_id)
);

CREATE INDEX IF NOT EXISTS idx_alias_targets_alias
  ON alias_targets(alias_id);

CREATE INDEX IF NOT EXISTS idx_alias_targets_enabled
  ON alias_targets(enabled);

-- Gateway keys represent customers; rate limits are applied per key only.
CREATE TABLE IF NOT EXISTS gateway_keys (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  name text,
  key text NOT NULL UNIQUE,
  enabled boolean NOT NULL DEFAULT true,
  rate_limit_rps integer CHECK (rate_limit_rps >= 0),
  rate_limit_rpm integer CHECK (rate_limit_rpm >= 0),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_gateway_keys_enabled
  ON gateway_keys(enabled);

-- Users are gateway administrators/operators with password auth.
CREATE TABLE IF NOT EXISTS users (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  email text NOT NULL UNIQUE,
  name text,
  password_hash text NOT NULL,
  password_updated_at timestamptz,
  enabled boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_users_enabled
  ON users(enabled);

-- Exact model whitelist per gateway key.
-- Entries are matched verbatim against request model string.
CREATE TABLE IF NOT EXISTS gateway_key_models (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  gateway_key_id uuid NOT NULL REFERENCES gateway_keys(id),
  model text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (gateway_key_id, model)
);

CREATE INDEX IF NOT EXISTS idx_gateway_key_models_key
  ON gateway_key_models(gateway_key_id);

-- Full request/response logs with metadata and raw payloads.
-- Bodies stored as bytea to preserve exact bytes.
CREATE TABLE IF NOT EXISTS request_logs (
  id bigserial PRIMARY KEY,
  request_id uuid NOT NULL,
  gateway_key_id uuid REFERENCES gateway_keys(id),
  api_type api_type NOT NULL,
  model text,
  alias text,
  provider text,
  endpoint text,
  status_code integer,
  latency_ms integer,
  client_ip inet,
  user_agent text,
  request_body bytea,
  response_body bytea,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_request_logs_created_at
  ON request_logs(created_at);

CREATE INDEX IF NOT EXISTS idx_request_logs_gateway_key
  ON request_logs(gateway_key_id);

CREATE INDEX IF NOT EXISTS idx_request_logs_status_code
  ON request_logs(status_code);

CREATE INDEX IF NOT EXISTS idx_request_logs_model
  ON request_logs(model);

COMMENT ON TYPE api_type IS 'Supported API variants exposed by the gateway.';
COMMENT ON TYPE lb_strategy IS 'Load-balancing strategies for alias routing.';

COMMENT ON TABLE providers IS 'Logical vendors (OpenAI, Anthropic, etc.).';
COMMENT ON COLUMN providers.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN providers.name IS 'Provider name, unique.';
COMMENT ON COLUMN providers.description IS 'Optional description for admin UI.';
COMMENT ON COLUMN providers.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN providers.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE provider_endpoints IS 'Concrete URL for a provider and api_type.';
COMMENT ON COLUMN provider_endpoints.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN provider_endpoints.provider_id IS 'Owning provider.';
COMMENT ON COLUMN provider_endpoints.api_type IS 'API variant for this endpoint.';
COMMENT ON COLUMN provider_endpoints.url IS 'Full endpoint URL including path.';
COMMENT ON COLUMN provider_endpoints.weight IS 'Weight for load balancing.';
COMMENT ON COLUMN provider_endpoints.priority IS 'Priority for priority-based routing.';
COMMENT ON COLUMN provider_endpoints.timeout_ms IS 'Request timeout in milliseconds.';
COMMENT ON COLUMN provider_endpoints.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN provider_endpoints.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE provider_keys IS 'Provider API keys bound to a provider.';
COMMENT ON COLUMN provider_keys.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN provider_keys.provider_id IS 'Owning provider.';
COMMENT ON COLUMN provider_keys.name IS 'Optional label for admin UI.';
COMMENT ON COLUMN provider_keys.key IS 'Secret key value, unique.';
COMMENT ON COLUMN provider_keys.weight IS 'Weight for key selection.';
COMMENT ON COLUMN provider_keys.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN provider_keys.fail_count IS 'Consecutive failures for circuit breaker.';
COMMENT ON COLUMN provider_keys.circuit_open_until IS 'If set, key is in open circuit until this time.';
COMMENT ON COLUMN provider_keys.last_fail_at IS 'Timestamp of last failure.';
COMMENT ON COLUMN provider_keys.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE models IS 'Provider-specific models constrained by api_type.';
COMMENT ON COLUMN models.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN models.provider_id IS 'Owning provider.';
COMMENT ON COLUMN models.api_type IS 'API variant for this model.';
COMMENT ON COLUMN models.name IS 'Provider model name.';
COMMENT ON COLUMN models.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN models.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE aliases IS 'User-facing model aliases with routing strategy.';
COMMENT ON COLUMN aliases.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN aliases.name IS 'Alias name, unique.';
COMMENT ON COLUMN aliases.api_type IS 'API variant allowed for this alias.';
COMMENT ON COLUMN aliases.strategy IS 'Load-balancing strategy.';
COMMENT ON COLUMN aliases.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN aliases.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE alias_targets IS 'Alias to endpoint+model mapping with routing metadata.';
COMMENT ON COLUMN alias_targets.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN alias_targets.alias_id IS 'Owning alias.';
COMMENT ON COLUMN alias_targets.provider_endpoint_id IS 'Target provider endpoint.';
COMMENT ON COLUMN alias_targets.model_id IS 'Target model.';
COMMENT ON COLUMN alias_targets.weight IS 'Weight for load balancing.';
COMMENT ON COLUMN alias_targets.priority IS 'Priority for priority-based routing.';
COMMENT ON COLUMN alias_targets.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN alias_targets.fail_count IS 'Consecutive failures for circuit breaker.';
COMMENT ON COLUMN alias_targets.circuit_open_until IS 'If set, target is in open circuit until this time.';
COMMENT ON COLUMN alias_targets.last_fail_at IS 'Timestamp of last failure.';
COMMENT ON COLUMN alias_targets.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE gateway_keys IS 'Customer API keys with per-key rate limits.';
COMMENT ON COLUMN gateway_keys.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN gateway_keys.name IS 'Optional label for admin UI.';
COMMENT ON COLUMN gateway_keys.key IS 'Secret key value, unique.';
COMMENT ON COLUMN gateway_keys.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN gateway_keys.rate_limit_rps IS 'Requests per second limit; NULL means unlimited.';
COMMENT ON COLUMN gateway_keys.rate_limit_rpm IS 'Requests per minute limit; NULL means unlimited.';
COMMENT ON COLUMN gateway_keys.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE users IS 'Gateway administrators/operators with password auth.';
COMMENT ON COLUMN users.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN users.email IS 'Unique email address.';
COMMENT ON COLUMN users.name IS 'Optional display name.';
COMMENT ON COLUMN users.password_hash IS 'Hashed password (argon2/bcrypt/etc).';
COMMENT ON COLUMN users.password_updated_at IS 'Timestamp when password last changed.';
COMMENT ON COLUMN users.enabled IS 'Soft enable/disable flag.';
COMMENT ON COLUMN users.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE gateway_key_models IS 'Exact model whitelist per gateway key.';
COMMENT ON COLUMN gateway_key_models.id IS 'Primary key UUIDv7.';
COMMENT ON COLUMN gateway_key_models.gateway_key_id IS 'Owning gateway key.';
COMMENT ON COLUMN gateway_key_models.model IS 'Exact model string allowed for the key.';
COMMENT ON COLUMN gateway_key_models.created_at IS 'Row creation timestamp.';

COMMENT ON TABLE request_logs IS 'Full request/response logs with metadata and raw payloads.';
COMMENT ON COLUMN request_logs.id IS 'Primary key sequence.';
COMMENT ON COLUMN request_logs.request_id IS 'Generated request id for correlation.';
COMMENT ON COLUMN request_logs.gateway_key_id IS 'Gateway key used for the request.';
COMMENT ON COLUMN request_logs.api_type IS 'API variant for this request.';
COMMENT ON COLUMN request_logs.model IS 'Requested model string.';
COMMENT ON COLUMN request_logs.alias IS 'Matched alias name, if any.';
COMMENT ON COLUMN request_logs.provider IS 'Provider name used to fulfill request.';
COMMENT ON COLUMN request_logs.endpoint IS 'Full endpoint URL used for the request.';
COMMENT ON COLUMN request_logs.status_code IS 'Upstream response status code.';
COMMENT ON COLUMN request_logs.latency_ms IS 'End-to-end latency in milliseconds.';
COMMENT ON COLUMN request_logs.client_ip IS 'Client IP address.';
COMMENT ON COLUMN request_logs.user_agent IS 'Client user-agent string.';
COMMENT ON COLUMN request_logs.request_body IS 'Raw request body bytes.';
COMMENT ON COLUMN request_logs.response_body IS 'Raw response body bytes.';
COMMENT ON COLUMN request_logs.created_at IS 'Row creation timestamp.';
