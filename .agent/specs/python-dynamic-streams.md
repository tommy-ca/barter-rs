# Python Dynamic Streams Bindings

Last updated: 2025-10-05

## Context
- The Python bindings expose `DynamicStreams` from `barter-data`, but the
  current implementation in `barter-python/src/data.rs` is a placeholder that
  returns `None` for all selectors and does not initialise live market
  streams.
- Without real bindings, Python users cannot consume the unified
  multi-exchange market data infrastructure that exists in the Rust crates.
- Several higher-level integration tests are blocked on the ability to drive
  market data from Python by selecting trade or order book streams.

## Goals
- Provide a `PyDynamicStreams` wrapper backed by the real
  `DynamicStreams<InstrumentIndex>` implementation.
- Expose selectors for trade, order book L1/L2, and liquidation streams that
  return Python-facing handles capable of receiving market events.
- Translate `MarketStreamResult` items into the existing Python `MarketEvent`
  representations, including reconnection markers and error propagation.
- Ensure the bindings run on a managed Tokio runtime so callers do not need to
  bootstrap their own event loop.
- Supply lightweight test utilities so unit tests can exercise the bindings
  without opening real exchange connections.

## Requirements
- Implement `PyDynamicStreams` with a shared `tokio::runtime::Runtime` and
  interior mutability to safely remove individual streams on demand.
- Provide a `PyMarketStream` helper exposing `recv(timeout: float | None)` and
  `try_recv()` methods that block on the runtime and surface events as Python
  objects.
- Support `select_trades`, `select_all_trades`, `select_l1s`,
  `select_all_l1s`, `select_l2s`, `select_all_l2s`, `select_liquidations`, and
  `select_all_liquidations` on the Python side.
- Implement `init_dynamic_streams` to convert nested batches of
  `PySubscription` objects, spawn the Rust async initialiser, and wrap the
  result.
- Add `#[cfg(test)]` helpers that build dynamic streams from in-memory
  channels so Rust unit tests can validate the bindings without network
  access.
- Reuse the existing Python data conversion helpers to avoid duplicating
  `MarketEvent` serialisation logic.

## Testing
- Add Rust unit tests that construct in-memory dynamic streams, obtain a
  Python `PyDynamicStreams` handle, and assert that the `recv`/`try_recv`
  methods yield the expected Python objects.
- Extend the pytest suite with regression tests that exercise trade stream
  selection via the new bindings, using the in-memory helper to supply fake
  market data.
- Ensure error cases propagate cleanly by pushing a `DataError` through the
  channel and asserting that Python observes a `ValueError` with the original
  message.

## Follow Ups
- Once live streaming stabilises, add async-friendly wrappers (e.g.
  `__aiter__`) using `pyo3-asyncio` so Python coroutines can drive the
  streams.
- Investigate exposing configurable reconnection policies and metrics export
  hooks to mirror the Rust ergonomics.
