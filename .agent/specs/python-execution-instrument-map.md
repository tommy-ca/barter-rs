# Python Execution Instrument Map Bindings (2025-10-05)

## Objective
- Expose the `barter_execution::map::ExecutionInstrumentMap` type and helpers to Python
  consumers through the `barter-python` bindings.
- Provide ergonomic helpers for translating between exchange identifiers, asset names,
  instrument names, and their corresponding indices.

## Requirements
- Add a `PyExecutionInstrumentMap` wrapper that owns an `ExecutionInstrumentMap` instance.
- Provide a Python-accessible constructor that accepts an `ExchangeId` and a sequence of
  instrument definitions (mirroring `SystemConfig` instruments) and internally builds the
  indexed representation.
- Surface lookup helpers mirroring the Rust API:
  - `find_exchange_id`, `find_exchange_index`
  - `find_asset_name_exchange`, `find_asset_index`
  - `find_instrument_name_exchange`, `find_instrument_index`
  - Iteration helpers exposing the available exchange asset and instrument names.
- Ensure lookups raise `ValueError` with informative messages when entries are missing.
- Export the wrapper from `barter_python` and provide a thin Python convenience class that
  delegates to the Rust bindings.

## Testing
- Add pytest coverage that builds a small instrument set, constructs the execution map,
  and validates round-trip conversions for assets and instruments.
- Exercise error paths (e.g., missing asset/instrument lookups) to confirm informative
  exceptions.
- Extend Rust unit coverage if necessary for newly introduced helper functions.

## Notes
- Reuse existing instrument parsing helpers (`PyInstrumentSpec`, `InstrumentConfig`) to
  avoid duplicating validation logic.
- Ensure bindings integrate cleanly with existing system configuration flows so Python
  users can reuse definitions when composing execution clients.
