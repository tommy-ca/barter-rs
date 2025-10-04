# Integration Snapshot Bindings (2025-10-04)

## Objectives
- Surface Rust-backed `barter_integration::snapshot::Snapshot` and `SnapUpdates` types in the
  Python extension module.
- Remove the pure-Python reimplementation from `python/barter_python/integration.py` in favour of
  re-exporting the new bindings.
- Provide ergonomic constructors and accessors for Python callers while keeping the wrappers
  unsendable and cloneable.

## Requirements
- Define `PySnapshot` that stores a `Snapshot<PyObject>` internally and exposes:
  - `__new__(value)` constructor accepting any Python object.
  - `.value` property returning the stored object.
  - `.map(callable)` returning a new `PySnapshot` with the mapped value.
  - Rich comparison based on the wrapped value.
  - `__repr__` mirroring `Snapshot(value=...)` formatting.
- Define `PySnapUpdates` that stores `snapshot` and `updates` fields, each as `PyObject`, and exposes:
  - `__new__(snapshot, updates)` constructor requiring a `PySnapshot` for the snapshot argument.
  - `.snapshot` and `.updates` accessors.
  - Rich comparison and `__repr__`.
- Update `lib.rs` module exports to register both classes.
- Update `python/barter_python/integration.py` to re-export the Rust wrappers and keep backwards
  compatibility for public names.

## Tests
- Add pytest coverage verifying construction, value access, equality, mapping, and repr formatting for
  both wrappers.
- Add a regression test confirming that `barter_python.integration.Snapshot` and `SnapUpdates`
  resolve to the Rust-backed types exposed through the extension module.

## Follow-Ups
- Evaluate exposing typed aliases for common snapshot payloads (e.g. order book snapshots) once the
  base wrappers are in place.
