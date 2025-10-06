# Python Trading Summary Generator Bindings (2025-10-06)

## Summary
- Expose `TradingSummaryGenerator` from `barter::statistic::summary` to Python callers.
- Allow Python code to incrementally update summaries from new balance snapshots and
  position exits without re-running full backtests.
- Provide ergonomic interval handling in line with existing summary helpers
  (`daily`, `annual252`, `annual365`).

## Requirements
- Introduce a `PyTradingSummaryGenerator` wrapper owning a `TradingSummaryGenerator`.
  - Disallow direct construction from Python; instances are created by helpers that
    already possess engine state (e.g. shutdown routines, backtests).
  - Surface read-only properties for `risk_free_return`, `time_engine_start`, and
    current `time_engine_now`/duration.
- Add update helpers:
  - `update_time_now(time: datetime)` to advance the generator clock.
  - `update_from_balance(balance: AssetBalance)` accepting `PyExecutionAssetBalance`.
  - `update_from_position(position: PositionExit)` accepting `PyPositionExit`.
- Provide `generate(interval: str | None = None)` returning `PyTradingSummary` with the
  same interval parsing as other summary helpers (default `daily`).
- Extend system/backtest helpers to return the generator alongside the summary:
  - `SystemHandle.shutdown_with_summary_generator(...) -> (TradingSummary, TradingSummaryGenerator)`.
  - `run_historic_backtest_with_generator(...) -> (TradingSummary, TradingSummaryGenerator)`.
- Ensure the new class is re-exported from the Python package (`barter_python.__all__`).

## Testing Strategy
- Rust unit tests confirming generator updates from `PyExecutionAssetBalance` and
  `PyPositionExit` correctly mutate `TradingSummaryGenerator` internals.
- Python integration test verifying:
  - `run_historic_backtest_with_generator` returns summary identical to the existing
    helper for the same inputs.
  - Updating the generator with a synthetic balance snapshot advances
    `time_engine_end` and affects `assets` as expected.
  - `generate("annual252")` and `generate("annual365")` honour interval labels.
- Re-run `cargo test -p barter-python` and `uv run pytest tests_py/test_summary_generator.py`
  to validate both sides of the FFI.

## Notes
- Constructing `PositionExited` instances requires rebuilding `AssetFees` objects; use
  `AssetFees::quote_fees` for quote-denominated fees captured in Python wrappers.
- Ensure snapshot updates borrow balances immutably (`Snapshot::new(&balance.inner)`) to
  avoid unnecessary cloning.
- Document the new workflow in `barter-python/README.md` after bindings land.
