# Plan (2025-10-04)

## Objectives
- Ensure Python bindings exist for all barter-rs crates (engine, data, execution, instrument, integration, macro).
- Provide end-to-end tests demonstrating integrations through Python API.
- Maintain TDD focus with incremental commits per atomic change.

## Initial Steps
1. Audit existing Rust crates and current PyO3 bindings coverage.
2. Define bridging strategy for missing crates and shared abstractions.
3. Implement bindings crate-by-crate with accompanying tests and docs.
4. Publish and verify via `uv` environment using example E2E flows.

## Notes
- Use `.agent/specs/` to capture crate-specific requirements.
- Leverage existing Rust modules; avoid rewriting logic in Python.
- Prioritize SOLID, KISS, DRY principles; keep interfaces small.
