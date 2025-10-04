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

## Testing
- Add pytest coverage asserting normal and edge-case calculations (zero volatility, negative
  excess returns) and interval scaling.
- Exercise string- and timedelta-based interval parsing paths.
- Ensure error handling for unsupported interval inputs is validated via pytest.
- Add edge-case coverage for zero drawdown scenarios to confirm the Python helper exposes the
  same semantics as the Rust API (positive, negative, and neutral excess returns).
