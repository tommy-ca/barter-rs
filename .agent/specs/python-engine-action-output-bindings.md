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

## Testing Strategy
- Add Rust unit tests covering conversions between `ActionOutput` values and
  the PyO3 wrappers (including the JSON fallback).
- Extend Python integration tests to assert audit events now yield structured
  `ActionOutput` wrappers with accessible attributes.
- Update existing audit-related tests to validate equality and representation
  of the new Python classes.
