# Python Index Wrappers (2025-10-04)

## Context
- Python bindings currently expose `AssetIndex` but rely on raw integers for
  `ExchangeIndex` and `InstrumentIndex` interactions (order keys, filters,
  account events).
- Adopting explicit wrappers across the FFI boundary keeps the API consistent
  with the Rust types and reduces accidental index mix-ups.

## Goals
- Expose `ExchangeIndex` and `InstrumentIndex` as PyO3 classes, matching the
  ergonomics of the existing `AssetIndex` wrapper.
- Allow constructing `OrderKey` values directly from these wrappers to align
  with typed workflows in Rust.

## Requirements
- Add `PyExchangeIndex` and `PyInstrumentIndex` wrappers with index getters,
  rich string/int representations, and access to the underlying Rust indices.
- Provide a convenience constructor on `OrderKey` to accept the new wrappers
  without breaking the existing integer-based API.
- Ensure the wrappers are exported from the extension module and surfaced via
  the Python package namespace.

## Testing
- Extend the Python binding tests to cover wrapper creation, comparison, and
  integration with `OrderKey.from_indices` using the new wrappers.
