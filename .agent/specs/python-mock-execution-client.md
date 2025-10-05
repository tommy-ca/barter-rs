# Python Mock Execution Client Bindings

Last updated: 2025-10-05

## Context
- The Python package currently exposes mock execution configuration types but lacks a
  programmable client for interacting with the Rust `MockExecution`/`MockExchange` stack.
- Bridging the client unlocks lightweight execution simulations directly from Python without
  spinning up the full engine, enabling faster TDD cycles for strategy prototyping.

## Goals
- Provide a PyO3-backed `MockExecutionClient` harness that wraps
  `barter_execution::client::mock::MockExecution` and manages the underlying `MockExchange` task.
- Support synchronous helpers (`account_snapshot`, `fetch_balances`, `fetch_open_orders`,
  `fetch_trades`) returning Python-native structures for rapid inspection.
- Expose an incremental event poller that surfaces account stream updates as Python dictionaries.
- Maintain ergonomic construction by reusing `MockExecutionConfig` and
  `ExecutionInstrumentMap` information already surfaced in the bindings.

## Non-Goals
- Deliver a fully async Python surface (reuse the existing Tokio runtime approach instead).
- Provide bindings for live exchange clients—limit work to the mock client harness for now.

## Requirements
- Extend the Rust bindings with a `PyMockExecutionClient` type that:
  - Accepts a `MockExecutionConfig` and `ExecutionInstrumentMap` for setup.
  - Spawns the `MockExchange::run` task on a managed Tokio runtime.
  - Lazily initialises and caches the account stream (`account_stream`) channel internally.
  - Exposes blocking methods wrapping the async client calls via `Runtime::block_on`.
  - Converts returned structs to Python dictionaries using `serde_json` round-trips to avoid
    duplicating mapping logic into index-based types.
  - Ensures drop glue waits for the exchange task to finish (join handle).
- Surface the new class and helper constructors in `barter_python/__init__.py` and the
  pure-Python execution facade for consistent ergonomics.
- Provide user-facing docstrings explaining lifecycle (construction, snapshot fetch, poll loop,
  shutdown semantics) with examples.

## Testing Strategy
- Add targeted pytest coverage under `tests_py/test_execution.py` exercising:
  - Creating a client from a minimal execution map and config.
  - Fetching an account snapshot and verifying expected balances/orders.
  - Polling at least one event from the mock stream after an order is opened.
- Add a Rust unit smoke test if practical (optional) asserting the helper conversion logic.
- Verify the full suite: `cargo test -p barter-python` and `pytest -q tests_py`.

## Follow-ups
- Consider exposing convenience helpers to translate unindexed snapshots/events into the typed
  wrappers once demand appears.
- Revisit async ergonomics (`pyo3-asyncio`) after baseline blocking API stabilises.
