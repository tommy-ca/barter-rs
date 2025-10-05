# Python Audit Tick Bindings

## Context
- Barter's `EngineAudit` stream carries structured audit ticks describing each processed engine event.
- Existing Python bindings expose audit updates as loosely-typed dictionaries, which hinders ergonomics and static analysis.
- Consumers need richer, typed accessors that align with the Rust audit model while preserving the current JSON-like helpers for backwards compatibility.

## Goals
- Provide typed Python wrappers for audit tick context and event summaries.
- Ensure audit updates can yield either the existing dictionary representation or the new typed wrapper without breaking current callers.
- Leverage existing `Sequence`, `NoneOneOrMany`, and `EngineEvent` helpers within the extension module.
- Add pytest coverage verifying the new typed bindings alongside dictionary backwards compatibility.

## Non-Goals
- Do not expose the full generic `EngineOutput` payloads (they remain serialized for now).
- No changes to the underlying Rust engine audit pipeline beyond binding layer transformations.
- Avoid introducing optional dependencies or mocks; reuse the live system integration harness.

## Acceptance Criteria
- New PyO3 classes for audit context, event summary, and tick exist with read-only properties.
- `AuditUpdates` exposes typed `recv_tick` and `try_recv_tick` helpers returning the new classes.
- Dictionary-based helpers continue to function unchanged.
- New integration test demonstrates both typed and dictionary flows.
- `cargo test -p barter-python` and `uv run pytest -q tests_py/test_integration_live.py` succeed.
