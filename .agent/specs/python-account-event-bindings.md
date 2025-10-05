# Python Account Event Bindings

Last updated: 2025-10-05

## Context
- `barter_python.execution` currently implements `AccountEvent` and
  `AccountEventKind` as pure Python dataclasses.
- The Rust crates already expose rich representations backed by
  `barter-execution::{AccountEvent, AccountEventKind}` along with
  typed snapshots, orders, and trades.
- Downstream consumers (eg. `MockExecutionClient`, integration tests)
  require parity with the Rust types to avoid drift and to take
  advantage of indexer utilities.

## Goals
- Provide PyO3 wrappers for `AccountEvent` and `AccountEventKind` that
  mirror the Rust semantics while keeping the Python ergonomics.
- Replace the pure Python dataclasses with the new bindings and ensure
  backwards-compatible constructors and accessors.
- Allow account events emitted by the mock execution client to return
  the typed bindings rather than ad-hoc dictionaries.

## Requirements
- Implement `PyAccountEventKind` with variants `snapshot`,
  `balance_snapshot`, `order_snapshot`, `order_cancelled`, and `trade`.
  - Provide `variant` (string) and `value(py)` accessors.
  - Implement `__repr__`, `__str__`, equality, and hashing consistent
    with the underlying Rust enum.
- Implement `PyAccountEvent` with `new(exchange, kind)`,
  `exchange` property (returning `int` index), `kind` accessor, and
  `to_json` / `from_json` helpers for serialisation.
- Update module registration in `lib.rs` to export the new classes and
  update the Python package to re-export them.
- Update `PyMockExecutionClient.poll_event` to return
  `PyAccountEvent` instances indexed via the execution instrument map.

## Testing Strategy
- Rust unit tests validating conversion helpers and JSON round-trips
  for both `PyAccountEventKind` variants and `PyAccountEvent`.
- Pytest coverage asserting the new bindings behave identically to the
  previous Python dataclasses (construction, equality, repr, hashing,
  variant matching) and that the mock execution client now returns
  typed events.
- Run `cargo test -p barter-python` and `uv run pytest tests_py` after
  implementation.

