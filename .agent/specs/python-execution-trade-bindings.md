# Python Execution Trade Bindings (2025-10-04)

## Context
- The Python package still ships pure Python implementations of trade-related
  types from `barter-execution`, including `TradeId`, `AssetFees`, and `Trade`.
- The Rust bindings crate already depends on `barter-execution` and exposes
  order identity wrappers via PyO3, making the trade duplication unnecessary.
- Providing Rust-backed wrappers ensures parity with the core engine, enables
  other bindings to share the same types, and removes subtle behaviour drift
  between Rust and Python.

## Goals
- Expose PyO3 wrappers for `TradeId`, `AssetFees`, and `Trade` that mirror the
  semantics of their Rust counterparts while feeling idiomatic in Python. The
  initial increment will focus on quote-denominated fees (`QuoteAsset`) needed
  for trade events and summaries.
- Update the Python module to re-export the new bindings instead of the pure
  Python dataclasses, keeping the public API stable for downstream users.
- Extend exploratory documentation to note the availability of the new
  bindings for composing account events and summaries.

## Requirements
- Implement `PyTradeId`, `PyQuoteAsset`, `PyAssetFees`, and `PyTrade` wrappers
  with constructors, rich comparisons, hashing, readable `__str__/__repr__`,
  and helper methods (`quote_fees`, `value_quote`).
- Accept native Python numeric types (`float`, `Decimal`, `int`) for monetary
  values, validating inputs and surfacing clear error messages on invalid
  values. String parsing is optional but preferred when provided.
- Ensure the wrappers integrate with existing bindings, allowing usage inside
  `PyEngineEvent` helpers and account event processing flows.
- Re-export the wrappers via `python/barter_python/execution.py` and update the
  public package namespace to point at the PyO3 types.

## Testing
- Expand `tests_py/test_execution.py` to construct the new wrappers, covering
  equality, hashing, and helper methods, and ensuring interoperability with the
  existing account event helpers.
- Run the full Rust and Python test suites (`cargo test -p barter-python`,
  `pytest -q tests_py`) to validate the new bindings end-to-end.
