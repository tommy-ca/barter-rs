# Python Collection Bindings (2025-10-05)

## Overview
- Expose `barter_integration::collection::none_one_or_many::NoneOneOrMany` and
  `barter_integration::collection::one_or_many::OneOrMany` through the
  `barter-python` PyO3 bindings.
- Provide ergonomic Python wrappers for representing zero, one, or many values
  produced during engine audits and integration workflows.
- Ensure Python bindings retain iteration, length, and inspection semantics so
  downstream consumers can interact with audit outputs without bespoke
  translation code.

## Requirements
- Implement `PyNoneOneOrMany` and `PyOneOrMany` classes that mirror the Rust
  enums while following Pythonic conventions (`len()`, iteration, equality,
  `repr`).
- Support construction from optional values, sequences, or individual payloads
  exposed via PyO3 conversions.
- Provide conversion helpers used inside the audit bindings so that audit
  snapshots, command results, and risk outputs can surface the new wrappers.
- Re-export the bindings from the Python package to make them available under
  `barter_python`.

## Testing Strategy
- Extend Rust unit tests within `barter-python` covering round-trip conversions
  between the wrappers and their underlying Rust enums.
- Add Python end-to-end coverage in `tests_py` asserting:
  - Construction from empty, single, and multi-item collections.
  - Iteration and length semantics.
  - Equality and representation expectations for debugging purposes.
- Verify integration with existing audit helpers by exercising a minimal engine
  audit flow that returns `NoneOneOrMany` results and ensuring they materialise
  as the new Python wrappers.

