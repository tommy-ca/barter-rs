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

- [x] Expose audit snapshot streaming to Python API (spec, bindings, tests, docs). (2025-10-04)

## Active (2025-10-04)
- [x] Bridge integration Snapshot types through Rust bindings and replace the pure Python fallback. (2025-10-04)
  - [x] Implement `PySnapshot` and `PySnapUpdates` wrappers in the extension module.
  - [x] Update `python/barter_python/integration.py` to re-export the bindings.
  - [x] Extend pytest coverage for Snapshot round-trips.
- [x] Implement Python engine account event processing to update balances, orders, and positions from execution events. (2025-10-04)
- [x] Add pytest coverage for account event handling (snapshot, balance updates, order cancellations, trades). (2025-10-04)
- [x] Replace Python close-position helpers with Rust-backed bindings (spec: `.agent/specs/python-strategy-bindings.md`).
  - [x] Add pytest coverage for Rust-backed close position helpers.
  - [x] Implement PyO3 strategy bindings and wire through Python package.
  - [x] Validate ergonomics & docs for strategy helpers post binding switch.

## Planned (2025-10-03)
- [x] Update top-level README with Python quickstart guidance. (2025-10-04)
- [x] Run `cargo test -p barter-python` *(blocked: linker fails to find libpython; see existing TODO)*
- [x] Run `pytest -q tests_py`
- [x] Expose account event constructors in Python bindings
- [x] Add coverage in tests for account event round trip
- [x] Document binding usage in README update

## Upcoming (2025-10-03)
- [x] Expose risk manager configuration knobs to Python API (implemented 2025-10-04; see tests in `barter-python/tests_py/test_risk_config.py`).
- [x] Surface portfolio analytics helpers (eg. Sharpe, Sortino calculators) for Python summaries. (2025-10-04)
- [x] Provide combined test runner script (Rust + Python) for contributors. (2025-10-03) ✅
- [x] Draft release cadence doc aligning Rust crate and Python wheel versioning. (2025-10-04)
- [x] Add "Release Notes" aggregation section to `barter-python/README.md` post cadence adoption. (2025-10-04)

 ## Later Opportunities
 - [x] Expand bindings for market stream events (2025-10-03)
   - Added market_order_book_snapshot constructor (2025-10-04)
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
- [x] Expose drawdown analytics helpers (`generate_drawdown_series`,
      `calculate_max_drawdown`, `calculate_mean_drawdown`) in Python bindings
      (2025-10-04).

 ## Completed (2025-10-04)
- [x] Expose Python helpers for profit factor and win rate calculations (bindings + pytest coverage). (2025-10-04)
- [x] Add `calculate_rate_of_return` with optional target interval scaling (bindings + pytest coverage). (2025-10-04)
- [x] Refresh package metadata (version bump + release notes) once analytics helpers land. (2025-10-04)
- [x] Complete pure Python port of barter-statistic module with comprehensive tests (77 tests covering all metrics, drawdown analytics, and time intervals). (2025-10-04)

 ## Completed (2025-10-04)
 - [x] Allow seeding initial balances when starting systems or running historic backtests from Python bindings. (2025-10-04)
 - [x] Complete pure Python port of barter-data structures (Candle, Liquidation, OrderBook, OrderBookSide) with comprehensive tests. (2025-10-04)
 - [x] Fix linting issues in Python and Rust codebases (unused imports, variables, clippy warnings). (2025-10-04)
 - [x] Update Python type annotations to use modern X | None syntax instead of Optional[X]. (2025-10-04)
 - [x] Add init_json_logging_py binding for JSON structured logging. (2025-10-04)
  - [x] Replace Python JSON parsing with Rust-backed MarketDataInMemory bindings and integration tests. (2025-10-04)
