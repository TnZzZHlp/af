# Agent Guide

This repo has a Rust backend (`backend/`) and a Vue 3 + Vite frontend (`frontend/`).
Use this file to stay aligned with existing build/lint/test workflows and code style.

## Rule Files

- Cursor rules: none found in `.cursor/rules/` or `.cursorrules`.
- Copilot rules: none found in `.github/copilot-instructions.md`.

If those files appear later, update this guide to reflect them.

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

Single-test workflow: no frontend test runner is configured in `package.json`.
If you add tests later, document the single-test command here.

### Backend (Rust + Axum + SQLx)

Run these from `backend/`.

- Build: `cargo build`
- Run: `cargo run`
- Check: `cargo check`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Test (all): `cargo test`
- Test (single): `cargo test <test_name>`
- Test (module): `cargo test <module>::<test_name>`

SQLx notes: no `sqlx` CLI scripts are defined; if you introduce migrations
or offline validation, add the commands here.

## Code Style Guidelines

### General

- Keep diffs small and focused; avoid drive-by refactors.
- Prefer explicit error handling over silent fallbacks.
- Avoid introducing new dependencies unless necessary.
- Keep strings and IDs in ASCII unless a file already uses Unicode.

### Rust Backend

#### Formatting and Imports

- Format with `cargo fmt` (no custom `rustfmt.toml` present).
- Use `use` groups similar to existing files: std -> external crates -> local.
- Avoid unused imports; keep `use` lists sorted and minimal.

#### Types and Error Handling

- Use `anyhow::Result` for fallible functions at service/db layers.
- Use the app error type in HTTP handlers: `AppResult<T>` and `AppError`.
- Prefer `AppError::BadRequest` for validation issues and `AppError::Internal`
  for unexpected failures.
- Log internal errors via `tracing::error!` (see `backend/src/error.rs`).

#### Naming and Structure

- Use `snake_case` for functions and modules, `CamelCase` for types.
- Keep handler functions thin; delegate to `services/` or `db/` modules.
- Prefer small, explicit structs for DB rows and API payloads.

#### API and Middleware

- Auth is handled via `Authorization: Bearer <key>` or `x-api-key` headers.
- Rate limit middleware currently reads limits but does not enforce them yet.
- Request logging records full bodies; be mindful of sensitive data.

#### SQLx Usage

- Use `sqlx::query!!` with bind parameters (no string interpolation).
- Map rows explicitly via `try_get` to avoid accidental schema drift.

### Vue Frontend

#### Formatting and Linting

- Formatting is handled by `oxfmt` (`pnpm format`).
- Linting uses `oxlint` and `eslint`; both are run by `pnpm lint`.
- ESLint uses Vue + TypeScript configs and `eslint-config-prettier`.

#### Imports and File Layout

- Use named imports from `vue`, `pinia`, `vue-router` as in current files.
- Keep imports at the top; separate external from local with a blank line.
- Prefer `./` relative imports for local modules.

#### Components and State

- Use `<script setup lang="ts">` for Vue SFCs.
- Keep components small; prefer composables/stores for shared logic.
- Pinia stores use the setup-style `defineStore` API.

#### Naming

- Use `PascalCase` for component names, `camelCase` for variables.
- Store IDs are lowercase strings (`defineStore('counter', ...)`).

#### Types and Validation

- Use explicit TypeScript types for public interfaces and store state.
- Avoid `any`; prefer `unknown` and narrow with guards if needed.

### Logging and Observability

- Backend uses `tracing` with `EnvFilter` from `RUST_LOG`.
- Log unexpected errors at the boundary; avoid noisy logs in hot paths.

### Config and Environment

- Backend config is loaded from env vars (see `backend/src/config.rs`).
- `SERVER_HOST` and `SERVER_PORT` are required; `DATABASE_URL` has a default.
- Document any new env vars or defaults in this file when added.

## Agent Checklist

- Identify which folder you are touching (`backend/` or `frontend/`).
- Run the relevant formatter and linter before handing off changes.
- Update this guide if you add new build/lint/test workflows or rules.
