# barter-data Python Binding Notes (2025-10-04)

## Current Coverage
- Exposed exchange identifiers via `PyExchangeId` with major venues.
- Expanded to expose the full `ExchangeId` enumeration in the Rust bindings (2025-10-04).
- Python `SubKind` now includes `PUBLIC_TRADES`, `ORDER_BOOKS_L1`, `ORDER_BOOKS_L2`, `ORDER_BOOKS_L3`, `LIQUIDATIONS`, and `CANDLES`.
- `PySubscription` supports spot, perpetual, future, and option market data instruments.
- Added `exchange_supports_instrument_kind` helper and `PySubscription.is_supported()` to mirror
  Rust validation utilities (2025-10-05).
- Instrument kind mapping parses expiry, strike, option kind, and exercise fields.

## Remaining Gaps / Follow-ups
- Monitor future upstream additions for new exchange identifiers.
- Expose builder helpers for dynamic stream selectors once Rust APIs stabilize.
- Support more complex instrument kinds (e.g., options with settlement metadata) if required by upstream crates.
- Provide validation utilities for subscription dictionaries on the Python side for clearer UX errors.

## Testing Strategy
- Rely on `tests_py/test_bindings.py::test_subscription_creation` for coverage of SubKind and instrument variants.
- Consider adding round-trip tests for future option serialization in Rust integration tests (`python_smoke.rs`).
