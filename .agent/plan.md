# Plan (2025-10-05)

## Objectives
- Ensure Python bindings exist for all barter-rs crates (engine, data, execution, instrument, integration, macro).
- Provide end-to-end tests demonstrating integrations through Python API.
- Maintain TDD focus with incremental commits per atomic change.

## Initial Steps
1. Audit existing Rust crates and current PyO3 bindings coverage.
2. Define bridging strategy for missing crates and shared abstractions.
3. Implement bindings crate-by-crate with accompanying tests and docs.
4. Publish and verify via `uv` environment using example E2E flows.

## Current Focus (2025-10-05)
- Bridge execution order enums into the Python API.
  - [x] Capture binding requirements in `.agent/specs/python-order-enum-bindings.md`.
  - [x] Expose PyO3 `OrderKind` and `TimeInForce` wrappers and re-export via `execution.py`.
  - [x] Extend pytest and Rust coverage for the new wrappers (cargo test, uv run pytest on 2025-10-05).
- Expand execution instrument mapping coverage to the Python API.
  - [x] Capture binding requirements in `.agent/specs/python-execution-instrument-map.md`.
  - [x] Expose `ExecutionInstrumentMap` wrappers & generator functions via PyO3.
  - [x] Add pytest coverage validating lookup helpers and error handling.
- [x] Replace pure Python instrument name wrappers with Rust-backed bindings and align pytest
  coverage (`.agent/specs/python-instrument-name-bindings.md`).
- Bridge mock execution client lifecycle into the Python API.
  - [x] Capture requirements in `.agent/specs/python-mock-execution-client.md`.
  - [x] Expose PyO3 bindings & Python wrapper for a `MockExecutionClient` harness.
  - [x] Add TDD coverage (pytest + doctest) exercising snapshots and stream polling.

## Completed (2025-10-05)
- Expose execution balance wrappers to the Python API.
  - [x] Align module exports with the Rust-backed `Balance` and `AssetBalance` bindings.
  - [x] Update pytest coverage asserting execution types are surfaced to Python callers.
  - [x] Run `uv run pytest -q tests_py` and `cargo test -p barter-python` after the migration.

## Completed (2025-10-04)
- Replace Python close-position helpers with Rust-backed strategy bindings.
  - [x] Finalise spec requirements under `.agent/specs/python-strategy-bindings.md`.
  - [x] Add pytest coverage asserting parity with existing helpers and error handling.
  - [x] Implement PyO3 bindings and update Python module exports.
  - [x] Verify ergonomics through strategy integration tests and docs refresh.

## Notes
- Use `.agent/specs/` to capture crate-specific requirements.
- Leverage existing Rust modules; avoid rewriting logic in Python.
- Prioritize SOLID, KISS, DRY principles; keep interfaces small.
