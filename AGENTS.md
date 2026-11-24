# Repository Guidelines

## Project Structure & Module Organization

- The root Rust 2024 workspace contains Cloud Run binaries under `crates/functions/`, shared service code in `crates/shared/` and `crates/shared-function/`, and application errors in `crates/app-error/`.
- Keep SeaORM entities, repositories, and migrations under `crates/database/`. The separate Anchor workspace in `pvault/` contains the on-chain program. Keep its instruction code in `programs/pvault/src/instructions/` and integration tests in `programs/pvault/tests/`, relative to that workspace.
- Pulumi infrastructure lives in `infra/`, the service image build in `docker/Dockerfile`, and CI workflows in `.github/workflows/`.

## Build, Test, and Development Commands

- `cargo build --locked --workspace` builds the service workspace. With a local PostgreSQL database, run `cargo run -p migration -- up`, then `cargo run -p api`. Set `DATABASE_URL` for both and `PORT` for the API.
- Run `cargo +nightly fmt --all --check` and `cargo +stable clippy --locked --workspace -- -D warnings` from both the root and `pvault/` to reproduce the Rust CI checks.
- From `infra/`, run `pnpm install --frozen-lockfile` with Node.js 24, then `pnpm run format:check` and `pnpm run lint`. See the infrastructure guide for local previews and stack configuration. [Infrastructure guide](infra/README.md)

## Coding Style & Naming Conventions

Use four-space Rust indentation, `snake_case` for functions and modules, and `UpperCamelCase` for types. Keep declarations local or private where possible and use plain `pub` for items needed outside their defining module. Follow the repository rules for dependency features, internal crate names, and external errors. [Code style](code_style.md)

## Testing Guidelines

Run `cargo test --locked --workspace` for the service workspace. From `pvault/`, run `anchor test` to invoke the configured SBF test command. Anchor integration tests use Mollusk and shared fixtures in `tests/common/`. Name tests for behavior, such as `deposit_transfers_tokens_from_a_non_authority_wallet`. Cover affected behavior when changing program logic.

## Commit & Pull Request Guidelines

Use short, action-oriented commit subjects such as `enable http2 h2c`. In pull requests, describe the change and its verification, link related issues when available, and call out migration or deployment effects.
