# Portfolio Analytics Helpers (2025-10-04)

## Objective
- Expose core portfolio analytics calculators (Sharpe & Sortino ratios) through the
  Python bindings so strategies can request metrics without running a full backtest.

## Requirements
- Provide module-level functions on the `barter_python` extension to calculate Sharpe and
  Sortino ratios using the existing Rust implementations.
- Accept Python numeric inputs (floats or ints) for returns and convert to `Decimal`.
- Support interval selection via string identifiers (`"Daily"`, `"Annual(252)"`,
  `"Annual(365)"`) and `datetime.timedelta` objects for custom periods.
- Return values using the existing `MetricWithInterval` wrapper for consistent Python API
  ergonomics.
- Bubble up invalid inputs (unknown interval, non-finite numbers) as `ValueError`.

### 2025-10-04 Update — Calmar Ratio
- Surface a `calculate_calmar_ratio` helper alongside Sharpe & Sortino.
- Accept `risk_free_return`, `mean_return`, and `max_drawdown` float inputs, converting to
  `Decimal` via the shared parser.
- Reuse the existing interval parsing flow (strings or `datetime.timedelta`).
- Return a `PyMetricWithInterval` with scaled interval naming consistent with Sharpe/Sortino.
- Preserve the special-case handling from the Rust implementation for zero drawdown values
  (mapping to `Decimal::MAX`, `Decimal::MIN`, or `Decimal::ZERO`).

### 2025-10-04 Update — Profit Factor & Win Rate Helpers
- Expose `calculate_profit_factor` and `calculate_win_rate` on the Python module.
- Accept floating point inputs for profits / losses and wins / total, converting to `Decimal`
  via the shared `parse_decimal` helper.
- Return `None` when the underlying Rust calculator yields `None` (eg. zero trades,
  zero profits and losses) so the Python API mirrors the trading summary semantics.
- Successful calculations should return a `decimal.Decimal` instance, not a plain float,
  to preserve precision alignment with Rust.
- Raise `ValueError` for non-finite inputs and negative totals (mirroring existing
  validation expectations for decimal parsing).

### 2025-10-04 Update — Rate of Return Helper
- Provide `calculate_rate_of_return` with parameters `(mean_return, interval, target_interval=None)`.
- Support the same interval parsing rules as the Sharpe/Sortino helpers for both base and
  optional target intervals (string identifiers or `datetime.timedelta`).
- Return a `PyMetricWithInterval` so the consumer receives the scaled value plus interval label.
- When `target_interval` is supplied, scale the metric using the linear interval scaling from
  the Rust implementation; otherwise return the base interval metric unchanged.
- Reject unsupported interval identifiers or non-finite values with `ValueError`.

## Testing
- Add pytest coverage asserting normal and edge-case calculations (zero volatility, negative
  excess returns) and interval scaling.
- Exercise string- and timedelta-based interval parsing paths.
- Ensure error handling for unsupported interval inputs is validated via pytest.
- Add edge-case coverage for zero drawdown scenarios to confirm the Python helper exposes the
  same semantics as the Rust API (positive, negative, and neutral excess returns).
- Cover profit factor behaviour for neutral, perfect, worst, and typical cases (including
  `None` returns) alongside validation for non-finite inputs.
- Cover win rate behaviour for empty trade sets and typical ratios.
- Cover rate-of-return scaling from daily to annual and custom `timedelta` intervals, including
  handling of optional `target_interval` omissions.
