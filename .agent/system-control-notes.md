# Engine Control Notes

**Date:** 2025-10-02

- `barter::system::SystemBuilder` drives creation of `System` instances and exposes
  builder flags (`EngineFeedMode`, `AuditMode`, `TradingState`).
- `System` handles runtime lifecycle and command fan-out; exposes `shutdown`,
  `abort`, `shutdown_after_backtest`, and direct helpers for trading commands.
- `EngineEvent` wraps shutdown, trading state updates, account streams, market streams,
  and trading commands, making it the natural Python FFI surface for feeding events.
- Python bindings should surface a managed runtime handle that owns the `System`
  and provides async-safe entry points for sending engine commands.
- Command coverage should include trading state toggles, order submission and
  cancellation, and position management via `InstrumentFilter`.
- Backtest helper already exists (`run_historic_backtest`); live/paper trading
  requires feeding events over `feed_tx` and driving shutdown explicitly.

