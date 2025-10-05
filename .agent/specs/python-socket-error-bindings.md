# Python Socket Error Bindings

- **Date:** 2025-10-05
- **Owner:** assistant

## Context
- Python bindings currently expose many `barter-integration` utilities but lack a
  typed wrapper or exception bridge for `SocketError` values.
- Integration helpers and runtime flows convert errors into generic strings,
  making it difficult for Python callers to branch on error categories such as
  `HttpTimeout`, `Subscribe`, or `Unsupported`.
- Exposing `SocketError` as a structured Python exception keeps parity with the
  Rust crate and improves ergonomics for composing other bindings that rely on
  integration connectivity.

## Goals
- Provide a `SocketError` binding that surfaces the Rust enum variants and
  payloads (where applicable) to Python consumers.
- Allow Python callers to inspect the error kind via attributes and use
  `isinstance` checks when catching exceptions raised from Rust-driven flows.
- Ensure the binding integrates with existing system helpers so future Rust
  code returning `SocketError` propagates through `PyErr` seamlessly.

## Requirements
- Implement a `PySocketError` wrapper exposed from the Rust extension module
  with variant-specific constructors and human-readable `__repr__` output.
- Map the wrapper to a custom Python exception type (e.g. `barter_python.SocketError`).
- Provide helper methods to convert from `barter_integration::error::SocketError`
  into the Python wrapper and back where feasible.
- Document the new binding in the Python package namespace and ensure it is
  re-exported for downstream imports.
- Maintain `#![forbid(unsafe_code)]` guarantees and follow existing module
  organisation conventions.

## Testing
- Add Rust unit tests asserting that converting a `SocketError` into a Python
  exception yields the expected type and message for representative variants.
- Add pytest coverage catching the new exception when a Rust helper returns a
  `SocketError`, verifying attributes such as `.kind` and associated payloads.
- Run `cargo test -p barter-python` and `uv run pytest -q tests_py` as part of
  the workflow.

## Notes
- Keep the binding minimal; focus on bridging the enum rather than porting the
  entire integration error stack.
- Avoid introducing mocks—prefer real conversions or lightweight Rust helpers
  invoked from tests.
- Document follow-up ideas (e.g. bridging `DataError`) separately if needed.
