# Python Execution Balance Bindings (2025-10-05)

## Context
- The Python package still defines pure Python `Balance` and `AssetBalance`
  dataclasses that mirror structures in `barter-execution`.
- Rust-based bindings now cover other execution types (orders, trades,
  snapshots), so Python-only balance implementations are an inconsistency.
- Exposing Rust-backed wrappers will keep behaviour aligned with the core
  engine and enable other bindings to re-use the same types.

## Goals
- Provide PyO3 wrappers for `Balance` and `AssetBalance` from
  `barter-execution`.
- Ensure Python users construct balances via the extension module while
  keeping ergonomics similar to the existing dataclasses.
- Update existing account snapshot bindings to emit the new wrappers.

## Requirements
- Add `PyBalance` with constructors, `total` / `free` getters, `used` helper,
  string representations, equality, and hashing.
- Add `PyAssetBalance` exposing `asset` (index), `balance` (`PyBalance`), and
  `time_exchange` fields plus readable representations.
- Integrate the wrappers into `PyAccountSnapshot` so Python callers receive
  Rust-backed values when iterating balances.
- Remove or replace the pure Python dataclasses in
  `python/barter_python/execution.py`, re-exporting the new bindings instead.

## Testing
- Extend `tests_py/test_execution.py` to cover creating `Balance` &
  `AssetBalance`, equality / hashing behaviour, and interactions inside account
  snapshots.
- Run `cargo test -p barter-python` and `pytest -q tests_py` to validate the
  bindings end-to-end.
