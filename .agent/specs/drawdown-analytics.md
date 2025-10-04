# Drawdown Analytics Helpers (2025-10-04)

## Objective
- Expose drawdown calculators through the Python bindings so strategists can
  obtain downside risk metrics without running a full trading summary.

## Scope
- Accept a monotonic sequence of `(datetime, numeric_value)` points describing a
  portfolio or equity curve.
- Provide the following helpers on the `barter_python` module:
  - `generate_drawdown_series(points)` → `list[Drawdown]`
  - `calculate_max_drawdown(points)` → `Drawdown | None`
  - `calculate_mean_drawdown(points)` → `MeanDrawdown | None`
- Return existing wrapper classes (`Drawdown`, `MeanDrawdown`) so the API stays
  consistent with trading summaries.

## Requirements
- Accept `datetime.datetime` instances (timezone-aware or naive). Naive values
  are assumed to be UTC.
- Accept floats, ints, or `decimal.Decimal` values for the equity points. Values
  must be finite; raise `ValueError` otherwise.
- Reject items that are not sized iterables of length two.
- Allow empty or single-point sequences and return empty/`None` results without
  raising.
- Preserve ordering; no additional sorting is performed by the helper.
- Surface input validation errors with descriptive `ValueError` messages that
  mention the offending index.

## Implementation Notes
- Reuse `barter::statistic::metric::drawdown::DrawdownGenerator` to build the
  drawdown series.
- Feed generated drawdowns into `MaxDrawdownGenerator` and
  `MeanDrawdownGenerator` for their respective helpers.
- Reuse existing conversion helpers (`parse_decimal`, `PyDrawdown::from_drawdown`,
  `PyMeanDrawdown::from_mean`).
- Add a private parsing utility in `analytics.rs` to transform Python inputs
  into `Vec<Timed<Decimal>>`.

## Testing
- Extend `tests_py/test_portfolio_metrics.py` with scenarios covering:
  - Typical curve producing multiple drawdowns (verify series ordering).
  - Curve ending mid-drawdown (ensure trailing drawdown is emitted).
  - Edge cases with empty / single-point inputs returning empty/`None`.
  - Validation paths for bad tuple shapes and non-finite values.
- Integration expectation: helpers work after `maturin develop` build and are
  exercised by CI via the existing portfolio metrics test module.

