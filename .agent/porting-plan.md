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
- [ ] Map the minimal set of Engine APIs needed for MVP Python usage.
- [ ] Review the `barter-bindings` repository for prior art and reusable assets.
- [ ] Prototype a dedicated `barter-python` crate with PyO3 and maturin config.
- [ ] Define data marshaling strategy for market events and commands across the FFI boundary.
- [ ] Draft a testing matrix covering core Engine flows invoked from Python.
