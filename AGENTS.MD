# AI Gateway Plan

## External APIs

- OpenAI: POST /v1/chat/completions, POST /v1/responses (return OpenAI format)
- Anthropic: POST /v1/messages (return Anthropic format)

## Auth and Rate Limits

- Require gateway API key (Authorization: Bearer <key> or x-api-key)
- Rate limit per gateway key only (rps + rpm)

## Model Naming and Whitelist

- Accept model name as "provider:real_model" or alias
- Enforce whitelist exact match per gateway key (gateway_key_models)
- provider:real_model must match request api_type

## Routing, Load Balancing, Failover

- Determine api_type from request path
- Alias -> targets filtered by api_type, enabled, and circuit closed
- Alias strategy: weighted round robin (default) or priority/round_robin
- Provider key pool bound to provider; select key by weighted round robin
- Failover order: next key, then next target
- Failure criteria: timeout, 5xx, 429; 4xx (except 429) does not increment fail_count
- Circuit breaker: fail_count >= 3 opens for 60s (configurable)

## Providers

- OpenAI: chat/completions and responses endpoints
- Anthropic: messages endpoint
- Return native response format per API type

## Logging

- request_logs stores full request/response headers and bodies
- Log metadata: request_id, gateway_key_id, api_type, model, alias, provider, endpoint, status_code, latency_ms, error_code, error_message, client_ip, user_agent, created_at
- No encryption; retention indefinite
- Indexes: created_at, gateway_key_id, status_code, model

## Database Tables

- providers
- provider_endpoints
- provider_keys (bound to provider)
- models
- aliases
- alias_targets
- gateway_keys (rate_limit_rps, rate_limit_rpm)
- gateway_key_models (exact match whitelist)
- request_logs

## Admin API and Vue Console

- CRUD: providers, endpoints, provider keys, models, aliases, targets, gateway keys, model whitelist
- Log query with filters + pagination

## Implementation Order

1. SQLx migrations and models
2. Axum routes + middleware (auth, rate limit, request_id, logging)
3. Provider adapters (OpenAI chat/responses, Anthropic messages)
4. Routing + load balancing + failover + circuit breaker
5. SSE streaming + full-body logging
6. Admin API + Vue UI
