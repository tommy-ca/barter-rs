# Porting Plan (2025-10-03)

## Immediate Objectives
1. Review existing bindings and overall repository state (2025-10-03 ✅).
2. Outline desired Python API surface and prioritise components for the initial release (2025-10-03 ✅).
3. Implement Rust-to-Python wrapper modules and supporting ergonomics (2025-10-03 ✅).
4. Integrate packaging, build scripts, and CI wiring to publish Python artifacts (2025-10-03 ✅).
5. Add essential unit & end-to-end tests alongside updated documentation (initial integration
   suite implemented 2025-10-03; see `.agent/specs/python-integration-tests.md`) (2025-10-03 ✅).

## Completed (2025-10-04)
1. Establish cross-language maintenance workflow (Rust + Python) including branching strategy and release cadence.
2. Identify remaining Rust APIs requiring Python exposure (risk manager configuration ✅ 2025-10-04; portfolio analytics extensions — profit factor, win rate, rate of return — shipped 2025-10-04; initial balance seeding ✅ 2025-10-04).
3. Produce incremental TDD plan emphasising new bindings with paired Rust/Python coverage.
4. Align CI to run `cargo test`, `pytest`, and packaging checks on every push & PR (✅ 2025-10-04 via `.github/workflows/ci.yml`).
5. Prepare developer onboarding notes for maintaining the hybrid workspace (✅ 2025-10-04; see `docs/developer-onboarding.md`).

## Pure Python Porting Progress (2025-10-04)
1. ✅ Implement core barter-instrument data structures in pure Python:
    - Side enum (Buy/Sell)
    - ExchangeId enum with all 30+ exchanges
    - Asset data structures (Asset, AssetNameInternal, AssetNameExchange)
    - Underlying generic structure for base/quote pairs
    - Keyed generic wrapper
    - Instrument name structures (internal/exchange)
    - Instrument quote asset enum
    - Option kinds and exercise styles
    - Contract types (Perpetual, Future, Option)
    - InstrumentKind enum with all variants
    - Full Instrument class with spot creation and utilities
 2. ✅ Add comprehensive unit tests (55 tests covering all structures)
 3. ✅ Apply modern Python practices: type hints, dataclasses-like behavior, proper equality/hashing
 4. ✅ Follow TDD with 80% implementation focus, maintain SOLID/KISS/DRY principles

## Pure Python Porting Progress (2025-10-04) - Continued
1. ✅ Implement core barter-execution data structures in pure Python:
    - OrderKind enum (Market/Limit)
    - TimeInForce enum (GTC, GTD, FOK, IOC)
    - ID classes (ClientOrderId, OrderId, StrategyId)
    - Balance and AssetBalance structures
    - Trade and TradeId structures
    - AssetFees structure
    - OrderKey and OrderEvent structures
    - RequestOpen and RequestCancel structures
    - ActiveOrderState variants (OpenInFlight, Open, CancelInFlight)
    - InactiveOrderState variants (Cancelled, FullyFilled, Expired, OpenFailed)
    - OrderState enum
    - Full Order class
    - Account event structures (AccountEvent, AccountEventKind, AccountSnapshot, etc.)
 2. ✅ Add comprehensive unit tests (100+ tests covering all structures)
 3. ✅ Apply modern Python practices: type hints, dataclasses-like behavior, proper equality/hashing
 4. ✅ Follow TDD with 80% implementation focus, maintain SOLID/KISS/DRY principles

 ## Pure Python Porting Progress (2025-10-04) - Continued
1. ✅ Implement core barter-integration data structures in pure Python:
     - SubscriptionId for stream subscriptions
     - Metric/Tag/Field/Value for metrics collection
     - Snapshot wrapper for data snapshots
     - SnapUpdates for snapshot with updates
  2. ✅ Add comprehensive unit tests (29 tests covering all structures)
  3. ✅ Apply modern Python practices: type hints, dataclasses-like behavior, proper equality/hashing
  4. ✅ Follow TDD with 80% implementation focus, maintain SOLID/KISS/DRY principles

 ## Pure Python Porting Progress (2025-10-04) - Completed
1. ✅ Implement core barter-data market event structures in pure Python:
      - Candle with OHLCV data and trade count
      - Liquidation with side, price, quantity, and timestamp
      - Full OrderBook with sequence, time_engine, and sorted bid/ask sides
      - OrderBookSide with Bids/Asks tagging and automatic level sorting
      - Updated DataKind to use proper data objects instead of placeholders
      - Helper functions for type-safe market event casting
   2. ✅ Add comprehensive unit tests (25+ additional tests covering all new structures)
   3. ✅ Apply modern Python practices: type hints, proper equality/hashing, decimal precision
   4. ✅ Follow TDD with 80% implementation focus, maintain SOLID/KISS/DRY principles

 ## Pure Python Porting Progress (2025-10-04) - Statistic Module Completed
1. ✅ Implement core barter-statistic module in pure Python:
      - TimeInterval protocol and implementations (Annual365, Annual252, Daily, TimeDeltaInterval)
      - SharpeRatio with calculate() and scale() methods (square root time scaling)
      - SortinoRatio with calculate() and scale() methods (downside deviation focus)
      - CalmarRatio with calculate() and scale() methods (maximum drawdown focus)
      - ProfitFactor calculator with special cases for zero profits/losses
      - WinRate calculator for trade success ratios
      - RateOfReturn with linear time scaling (unlike risk metrics)
   2. ✅ Add comprehensive unit tests (42 tests covering all structures and edge cases)
   3. ✅ Apply modern Python practices: type hints, dataclasses, proper equality/hashing, decimal precision
   4. ✅ Follow TDD with 80% implementation focus, maintain SOLID/KISS/DRY principles

## Notes
- Maintain commit discipline with atomic changes (commit & push each step).
- Balance effort with ~80% focused on core porting work, ~20% on testing scaffolding.
- Use `.agent` directory for scratch notes and future TODOs.

## Prior Roadmap Snapshot
1. ✅ Audit existing Rust crates and current `barter-python` module.
2. ✅ Design binding architecture, build tooling, and packaging approach.
3. ✅ Implement core binding modules and integrate with Rust components.
4. ✅ Add Python packaging metadata plus unit and end-to-end tests (integration suite landed
   2025-10-03; automated wheel publishing wired up via `python-wheels` workflow).
5. 🔄 Refresh documentation, examples, and CI pipelines for the hybrid project (README python
   quickstart updated 2025-10-03; release notes section added 2025-10-04; further updates pending).
