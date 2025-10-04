# Python Mock Execution Bindings

Last updated: 2025-10-04

## Goal
- Replace the pure Python `MockExecutionConfig` placeholder with bindings to the Rust `barter_execution::client::mock::MockExecutionConfig` and related types.
- Allow constructing execution configurations programmatically from Python while reusing the existing account snapshot wrappers.
- Ensure E2E tests can exercise mock execution flows through the PyO3 bindings.

## Requirements
- Expose a `MockExecutionConfig` PyO3 wrapper that mirrors all struct fields: `mocked_exchange`, `initial_state`, `latency_ms`, and `fees_percent`.
- Provide conversion helpers to and from `PyAccountSnapshot` for the `initial_state` field.
- Extend bindings with an `ExecutionConfig.mock(mock_config)` constructor so Python users can append execution configs into `SystemConfig`.
- Update the Python package (`python/barter_python`) to re-export these bindings and deprecate the existing pure Python placeholders.
- Add rust-level doctest or unit coverage if practical, plus Python pytest coverage to confirm:
  - Round-trip construction (`PyMockExecutionConfig` -> Python repr -> clone).
  - Integration with `SystemConfig` loading and runtime bootstrapping, ensuring seeded balances interact correctly.

## Testing Strategy
- Write a new pytest verifying that constructing a `MockExecutionConfig` with custom latency/fees is reflected when serialising the parent `SystemConfig` to dict/JSON.
- Add a focussed Rust unit test if necessary to validate conversions, otherwise rely on Python tests.
- Run `pytest -q tests_py` and `cargo test -p barter-python` to ensure regressions are caught.

## Follow-ups
- Consider exposing additional helpers for building common account snapshots in Python if the API proves verbose.
- Evaluate whether to surface mock exchange event streams for deterministic simulation scenarios.
