# Python Instrument Specification Bindings

- **Date:** 2025-10-05
- **Author:** assistant

## Context

Python consumers previously relied on loose dictionaries when configuring
instrument specifications for `SystemConfig`. This made it easy to introduce
typos and removed alignment with the strongly-typed Rust
`InstrumentSpec` structures.

## Goals

- Surface thin PyO3 wrappers for the Rust `InstrumentSpec` family so Python
  code can construct and inspect specs without manual JSON shaping.
- Provide constructors for the price, quantity, and notional components while
  reusing existing `Asset` wrappers for asset-based quantity units.
- Ensure the new bindings participate in tests and module exports for both the
  Rust extension and the pure Python namespace re-export.

## Requirements

- Add PyO3 wrappers:
  - `OrderQuantityUnits` limited to existing variants (`asset`, `contract`,
    `quote`).
  - `InstrumentSpecPrice`, `InstrumentSpecQuantity`, and
    `InstrumentSpecNotional` with validation for non-negative / positive values
    as appropriate.
  - `InstrumentSpec` wrapper composing the above components.
- Update the module init to register the new classes so
  `import barter_python as bp` exposes them.
- Extend pytest coverage verifying round-trip behaviour, including unit access
  and decimal preservation.
- Document availability in the pure Python `Instrument` helper to steer users
  towards the bindings.

## Testing

- Rust: `cargo test -p barter-python` (covers module registration).
- Python: `pytest -q tests_py/test_instrument.py` including new spec binding
  tests.

## Follow-ups

- Consider providing convenience helpers on `PyInstrumentSpec` to translate to
  dict/JSON for configuration authoring.
- Explore exposing builders that integrate directly with `SystemConfig` once
  the Python port of the engine accepts structured specs.
