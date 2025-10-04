# Python Initial Balance Seeding (2025-10-04)

## Objective
- Allow Python bindings to seed initial engine balances when starting systems or
  executing historic backtests so that portfolio state can mirror external
  account funding.

## Requirements
- Extend `barter_python.start_system` and `barter_python.run_historic_backtest`
  with an optional `initial_balances` argument.
- Accept an iterable of mapping-like objects containing `exchange`, `asset`,
  `total`, and optional `free` fields.
- Normalise `exchange` using the existing `ExchangeId` snake_case variants and
  validate that both totals convert to finite decimals using the existing
  `parse_decimal` helper logic.
- Default the `free` amount to the provided `total` when omitted and reject
  payloads where `free` exceeds `total`.
- Override any preconfigured balances for matching `(exchange, asset)` pairs in
  the engine state; do not mutate the provided `SystemConfig`.
- Preserve existing behaviour when the argument is omitted or `None`.

## Testing
- Add pytest coverage asserting that seeding balances reflects in the returned
  `TradingSummary` asset tear sheet after shutting down a live system handle.
- Ensure parsing errors (unknown exchange identifiers, non-finite numeric
  inputs) raise `ValueError` with descriptive messaging.

## Notes
- Use the `.agent` memory to surface follow-up tasks if additional audit
  surfacing is required later.
