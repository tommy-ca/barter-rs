# Porting Plan

**Last Updated:** 2025-10-02

## Vision
- Deliver ergonomic Python bindings for Barter's Engine, data streams, and risk components without duplicating core logic.
- Keep the Rust workspace authoritative while enabling Python users to compose trading workflows.

## Guiding Principles
- Reuse existing Rust crates and APIs whenever possible; bindings should be thin adapters.
- Aim for feature parity with core Rust Engine flows before expanding to advanced use cases.
- Prioritise runtime safety and deterministic behaviour across the FFI boundary.
- Maintain test coverage via a mix of Rust unit tests and Python integration tests (approx. 80/20 effort split per instructions).

## Workstreams
1. **Architecture Survey:** Catalogue Barter crates, dependencies, and public APIs relevant to bindings.
2. **Binding Design:** Decide on PyO3/maturin tooling, module layout, and async story.
3. **Implementation:** Expose Engine configuration, lifecycle management, and core data types to Python.
4. **Testing:** Add Rust-level assertions for FFI helpers plus Python end-to-end scenarios.
5. **Distribution:** Package wheels via maturin, integrate CI, and document Python usage.

## Immediate TODOs
- [x] Map the minimal set of Engine APIs needed for MVP Python usage. *(2025-10-02)*
- [ ] Review the `barter-bindings` repository for prior art and reusable assets. *(blocked — GitHub repo access denied on 2025-10-03)*
- [x] Prototype a dedicated `barter-python` crate with PyO3 and maturin config.
- [x] Define data marshaling strategy for market events and commands across the FFI boundary. *(2025-10-02)*
- [x] Draft a testing matrix covering core Engine flows invoked from Python. *(documented in `.agent/specs/python-integration-tests.md`, 2025-10-03)*
- [x] Expand Python packaging metadata plus tooling for wheel builds and integration tests. *(CLI entrypoint + mixed-project layout updated 2025-10-03)*

## Engine Command Surface (MVP)
- **TradingState Update**: toggle live trading via `EngineEvent::TradingStateUpdate` with `TradingState::{Enabled, Disabled}`.
- **Graceful Shutdown**: trigger system shutdown using `EngineEvent::Shutdown`.
- **SendOpenRequests**: submit one or many `OrderRequestOpen` payloads (market or limit) keyed by exchange, instrument, strategy, and client order id.
- **SendCancelRequests**: mirror `OrderRequestCancel` to revoke outstanding orders by key and optional order id.
- **ClosePositions**: drive position unwinds using `InstrumentFilter::{None, Exchanges, Instruments, Underlyings}`.
- **CancelOrders**: cancel orders matching an `InstrumentFilter` without explicit order ids.

Notes:
- Engine events can now cross the FFI boundary via serde-backed JSON/dict helpers.
- Target Python ergonomics: simple constructors for keys, order requests, and instrument filters feeding higher-level helpers on `SystemHandle`.
