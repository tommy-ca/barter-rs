# Python Market Stream Event Bindings

- **Status:** Draft
- **Last Updated:** 2025-10-05

## Overview
- Provide typed Python wrappers for `barter_data::streams::reconnect::Event` so reconnecting
  notifications and market items are exposed as structured objects instead of loosely typed
  dictionaries.
- Ensure the bindings continue to lean on the existing `barter-data` Rust models with minimal
  duplication, maintaining FFI safety and ergonomics.

## Requirements
- Introduce Python classes capturing the two event variants:
  - `MarketStreamReconnecting` with attributes `exchange` and `kind == "reconnecting"`.
  - `MarketStreamItem` with attributes `event` (typed `MarketEvent`) and `kind == "item"`.
- Export the new classes from `barter_python.data` and make them hashable, comparable, and
  provide informative `__repr__` outputs.
- Update the Rust binding layer to construct these classes when translating
  `MarketStreamResult` values for Python consumers.
- Preserve backwards compatibility for existing APIs while steering users toward the structured
  wrappers (avoid breaking signatures where possible).

## Testing
- Extend `tests_py/test_dynamic_streams.py` (and related suites) to assert the new wrappers are
  returned, verify attribute contents, and ensure equality / repr semantics.
- Add regression coverage for reconnecting events emitted during dynamic stream rebuilds.

