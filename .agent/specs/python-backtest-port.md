# Pure Python Port of Barter Backtest Module (2025-10-04)

## Objective
- Port the Rust `barter::backtest` module to pure Python, maintaining feature parity and API compatibility.
- Provide historical simulation capabilities for trading strategies using market data.
- Follow TDD principles with 80% focus on implementation, 20% on testing.

## Requirements
- Implement BacktestMarketData protocol for different market data sources.
- Port BacktestSummary and MultiBacktestSummary data structures.
- Implement run_backtests and backtest functions for concurrent simulations.
- Use existing pure Python components (statistic, strategy, data, etc.) for consistency.
- Handle async operations using asyncio for concurrent backtest execution.

## Components to Port

### BacktestMarketData Protocol
- `BacktestMarketData` protocol with `stream()` and `time_first_event()` methods.
- Support for different market data formats (JSON streams, CSV, etc.).
- Async iteration over market events.

### Summary Structures
- `BacktestSummary`: Contains backtest ID, risk-free return, and trading summary.
- `MultiBacktestSummary`: Aggregates results from multiple concurrent backtests with total duration.

### Backtest Runner
- `run_backtests()`: Run multiple backtests concurrently with shared constants and varying parameters.
- `backtest()`: Run single backtest simulation.
- Support for different strategies, risk managers, and market data sources.

## Implementation Notes
- Use dataclasses for simple data structures.
- Implement proper async/await for concurrent execution.
- Integrate with existing pure Python engine simulation.
- Maintain compatibility with existing trading summary generation.

## Testing
- Port all Rust unit tests to Python equivalents.
- Add integration tests for full backtest workflows.
- Test concurrent execution and result aggregation.
- Ensure compatibility with existing pure Python components.

## Dependencies
- `asyncio` for concurrent execution.
- `datetime` and `decimal` for time and financial calculations.
- Existing barter_python modules (statistic, strategy, data, execution).</content>
</xai:function_call: write>
<parameter name="filePath">.agent/specs/python-backtest-port.md