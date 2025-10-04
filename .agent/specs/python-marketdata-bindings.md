# Python Market Data Bindings (2025-10-04)

## Context
- The Python package currently parses historical market data JSON directly in
  `python/barter_python/backtest.py`.
- The existing implementation only supports a subset of `DataKind` variants and
  duplicates parsing logic that already exists in the Rust crates.
- Bridging to the Rust `barter::backtest::market_data::MarketDataInMemory`
  infrastructure keeps behaviour aligned with the engine while reducing
  maintenance overhead.

## Goals
- Expose a `MarketDataInMemory` wrapper from Rust that Python can construct from
  JSON files or in-memory `MarketStreamEvent` collections.
- Convert loaded events into the canonical Python data structures
  (`barter_python.data.MarketEvent`, `DataKind`, etc.) to preserve ergonomics.
- Provide synchronous helpers that Python can wrap into async generators for
  existing backtest interfaces.

## Requirements
- Implement a PyO3 `PyMarketDataInMemory` type storing the underlying Rust
  `MarketDataInMemory<DataKind>` along with the parsed events.
- Support construction via `from_json_file(path: &str)` using serde to decode
  the Barter JSON market data format (including reconnect markers, which should
  be skipped for now).
- Surface helpers for:
  - `time_first_event()`: returning the first `time_exchange` timestamp.
  - `events(py)`: returning a list of Python `MarketEvent` instances for
    streaming.
- Reuse existing Python classes (`PublicTrade`, `Candle`, `OrderBookL1`,
  `Liquidation`, `DataKind`) when materialising events on the Python side.
- Ensure the wrapper is unsendable and cloneable to avoid accidental sharing
  across threads without explicit handling.

## Testing
- Add pytest coverage validating that `MarketDataInMemory.from_json_file`
  returns deterministic events matching the Rust-parsed representation.
- Verify the first event timestamp matches the JSON fixture and that events are
  returned in order.
- Confirm support for at least trade and candle variants using synthetic JSON
  fixtures.

## Follow-Up Ideas
- Extend the wrapper to expose an async-compatible iterator via
  `pyo3_asyncio` once the synchronous surface is stable.
- Consider surfacing additional constructors (e.g. from in-memory Python
  dictionaries) to simplify test authoring without touching the filesystem.
