# Python Engine Action Output Bindings (2025-10-05)

## Overview
- Provide typed Python wrappers for `barter::engine::action::ActionOutput` and
  related `SendRequestsOutput` structures emitted via engine audits.
- Replace the current JSON-oriented dictionaries returned by the bindings with
  ergonomic classes that expose structured attributes and helper methods.

## Requirements
- Implement `PyActionOutput` enum-like wrapper with variants for `CancelOrders`,
  `OpenOrders`, `ClosePositions`, and a fallback `Other` representation wrapping
  arbitrary JSON payloads.
- Implement `PySendRequestsOutput` wrapper exposing `sent` and `errors`
  collections with sequence semantics (`len`, iteration, `repr`).
- Update `PyAuditEvent` construction to emit the new wrappers inside
  `outputs` / `errors` collections.
- Ensure close-position outputs expose `cancels` and `opens` as typed
  `PySendRequestsOutput` wrappers.
- Re-export the new wrappers from the Python package for direct consumption.

## Python API
- `PyActionOutput`
  - Provide a `variant` property returning the canonical variant string.
  - Accessors `cancel_orders` & `open_orders` return `SendRequestsOutput` when
    the variant matches, otherwise `None`.
  - `close_positions` accessor returns a dedicated `ClosePositionsOutput`
    wrapper exposing `cancels` & `opens` as `SendRequestsOutput` values.
  - `other` accessor returns the JSON payload wrapped for unsupported variants.
  - Implement `__repr__` to surface the variant and emptiness flags.
- `PyClosePositionsOutput`
  - Holds `cancels` & `opens` as `SendRequestsOutput` wrappers.
  - Implements `__repr__` delegating to the child wrappers for readability.
- `PySendRequestsOutput`
  - Expose `sent` & `errors` properties returning `NoneOneOrMany` wrappers.
  - Provide ergonomic helpers: `is_empty`, `to_list`, `errors_to_list`.
  - Implement `__len__` (delegating to `sent.len()`), `__iter__`, and
    `__repr__` for concise debugging output.

## Testing Strategy
- Add Rust unit tests covering conversions between `ActionOutput` values and
  the PyO3 wrappers (including the JSON fallback).
- Extend Python integration tests to assert audit events now yield structured
  `ActionOutput` wrappers with accessible attributes.
- Update existing audit-related tests to validate equality and representation
  of the new Python classes.
