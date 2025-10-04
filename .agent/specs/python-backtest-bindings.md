# Python Backtest Bindings Specification

**Last Updated:** 2025-10-04

## Objective
- Bridge the Rust `barter::backtest` module into the `barter-python` package via PyO3 bindings.
- Replace the incomplete pure Python backtest implementation with direct Rust-backed execution.
- Ensure trading summaries returned from backtests reuse existing `PyBacktestSummary` and `PyMultiBacktestSummary` wrappers.

## Scope
- Expose `BacktestArgsConstant` and `BacktestArgsDynamic` constructors for default engine types.
- Provide synchronous Python-callable wrappers for `backtest` and `run_backtests` using Tokio runtimes under the hood.
- Support market data sources backed by the existing `MarketDataInMemory` binding.
- Return rich Python trading summaries with decimal fidelity and dictionary conversion helpers.
- Validate inputs (risk-free return, summary interval strings, strategy identifiers) before calling Rust functions.

## Out of Scope
- Custom strategy or risk manager injection beyond the default engine implementations.
- Alternate market data sources beyond the existing in-memory helper.
- Async Python APIs for streaming backtest progress.

## Requirements
1. Add PyO3 bindings in `barter-python/src/backtest.rs` for:
   - `BacktestArgsConstant::new` accepting instruments, executions, market data, summary interval, and engine state.
   - `BacktestArgsDynamic::new` accepting id, risk-free return, strategy handles, and risk manager handles.
   - `backtest` returning a `PyBacktestSummary`.
   - `run_backtests` returning a `PyMultiBacktestSummary`.
2. Extend bindings to construct default engine state from Python-provided JSON (reuse `SystemConfig` parsing where possible).
3. Update the Python `backtest` module to call into the new bindings while preserving high-level ergonomics.
4. Cover new behaviour with Rust unit tests (where feasible) and Python integration tests exercising single and multi backtests.
5. Document usage in `barter-python/README.md` and update `.agent/todo.md` if follow-ups emerge.

## Testing Strategy
- Add Pytest coverage verifying `run_backtests` aggregates multiple strategies and returns consistent summaries.
- Ensure numeric fields (duration, risk-free return) round-trip accurately through the bindings.
- Validate error surfaces for invalid market data or mismatched instruments.
- Run `cargo test -p barter-python` and `pytest -q tests_py` after implementation.

