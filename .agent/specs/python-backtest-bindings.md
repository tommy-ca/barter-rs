# Python Backtest Bindings Specification

**Last Updated:** 2025-10-05 (Milestone 2 complete)

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

## Milestones

### Milestone 1 — Argument Wrappers (2025-10-05)
- Introduce `PyBacktestArgsConstant` and `PyBacktestArgsDynamic` wrappers that expose strongly typed accessors for instruments, executions, market data, and risk-free rate inputs.
- Allow constructing constant arguments from `PySystemConfig`, `PyMarketDataInMemory`, and a provided summary interval type without invoking the engine yet.
- Ensure dynamic arguments accept minimal metadata (id, risk-free return) while deferring strategy and risk manager configuration for later milestones.
- Add Rust unit coverage confirming argument wrappers serialize into the underlying `BacktestArgsConstant`/`BacktestArgsDynamic` structures with expected indices and validation errors.
- Update the Python `backtest` module to surface the new wrappers for downstream consumers.

### Milestone 2 — Execution Pipeline (2025-10-05)
- Wire the wrappers into synchronous helpers that execute `backtest`/`run_backtests` behind the scenes while releasing the GIL. ✅
- Extend pytest coverage to exercise single and multi-run flows using canned market data fixtures. ✅
- Document usage patterns in the README and integration guides. (Follow-up: refresh README examples to mention new helpers.)

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
4. Cover new behaviour with Rust smoke tests (where feasible) and Python integration tests exercising single and multi backtests as each milestone lands.
5. Document usage in `barter-python/README.md` and update `.agent/todo.md` if follow-ups emerge.

## Testing Strategy
- Add Pytest coverage verifying `run_backtests` aggregates multiple backtests and returns consistent summaries.
- Ensure numeric fields (duration, risk-free return) round-trip accurately through the bindings.
- Validate error surfaces for invalid market data or mismatched instruments.
- Run `cargo test -p barter-python` and `pytest -q tests_py` after implementation.
