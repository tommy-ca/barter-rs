# Python Indexed Instruments Bindings (2025-10-05)

## Context
- Python consumers currently have to derive exchange, asset, and instrument indices indirectly
  via `SystemConfig` loaders or the execution instrument map helper.
- The underlying `barter_instrument::index::IndexedInstruments` collection powers many engine
  components, but its lookup helpers are not exposed through the bindings.
- Providing a direct binding keeps the Python API aligned with the Rust engine data model and
  reduces duplication across future bridging work.

## Goals
- Expose a `IndexedInstruments` PyO3 wrapper that can be constructed from a `SystemConfig` or
  plain instrument definitions.
- Surface typed lookups for exchange, asset, and instrument indices as well as conversion back to
  human-readable identifiers.
- Allow retrieving the full asset or instrument metadata as Python dictionaries for downstream
  processing.

## Requirements
- Provide `IndexedInstruments.from_system_config(config)` to consume an existing
  `barter_python.SystemConfig`.
- Provide `IndexedInstruments.from_definitions(definitions)` accepting Python dictionaries or
  sequences matching the JSON schema used by `SystemConfig`.
- Support lookups:
  - `exchange_index(exchange_id)` → `ExchangeIndex`
  - `exchange_id(exchange_index)` → `ExchangeId`
  - `asset_index(exchange_id, asset_name_internal)` → `AssetIndex`
  - `asset(asset_index)` → `Asset`
  - `instrument_index_from_exchange_name(exchange_id, instrument_name_exchange)` →
    `InstrumentIndex`
  - `instrument(instrument_index)` → Python dictionary of instrument metadata.
- Implement `__repr__`/`__len__` helpers for quick inspection.

## Testing
- Add pytest coverage constructing the wrapper from raw definitions and from an existing
  `SystemConfig` fixture.
- Assert successful round-trips for each lookup helper and ensure informative `ValueError`
  messages appear when lookups fail.
- Keep tests integration-focused (no mocks) and reuse existing system config data where possible.
