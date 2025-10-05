# Python Engine Output Bindings (2025-10-05)

## Overview
- Provide typed Python wrappers for `barter::engine::EngineOutput` variants
  emitted via audit processing so callers can inspect outputs without falling
  back to generic dictionaries.
- Preserve ergonomic access to existing `ActionOutput` wrappers while adding
  coverage for trading disabled notices, disconnect signals, position exits,
  and generated algo orders.

## Requirements
- Introduce a `PyEngineOutput` enum-like wrapper with variants for
  `Commanded`, `OnTradingDisabled`, `AccountDisconnect`, `MarketDisconnect`,
  `PositionExit`, and `AlgoOrders`.
- Reuse existing `PyActionOutput` wrapper when the variant is `Commanded`.
- Expose structured helpers for each variant:
  - `trading_disabled` returns the Python representation of the
    `OnTradingDisabled` payload or `None`.
  - `account_disconnect` / `market_disconnect` mirror `OnDisconnect` payloads.
  - `position_exit` returns a typed wrapper exposing instrument key and quote
    summary fields.
  - `algo_orders` surfaces the generated algo orders payload via a typed
    helper (for now backed by JSON dictionaries until further binding work).
- Implement `PyPositionExit` wrapper covering quantity, instrument, and realised
  PnL attributes with `__repr__` support.
- Update audit event conversion so `NoneOneOrMany` collections contain
  `PyEngineOutput` objects instead of raw dictionaries when possible.
- Re-export new wrappers from the Python package for direct consumption.

## Python API
- `PyEngineOutput`
  - `variant` property returns canonical variant string.
  - Variant accessors (`commanded`, `trading_disabled`, `account_disconnect`,
    `market_disconnect`, `position_exit`, `algo_orders`) expose typed wrappers
    or `None` when mismatched.
  - `other` accessor preserves JSON fallback when variant payloads are not yet
    bound.
  - `__repr__` highlights the variant and whether typed payload is available.
- `PyPositionExit`
  - Attributes: `instrument`, `quantity`, `realised_quote`, `fees`.
  - `to_dict()` helper for ergonomic inspection in Python tests.
  - `__repr__` summarises key fields for debugging.

## Testing Strategy
- Add Rust unit tests verifying conversions from each `EngineOutput` variant to
  the PyO3 wrappers, including payload accessors and fallbacks.
- Extend Python integration tests (`tests_py/test_integration_live.py` and
  related suites) to assert audit outputs now yield `EngineOutput` wrappers.
- Ensure existing tests covering `ActionOutput` continue to pass and recognise
  the new wrapper types.

## Notes
- Future work: replace JSON fallbacks for algo order payloads with typed
  wrappers once the underlying structs are stabilised for Python usage.
