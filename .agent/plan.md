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

## Current Focus (2025-10-04)
- Bridge full account snapshot events from `barter-execution` into the Python API.
  - [x] Document binding requirements under `.agent/specs/python-account-snapshot-bindings.md`.
  - [x] Add failing pytest coverage capturing snapshot round-trips and validation errors.
  - [x] Implement new PyO3 wrappers and expose them via the extension module.
  - [x] Enforce instrument & exchange alignment checks in the PyO3 constructors.

## Notes
- Use `.agent/specs/` to capture crate-specific requirements.
- Leverage existing Rust modules; avoid rewriting logic in Python.
- Prioritize SOLID, KISS, DRY principles; keep interfaces small.
