# Python System Feed Mode

Last updated: 2025-10-05

## Context
- `start_system` and `run_historic_backtest` in the Rust extension currently hardcode
  `EngineFeedMode::Stream`.
- The underlying `SystemBuilder` supports both `Iterator` and `Stream` feed modes, and the
  Barter `Engine` exposes different runtime characteristics depending on the selection.
- Python users need the ability to select the feed mode to match their execution environment or
  backtest runner architecture.

## Requirements
- Extend the Python bindings to accept an optional `engine_feed_mode` argument for both
  `start_system` and `run_historic_backtest`.
- Support values `"stream"` (default) and `"iterator"`, accepting case-insensitive inputs.
- Surface an informative `ValueError` when an unsupported value is provided.
- Ensure seeded balances and existing parameters continue to work with the new argument.
- Re-export the available feed mode options from the Python convenience layer for discoverability.

## Testing Strategy
- Python integration test: `start_system(..., engine_feed_mode="iterator")` runs and can be
  cleanly shut down via `handle.shutdown_with_summary`.
- Python integration test: `run_historic_backtest(..., engine_feed_mode="iterator")` returns a
  trading summary matching the default mode.
- Python unit test: unknown feed mode string raises `ValueError` with the offending value.
- Rust unit test: parser helper converts recognised strings into the correct `EngineFeedMode` and
  rejects invalid input.

