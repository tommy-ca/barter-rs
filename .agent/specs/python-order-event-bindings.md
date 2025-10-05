# Python Order Event Bindings

- **Date:** 2025-10-05
- **Owner:** assistant

## Context
- Python currently receives `AccountEvent` updates via the bindings but lacks a
  structured wrapper for `barter_execution::order::OrderEvent` values.
- Integration tests emulate order updates using ad-hoc dictionaries. This
  diverges from the typed Rust surface, making it harder to keep parity when new
  `OrderState` variants or metadata are introduced.
- Bridging the strongly typed event structure will let Python consumers inspect
  order transitions (open, filled, cancelled, rejected) without reimplementing
  conversion glue.

## Goals
- Expose a `PyOrderEvent` wrapper covering the key (`OrderKey`) and state
  (`OrderState`) of execution events.
- Provide helpers to materialise events as Python dictionaries for ergonomic
  consumption in the pure Python engine components.
- Ensure the binding integrates with existing command helpers so events raised
  inside Rust-driven system handles can be forwarded directly into Python.

## Requirements
- Implement `PyOrderEvent` in `barter-python/src/command.rs` (or a dedicated
  execution module) wrapping `OrderEvent<ExchangeIndex, InstrumentIndex,
  OrderState<AssetIndex, InstrumentIndex>>`.
- Surface getters for `key`, `state`, and a `to_dict()` helper mirroring the
  JSON representation used across the workspace.
- Update the Python package (`python/barter_python/execution.py`) to re-export
  the binding and, if helpful, provide a lightweight Python-side wrapper that
  preserves type hints.
- Add pytest coverage asserting that round-tripping an `OrderEvent` through the
  binding preserves all fields and order state variants (Open, Cancelled,
  Filled, Rejected).
- Extend Rust unit tests where necessary to validate conversion logic.

## Testing
- Run `cargo test -p barter-python` and `pytest -q tests_py`.
- Add a focused pytest (e.g. `test_order_event_bindings.py`) that constructs an
  event via bindings, inspects attributes, and converts to dict.
- Ensure drawdown or unrelated analytics tests remain untouched.

## Notes
- This work bridges the remaining gap between execution order updates in Rust
  and Python without porting the full engine logic.
- Follow TDD by introducing failing Python coverage before implementing the
  binding.
- 2025-10-05: Engine audit command outputs now surface cancel / open request
  results as typed `OrderRequestCancel` / `OrderRequestOpen` wrappers with
  structured error entries for Python consumption.
