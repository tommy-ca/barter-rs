# Python Backtest Bindings Specification

**Last Updated:** 2025-10-04

## Objective
- Bridge the Rust `barter::backtest` module into the `barter-python` package via PyO3 bindings.
- Replace the incomplete pure Python backtest implementation with direct Rust-backed execution.
- Ensure trading summaries returned from backtests reuse existing `PyBacktestSummary` and `PyMultiBacktestSummary` wrappers.

## Scope
- Provide synchronous Python-callable wrappers for `backtest` and `run_backtests` using Tokio runtimes under the hood.
- Build the necessary `BacktestArgsConstant` values directly from a `SystemConfig`, seeded balances, and `MarketDataInMemory` supplied from Python.
- Generate `BacktestArgsDynamic` values internally using the default strategy and risk manager; expose a lightweight Python struct to let callers supply unique identifiers and risk-free rates.
- Return rich Python trading summaries with decimal fidelity and dictionary conversion helpers.
- Validate inputs (risk-free return, summary interval strings, requested backtest ids) before calling Rust functions.

## Out of Scope
- Custom strategy or risk manager injection beyond the default engine implementations.
- Alternate market data sources beyond the existing in-memory helper.
- Async Python APIs for streaming backtest progress.

## Requirements
1. Add PyO3 bindings in `barter-python/src/backtest.rs` for:
   - Building default `BacktestArgsConstant` instances from `PySystemConfig`, `PyMarketDataInMemory`, seeded balances, and a requested summary interval string.
   - Constructing lightweight `PyBacktestArgsDynamic` wrappers that capture an id and risk-free return for default strategy/risk setups.
   - Executing `backtest` and `run_backtests` (multi run) and surfacing results as `PyBacktestSummary` / `PyMultiBacktestSummary`.
2. Ensure bindings spin up Tokio runtimes and block on the async Rust functions while releasing the GIL.
3. Update the Python `backtest` module to delegate to the new bindings while preserving a friendly API surface.
4. Cover new behaviour with Rust smoke tests (where feasible) and Python integration tests exercising single and multi backtests.
5. Document usage in `barter-python/README.md` and update `.agent/todo.md` if follow-ups emerge.

## Testing Strategy
- Add Pytest coverage verifying `run_backtests` aggregates multiple backtests and returns consistent summaries.
- Ensure numeric fields (duration, risk-free return) round-trip accurately through the bindings.
- Validate error surfaces for invalid market data or mismatched instruments.
- Run `cargo test -p barter-python` and `pytest -q tests_py` after implementation.
