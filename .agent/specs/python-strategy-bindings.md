# Python Strategy Binding Enhancements (2025-10-04)

## Context
- `barter_python.strategy` currently re-implements close-position helpers in pure Python.
- The Rust crate already provides `build_ioc_market_order_to_close_position` and
  `close_open_positions_with_market_orders` within `barter::strategy::close_positions`.
- Maintaining duplicate logic risks drift and makes it harder to benefit from future Rust fixes.

## Goals
- Expose Rust close-position helpers through PyO3 so Python callers reuse the canonical
  implementations for constructing market-close requests.
- Preserve the existing Python API surface (`build_ioc_market_order_to_close_position` and
  `close_open_positions_with_market_orders`) while delegating the order construction to Rust.
- Accept simple Python-native inputs (ints, floats/decimals, enums, callables) to keep the Python
  ergonomics unchanged.

## Requirements
- Add a new `strategy` module in the PyO3 crate registering
  `build_ioc_market_order_to_close_position(exchange, instrument, side, quantity, strategy_id,
  price, client_order_id=None)` returning a `PyOrderRequestOpen`.
- In the Python wrapper, reuse the Rust binding for individual order construction while keeping
  the higher-level iteration and generator handling in pure Python.
  - Convert the binding result back into the existing Python dataclasses (`OrderKey`,
    `RequestOpen`, `OrderRequestOpen`) so downstream code observes the same interface.
  - Validate that quantities are positive and prices are supplied before invoking the binding,
    raising `ValueError` with informative messages.
  - Support custom client order IDs by allowing callables or explicit values and forwarding the
    resolved identifier into the binding.
- Re-export the binding-backed helper from the Python packaging layer, keeping
  backwards-compatible names.

## Testing
- Extend `tests_py/test_strategy.py` to assert that the Python helpers still behave identically
  while flowing through the new bindings.
- Add regression coverage for negative quantities and missing prices raising `ValueError`.

## Follow-Ups
- Explore exposing additional `barter::strategy` utilities (eg/ disconnect / trading-disabled
  helpers) once the close-position path is stable.
- Prefer the Rust-backed `barter_python.InstrumentFilter` class across Python modules rather than
  the legacy Protocol placeholder so type hints and runtime behaviour stay aligned with the
  bindings (2025-10-05).
