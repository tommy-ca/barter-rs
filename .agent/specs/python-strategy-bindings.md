# Python Strategy Binding Enhancements (2025-10-04)

## Context
- `barter_python.strategy` currently re-implements close-position helpers in pure Python.
- The Rust crate already provides `build_ioc_market_order_to_close_position` and
  `close_open_positions_with_market_orders` within `barter::strategy::close_positions`.
- Maintaining duplicate logic risks drift and makes it harder to benefit from future Rust fixes.

## Goals
- Expose Rust close-position helpers through PyO3 so Python callers reuse the canonical
  implementations.
- Preserve the existing Python API surface (`build_ioc_market_order_to_close_position` and
  `close_open_positions_with_market_orders`) while delegating work to Rust bindings.
- Accept simple Python-native inputs (ints, floats/decimals, enums, callables) to keep the Python
  ergonomics unchanged.

## Requirements
- Add a new `strategy` module in the PyO3 crate registering:
  - `build_ioc_market_order_to_close_position(exchange, position, strategy_id, price, gen_cid)`
    returning a `PyOrderRequestOpen`.
  - `close_open_positions_with_market_orders(strategy_id, instruments, gen_cid)` that produces
    iterables of cancel/open order requests.
- Provide lightweight conversion structs so Python code can pass position and instrument snapshots
  without depending on internal `EngineState` types.
  - Validate that quantities are non-negative and that required prices are supplied when
    generating market orders.
  - Support custom client order ID generation by accepting an optional Python callable.
- Re-export the functions from the Python packaging layer, keeping backwards-compatible names.

## Testing
- Extend `tests_py/test_strategy.py` to assert that the Python helpers still behave identically
  while flowing through the new bindings.
- Add regression coverage for negative quantities and missing prices raising `ValueError`.

## Follow-Ups
- Explore exposing additional `barter::strategy` utilities (eg/ disconnect / trading-disabled
  helpers) once the close-position path is stable.

