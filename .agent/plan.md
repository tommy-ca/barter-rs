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
- Introduce typed Python wrappers for engine `ActionOutput` results produced by audit streams.
  - [ ] Capture spec under `.agent/specs/python-engine-action-output-bindings.md`.
  - [ ] Add TDD coverage (Rust + Python) for the new wrappers.
  - [ ] Implement PyO3 bindings and expose modules to Python consumers.

## Completed (2025-10-05)
- Add integration coverage for backtest argument wrappers.
  - [x] Added scenario 5 to `.agent/specs/python-integration-tests.md`.
  - [x] Implemented `test_backtest_argument_wrappers_end_to_end` exercising `backtest` and `run_backtests`.
- Expose typed audit tick bindings to Python consumers.
  - [x] Document requirements under `.agent/specs/python-audit-tick-bindings.md`.
  - [x] Add `PyAuditContext`, `PyAuditEvent`, and `PyAuditTick` with `recv_tick` helpers on `AuditUpdates`.
  - [x] Extend integration coverage exercising typed audit flows alongside dict summaries.
  - [x] Update README audit example to demonstrate the typed helpers.
- Bridge backtest argument wrappers into the Python API.
  - [x] Update `.agent/specs/python-backtest-bindings.md` with milestone breakdown.
  - [x] Add `PyBacktestArgsConstant` & `PyBacktestArgsDynamic` with validation and market data coercion.
  - [x] Re-export wrappers via `python/backtest.py` and align pytest expectations (`TestBacktestArgs`).
  - [x] Run `cargo test -p barter-python` and `uv run pytest tests_py/test_backtest.py` post-change.
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
