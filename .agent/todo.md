- [x] Remove tracked Python bytecode artefacts from `barter-python/tests_py/__pycache__`.
- [x] Finalize `shutdown_with_summary` bindings (Rust + Python tests + docs).
- [x] Ensure summary serialization returns rich Python objects.
- [x] Update packaging metadata (wheel classifiers, maturin settings) once API settles.
- [x] Plan integration tests covering live system lifecycle vs. backtest.
  - Captured in `.agent/specs/python-integration-tests.md` (2025-10-03).
- [x] Implement pytest integration suite per new spec.
  - [x] Add live system lifecycle coverage (`tests_py/test_integration_live.py`).
  - [x] Cover historic backtest summary scenario (`tests_py/test_integration_backtest.py`).
  - [x] Exercise command builder round-trip (`tests_py/test_integration_commands.py`).
  - [x] Capture failure surface behaviour (`tests_py/test_integration_failures.py`).
- [x] Wire integration test marker into CI workflow after maturin build.
- [x] Add tracing/log capture fixture to aid debugging slow tests.
- [x] Resolve `cargo test -p barter-python` linker failure caused by missing libpython symbols when
      building with the `extension-module` feature enabled. (2025-10-03)
- [x] Evaluate packaging automation
  - Documented follow-ups in `.agent/specs/python-packaging-automation.md` (2025-10-03).

- [x] Document new feature flag workflow in developer README once stabilised. (2025-10-03)

## Planned (2025-10-03)
- [x] Update top-level README with Python quickstart guidance. (2025-10-03)
- [x] Run `cargo test -p barter-python` *(blocked: linker fails to find libpython; see existing TODO)*
- [x] Run `pytest -q tests_py`
- [x] Expose account event constructors in Python bindings
- [x] Add coverage in tests for account event round trip
- [x] Document binding usage in README update

## Upcoming (2025-10-03)
- [x] Expose risk manager configuration knobs to Python API (implemented 2025-10-04; see tests in `barter-python/tests_py/test_risk_config.py`).
- [x] Surface portfolio analytics helpers (eg. Sharpe, Sortino calculators) for Python summaries. (2025-10-04)
- [x] Provide combined test runner script (Rust + Python) for contributors. (2025-10-03)
- [x] Draft release cadence doc aligning Rust crate and Python wheel versioning. (2025-10-04)
- [x] Add "Release Notes" aggregation section to `barter-python/README.md` post cadence adoption.

## Later Opportunities
- [x] Expand bindings for market stream events (2025-10-03)
- [x] Add CLI example for Python package (2025-10-03)
- [x] Automate publishing of built Python wheels once credentials are available (requires `PYPI_API_TOKEN` secret for release tags)

## Completed
- [x] Expose module version constant to Python consumers.
- [x] Expand `barter-python` bindings to cover engine configuration and system control.
- [x] Mirror key configuration structs (SystemConfig) in Python API.
- [x] Provide runtime helpers to run trading system from Python via async tasks.
- [x] Add Python-level integration tests exercising basic system lifecycle.
- [x] Expose account reconnect events in Python API (2025-10-03).
- [x] Expose abort helper on Python `SystemHandle` for immediate teardown (2025-10-03).
- [x] Expose additional market event constructors (L1 order book, candle, liquidation) in Python bindings (2025-10-03).
- [x] Add Python binding helper for account trade events (2025-10-03).
- [x] Allow selecting annualisation interval for Python trading summaries (2025-10-03).
- [x] Capture a release checklist for Python publishing (see `.agent/specs/python-release-checklist.md`, 2025-10-03).
- [x] Expose account order snapshot & cancellation helpers in Python bindings (2025-10-03).

## In Progress (2025-10-04)
- [x] Expose Python helpers for profit factor and win rate calculations (bindings + pytest coverage). (2025-10-04)
- [x] Add `calculate_rate_of_return` with optional target interval scaling (bindings + pytest coverage). (2025-10-04)
- [x] Refresh package metadata (version bump + release notes) once analytics helpers land. (2025-10-04)

## Upcoming (2025-10-04)
- [x] Allow seeding initial balances when starting systems or running historic backtests from Python bindings. (2025-10-04)
