# Python Sequence Bindings (2025-10-05)

## Context
- Python integrations currently observe engine audit metadata with raw integers for event
  sequencing, losing parity with the strongly typed `barter::Sequence` wrapper used across the
  Rust core.
- Without bridging `Sequence`, consumers cannot checkpoint or advance engine processing using
  the same invariants enforced in Rust, limiting cross-language feature work.

## Goals
- Expose a PyO3 `PySequence` wrapper that mirrors the behaviour of `barter::Sequence`.
- Ensure engine audit context emitted to Python includes the strongly typed wrapper instead of a
  bare integer.
- Provide ergonomic helpers for Python callers to read, clone, and advance sequences.

## Requirements
- Implement `PySequence` storing the underlying `barter::Sequence` value with methods:
  - `value` getter returning the underlying `u64`.
  - `fetch_add` mutating in-place and returning the previous `Sequence` value as a new wrapper.
  - `next_value` helper returning the post-increment integer while keeping the underlying sequence
    advanced.
  - Rich comparison, `__int__`, and `__repr__` implementations to integrate cleanly with Python
    ergonomics.
- Provide internal conversion helpers (`from_inner`, `clone_inner`) so other bindings can surface
  sequences without manual integer handling.
- Update audit context translation in `system.rs` to surface `PySequence` instances.
- Re-export the new class from the extension module so Python users can construct sequences when
  needed.

## Testing
- Add Rust unit coverage asserting `fetch_add` parity with the Rust implementation and ensuring
  `value` reports the latest sequence counter.
- Extend Python integration tests to verify audit snapshots expose `PySequence` objects and that
  direct bindings support mutation semantics from Python code.
- Add pytest coverage demonstrating comparisons, `int()` coercion, and increment helpers.

## Follow-Ups
- Consider exposing helper constructors for sequence checkpoints on engine handles once the core
  wrapper is stable.

## Status (2025-10-05)
- Implemented `PySequence` wrapper with parity methods (`value`, `fetch_add`, `next_value`) and
  Python comparison/representation support.
- Updated audit context translation to emit `PySequence` objects and added both Rust unit coverage
  and pytest assertions validating the new behaviour.
