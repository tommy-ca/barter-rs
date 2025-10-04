# Python Audit Stream Bindings

- **Author:** assistant
- **Date:** 2025-10-04

## Background

- `System::take_audit` exposes `SnapUpdates` containing a full engine state snapshot and an
  `UnboundedRx` stream of audit ticks.
- The current Python bindings disable audit mode entirely, preventing Python clients from
  observing engine state replicas or processing audit updates in real time.
- Bridging this functionality unlocks monitoring and UI integrations without reimplementing core
  Rust logic.

## Goals

- Allow Python callers to enable audit streaming when starting systems or backtests.
- Surface the initial audit snapshot as structured Python data (dict-based) via existing
  `Snapshot`/`SnapUpdates` wrappers.
- Provide a Python wrapper over the audit update receiver with synchronous `recv` and
  non-blocking `try_recv` helpers driven by the embedded Tokio runtime.
- Ensure the new API is covered by unit / integration tests and documented in the Python README.

## Non-Goals

- Exposing the full `EngineState` API with dedicated Python classes.
- Building async/await native integrations for audit streams (initial cut can be blocking).
- Persisting audit data beyond in-memory streaming.

## Acceptance Criteria

1. `bp.start_system(..., audit=True)` enables audit streaming and returns `SnapUpdates` when
   `handle.take_audit()` is called.
2. The snapshot contains a JSON-like dict with `event` & `context` keys (sequence + timestamp).
3. The audit updates wrapper exposes `recv(timeout=None)` and `try_recv()` methods that return
   dictionaries representing `AuditTick`s or `None` when exhausted.
4. Calling `take_audit()` without enabling audit returns `None`.
5. Python integration tests assert the behaviours above; documentation highlights the new usage.

## Open Questions

- Future work may convert the blocking update API into an async iterator once the binding surface
  stabilises.
