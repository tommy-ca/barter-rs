# Python Execution Order Enum Bindings (2025-10-05)

## Context
- Python users currently rely on pure Python `Enum` definitions for
  `OrderKind` and `TimeInForce` in `barter_python.execution`.
- The Rust `barter-execution` crate already provides strongly typed
  implementations that power the trading engine, and the bindings crate
  exposes neighbouring primitives such as `OrderRequestOpen` and
  `OrderKey`.
- Bridging these enums keeps the Python API aligned with the Rust
  semantics (including `GoodUntilCancelled` post-only flags) and removes
  drift between the two implementations.

## Goals
- Expose PyO3-backed wrappers for `OrderKind` and `TimeInForce` tailored
  for Python ergonomics.
- Preserve a simple `.value` representation for backwards compatibility
  while surfacing additional metadata (e.g. `post_only`).
- Ensure the wrappers integrate seamlessly with existing order request
  helpers and engine bindings.

## Requirements
- Implement `PyOrderKind` with class constructors for `market()` and
  `limit()`, equality, hashing, and string representations mirroring the
  Rust enum.
- Implement `PyTimeInForce` with helpers for
  `good_until_cancelled(post_only: bool = False)`, `good_until_end_of_day()`,
  `fill_or_kill()`, and `immediate_or_cancel()`.
- Each wrapper should expose a `.value` property matching the lowercase
  snake-case representation used today.
- `PyTimeInForce` must expose a `.post_only` boolean property that is `True`
  when the Rust variant is `GoodUntilCancelled { post_only: true }` and
  `False` otherwise.
- Update `python/barter_python/execution.py` to re-export the new wrappers
  (removing the pure Python enums) and adjust downstream helpers if
  required.
- Export the wrappers via the extension module (`lib.rs`).

## Testing
- Extend `tests_py/test_execution.py` to construct the new wrappers,
  asserting `.value`, `.post_only`, equality, hashing, and string / repr
  semantics.
- Add Rust unit coverage ensuring round-trips from Python create the
  expected Rust enum variants.
- Run `cargo test -p barter-python` and `pytest -q tests_py` to validate
  the change end-to-end.
