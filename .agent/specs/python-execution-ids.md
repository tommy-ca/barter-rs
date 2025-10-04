# Python Execution Identity Bindings (2025-10-04)

## Context
- The Python package still implements the execution identity types (`ClientOrderId`,
  `OrderId`, `StrategyId`, `OrderKey`) purely in Python.
- The Rust crate `barter-execution` already defines these types with richer
  validation and helper constructors, and the bindings crate depends on it.
- Bridging through PyO3 will remove duplication, keep behaviour consistent with
  Rust, and let downstream crates share these types across bindings.

## Goals
- Expose PyO3 wrappers for the execution identity types while keeping an
  idiomatic Python API (constructors, equality, hashing, repr/str).
- Allow constructing and deconstructing `OrderKey` values with the new wrappers
  and exchange/instrument indices from the bindings crate.
- Ensure the wrappers interoperate with existing account and order event
  bindings, enabling future Rust-backed order management in Python.

## Requirements
- Add `PyClientOrderId`, `PyOrderId`, and `PyStrategyId` classes in the Rust
  bindings with `new` constructors, `unknown` for strategies, string accessors,
  and rich comparisons / hashing that mirror the Rust semantics.
- Provide a `PyOrderKey` wrapper with tuple-based equality & hashing and helper
  constructors that accept exchange & instrument indices (both integers and the
  typed wrappers already exposed in the bindings).
- Update the Python package to re-export the new bindings instead of the pure
  Python dataclasses.
- Preserve backwards compatibility with existing call sites by keeping module
  level aliases (e.g. `ClientOrderId = PyClientOrderId`) until the pure Python
  implementations are fully removed.

## Testing
- Extend the Python binding tests to cover constructing and comparing the new
  wrappers, including hashing in dictionaries and round-tripping via `OrderKey`.
- Add Rust unit tests for `PyOrderKey` to confirm conversions with the typed
  indices and `barter_execution` identities.
- Run the full pytest suite to ensure downstream modules continue to use the
  new bindings without regressions.
