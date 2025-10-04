# Pure Python Port of Barter Statistic Module (2025-10-04)

## Objective
- Port the Rust `barter::statistic` module to pure Python, maintaining feature parity and API compatibility.
- Provide financial metrics calculations (Sharpe, Sortino, Calmar ratios, drawdown metrics, profit factor, win rate, rate of return) with proper time interval handling.
- Follow TDD principles with 80% focus on implementation, 20% on testing.

## Requirements
- Implement TimeInterval protocol and concrete classes (Annual365, Annual252, Daily, TimeDeltaInterval).
- Port all metric classes with their calculate and scale methods where applicable.
- Use decimal.Decimal for financial precision instead of floats.
- Maintain the same API structure and behavior as Rust implementations.
- Handle edge cases (zero standard deviation, zero drawdown, etc.) consistently.

## Components to Port

### Time Intervals
- `TimeInterval` protocol with `name()` and `interval()` methods.
- `Annual365`: 365-day annual interval.
- `Annual252`: 252-day annual interval (trading days).
- `Daily`: 1-day interval.
- `TimeDeltaInterval`: Custom timedelta-based interval.

### Metrics
- `SharpeRatio`: Risk-adjusted return metric with calculate() and scale() methods.
- `SortinoRatio`: Downside risk-adjusted return metric.
- `CalmarRatio`: Risk-adjusted return using maximum drawdown.
- `MaxDrawdown`: Maximum peak-to-trough decline.
- `MeanDrawdown`: Average drawdown magnitude.
- `ProfitFactor`: Gross profit divided by gross loss.
- `WinRate`: Percentage of winning trades.
- `RateOfReturn`: Return rate with interval scaling.

## Implementation Notes
- Use `dataclasses` or similar for simple data structures.
- Implement proper `__eq__`, `__hash__`, and comparison methods.
- Use `decimal.Decimal` for all financial calculations.
- Handle division by zero and invalid inputs gracefully.
- Maintain the same scaling logic for time intervals (square root of time ratio).

## Testing
- Port all Rust unit tests to Python equivalents.
- Add edge case coverage for zero values, negative returns, etc.
- Test interval scaling between different time periods.
- Ensure decimal precision is maintained throughout calculations.

## Dependencies
- `decimal` for high-precision decimal arithmetic.
- `datetime` for timedelta handling.