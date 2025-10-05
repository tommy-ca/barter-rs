# Python Integration Test Specification

**Last Updated:** 2025-10-03 (integration scenarios implemented)

## Goals
- Exercise end-to-end trading flows through the Python bindings without mocks.
- Validate that Python control paths produce identical effects to the canonical Rust APIs.
- Capture regressions in lifecycle management (start, feed, shutdown) and reporting.

## Shared Fixtures
- `barter/examples/config/system_config.json` — canonical multi-exchange configuration.
- `barter/examples/data/binance_spot_market_data_with_disconnect_events.json` — deterministic
  historical stream for backtests.
- Temporary directory per test for generated config copies or exported summaries.
- Runtime initialised via `start_system` helper (no direct Tokio bootstrapping from Python).

## Scenarios

### 1. Live System Lifecycle
- Start system with trading disabled.
- Toggle trading on/off via `SystemHandle.set_trading_enabled` and confirm engine state via
  follow-up events.
- Feed a small sequence of `EngineEvent` values (open requests, cancel commands, shutdown).
- Assert `shutdown_with_summary` returns deterministic metrics (`pnl == 0`, matching intervals).
- Ensure handle reports not running after shutdown and all commands succeed without raising.

### 2. Historic Backtest Summary
- Load config JSON from disk through `SystemConfig.from_json`.
- Run `run_historic_backtest` with shared market data path.
- Verify returned `TradingSummary` exposes:
  - Non-decreasing `time_engine_start` / `time_engine_end`.
  - Instrument tear sheets with zeroed PnL and expected ratio intervals (Daily).
  - Asset tear sheets keyed by `exchange:asset`.
- Round-trip summary via `to_dict()` to guarantee JSON-serialisable payloads.

### 3. Command Builders Round-Trip
- Construct `OrderKey`, `OrderRequestOpen`, and `OrderRequestCancel` objects from Python.
- Use `SystemHandle` helpers to submit open / cancel batches and close positions.
- Confirm no exceptions are raised and events echo expected representations via `repr`.
- Validate `InstrumentFilter` factories (none/exchanges/instruments/underlyings) behave with
  non-empty constraints.

### 4. Failure Surfaces
- Attempt to load malformed JSON config and expect `ValueError` with original message.
- Invoke `run_historic_backtest` with missing file and assert `ValueError` bubble-up.
- Call `shutdown_with_summary` twice to guarantee the second invocation errors with
  "system is not running".

### 5. Backtest Argument Wrappers
- Build `BacktestArgsConstant` from a system config, market data, and seeded balances.
- Create multiple `BacktestArgsDynamic` configurations with unique identifiers and risk-free rates.
- Execute `backtest.backtest` and `backtest.run_backtests` and assert IDs and summaries match expectations.
- Ensure the multi-run summary reports the number of executed backtests and yields trading summaries sorted by ID.

## Tooling & Execution
- Prefer `pytest -m integration` to group the above scenarios; keep fast unit tests separate.
- Leverage `maturin develop` during CI and local runs to build extension in release mode.
- Tag long-running scenarios with `pytest.mark.slow` to allow focused unit test runs.
- Capture logs via `tracing` subscriber configured in `tests_py/conftest.py` (TBD) for easier
  diagnosis.

## Next Steps
- [x] Implement `pytest` markers and fixtures mirroring the above scenarios.
  - Implemented on 2025-10-03 via `tests_py/test_integration_*.py` and `pyproject.toml` marker.
- [x] Add CI job invoking `pytest -m integration --maxfail=1` post wheel build.
  - Added 2025-10-03 as the `python-tests` job in `.github/workflows/ci.yml` running against `barter-python`.
- [x] Evaluate need for synthetic market data fixture with shorter event stream to improve runtime.
  - Added `tests_py/data/synthetic_market_data.json` and updated `example_paths` fixture on 2025-10-03.
