# Python Account Snapshot Bindings (2025-10-04)

## Context
- Python bindings currently expose granular account events (balance snapshots, order snapshots,
  cancellations, trades) but lack coverage for the full `AccountSnapshot` variant emitted by
  `barter_execution`.
- Without access to the snapshot event, Python integrations cannot synchronise complete exchange
  state during initial handshakes or reconnect scenarios.

## Goals
- Surface `AccountSnapshot` and `InstrumentAccountSnapshot` structures through PyO3 bindings.
- Allow Python callers to construct snapshot payloads for testing and to consume snapshots emitted
  from Rust when running systems via Python.
- Expand `EngineEvent` helpers with an `account_snapshot` constructor that accepts the new wrapper
  type.

## Requirements
- Implement `PyInstrumentAccountSnapshot` storing the instrument index and list of order snapshots.
  - Provide constructor validation ensuring instrument indices are non-negative.
  - Expose `.instrument` and `.orders` accessors returning Python-native types.
- Implement `PyAccountSnapshot` storing exchange index, balances, and instrument snapshots.
  - Balances are supplied as `(asset_index, total, free, time_exchange)` tuples using decimal
    parsing consistent with existing helpers.
  - Enforce that `free <= total` for each balance entry.
  - Provide properties for `.exchange`, `.balances`, `.instruments`, and `.time_most_recent()`.
- Extend `PyEngineEvent` with a `@staticmethod account_snapshot(snapshot)` constructor returning an
  account `EngineEvent` carrying the snapshot.
- Register the new classes in `lib.rs` and update Python `__init__` exports.

## Testing
- Add pytest coverage validating:
  - Constructing snapshots with balances and order snapshots yields the expected event JSON.
  - Validation errors surface for negative balances or mismatched exchange indices.
  - `time_most_recent()` matches the latest timestamp across balances and orders.

## Follow-Ups
- Consider convenience constructors for building snapshots from Python dictionaries once basic
  support is stable.
