# Agent Guide

This repo has a Rust backend (`backend/`) and a Vue 3 + Vite frontend (`frontend/`).
Use this file to stay aligned with existing build/lint/test workflows and code style.

## Rule Files

- Cursor rules: none found in `.cursor/rules/` or `.cursorrules`.
- Copilot rules: none found in `.github/copilot-instructions.md`.

## Build, Lint, Test Commands

### Frontend (Vite + Vue)

Run these from `frontend/`.

- Install: `pnpm install`
- Dev server: `pnpm dev`
- Build (type-check + build): `pnpm build`
- Build only: `pnpm build-only`
- Type-check only: `pnpm type-check`
- Lint (all): `pnpm lint`
- Lint (oxlint only): `pnpm lint:oxlint`
- Lint (eslint only): `pnpm lint:eslint`
- Format: `pnpm format`

No frontend test runner configured. Document test commands if added.

### Backend (Rust + Axum + SQLx)

Run these from `backend/`.

- Build: `cargo build`
- Build (Release): `cargo build --release`
- Run: `cargo run`
- Check: `cargo check`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Test (all): `cargo test`
- Test (single): `cargo test <test_name>`
- Test (module): `cargo test <module>::<test_name>`

No tests exist yet. Add tests via `#[cfg(test)] mod tests { ... }` or `tests/` directory.

#### SQLx Workflows

Project uses `sqlx` with offline mode (`.sqlx` directory).

- Run migrations: `sqlx migrate run` (requires `DATABASE_URL`)
- Create migration: `sqlx migrate add <description>`
- Update offline data: `cargo sqlx prepare` (after changing queries)
- Check offline data: `cargo sqlx prepare --check`

## Code Style Guidelines

### General

- Keep diffs small and focused; avoid drive-by refactors.
- Prefer explicit error handling over silent fallbacks.
- Avoid introducing new dependencies unless necessary.
- Keep strings and IDs in ASCII unless a file already uses Unicode.

### Rust Backend

#### Formatting and Imports

- Format with `cargo fmt` (no custom `rustfmt.toml`).
- Group imports: std -> external crates -> local modules (crate::).
- Example:
  ```rust
  use std::net::SocketAddr;
  
  use axum::{Json, extract::State};
  use serde::Deserialize;
  use uuid::Uuid;
  
  use crate::{error::AppResult, state::AppState};
  ```

#### Types and Error Handling

- Use `anyhow::Result` for service/db layers.
- Use `AppResult<T>` and `AppError` in HTTP handlers.
- Error variants: `BadRequest`, `Unauthorized`, `Forbidden`, `TooManyRequests`,
  `NotFound`, `Internal`.
- Prefer `AppError::BadRequest` for validation, `AppError::Internal` for failures.
- Log internal errors via `tracing::error!` (see `backend/src/error.rs`).

#### Naming and Structure

- `snake_case` for functions/modules, `CamelCase` for types.
- Keep handlers thin; delegate to `services/` or `db/` modules.
- Request structs: `CreateXxxRequest`, `UpdateXxxRequest`, `ListXxxQuery`.
- DB parameter structs: `CreateXxxParams`, `UpdateXxxParams`.

#### API and Middleware

- Auth via `Authorization: Bearer <key>` or `x-api-key` headers.
- Middleware order (innermost first): auth -> rate_limit -> response_cache.
- Extract authenticated user via `GatewayKeyId` extension in handlers.
- Request logging records full bodies; be mindful of sensitive data.

#### SQLx Usage

- Use `sqlx::query!` and `sqlx::query_as!` for compile-time verification.
- Run `cargo sqlx prepare` after modifying queries.
- Use bind parameters (no string interpolation).
- Map rows explicitly via struct construction:
  ```rust
  let row = sqlx::query!("SELECT ...", id).fetch_optional(pool).await?;
  Ok(row.map(|r| GatewayKey { id: r.id, ... }))
  ```

#### Module Organization

- `main.rs`: CLI parsing, config, DB pool, service init.
- `router.rs`: Route definitions and middleware stacking.
- `handlers/`: HTTP handlers (thin, delegate to services).
- `services/`: Business logic, orchestration.
- `db/`: Database queries and model structs.
- `middleware/`: Axum middleware functions.
- `state.rs`: Application state struct.

### Vue Frontend

#### Formatting and Linting

- Format with `oxfmt` (`pnpm format`).
- Lint with `oxlint` and `eslint` (`pnpm lint`).
- Oxlint plugins: eslint, typescript, unicorn, oxc, vue.

#### Imports and File Layout

- Use named imports from `vue`, `pinia`, `vue-router`.
- Use `@/` alias for src imports.
- Example:
  ```typescript
  import { ref, computed, onMounted } from 'vue'
  import { useStatsStore } from '@/stores/stats'
  import { Card, CardContent } from '@/components/ui/card'
  ```

#### Components and State

- Use `<script setup lang="ts">` for Vue SFCs.
- Pinia stores use setup-style `defineStore` with arrow functions.
- Store pattern: reactive refs for state, async functions for actions.

#### Naming

- `PascalCase` for components, `camelCase` for variables.
- Store IDs: lowercase strings (`defineStore('auth', ...)`).
- Page components: `XxxPage.vue`, Layouts: `XxxLayout.vue`.

#### Types and API Layer

- Use explicit TypeScript types; avoid `any`.
- API functions in `src/api/` use `requestJson<T>()` from `client.ts`.
- Auth token auto-attached from localStorage.
- Catch `ApiError` and extract `message` property for errors.
- Use Zod for runtime validation (already a dependency).

### Logging and Config

- Backend uses `tracing` with `EnvFilter` from `RUST_LOG`.
- CLI args: `--log-level` (default: info) or `--log-filter` for full control.
- Config from env vars: `SERVER_HOST`, `SERVER_PORT` (required).
- Optional: `DATABASE_URL`, `DATABASE_MAX_CONNECTIONS`, `JWT_SECRET` (have defaults).

## Agent Checklist

- Identify which folder you are touching (`backend/` or `frontend/`).
- Run the relevant formatter and linter before handing off changes.
- After modifying SQL queries in backend, run `cargo sqlx prepare`.
- Update this guide if you add new build/lint/test workflows or rules.
