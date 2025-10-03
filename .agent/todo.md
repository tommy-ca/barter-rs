- [x] Remove tracked Python bytecode artefacts from `barter-python/tests_py/__pycache__`.
- [x] Finalize `shutdown_with_summary` bindings (Rust + Python tests + docs).
- [x] Ensure summary serialization returns rich Python objects.
- [x] Update packaging metadata (wheel classifiers, maturin settings) once API settles.
- [x] Plan integration tests covering live system lifecycle vs. backtest.
  - Captured in `.agent/specs/python-integration-tests.md` (2025-10-03).
- [x] Implement pytest integration suite per new spec.
  - [x] Add live system lifecycle coverage (`tests_py/test_integration_live.py`).
  - [x] Cover historic backtest summary scenario (`tests_py/test_integration_backtest.py`).
  - [x] Exercise command builder round-trip (`tests_py/test_integration_commands.py`).
  - [x] Capture failure surface behaviour (`tests_py/test_integration_failures.py`).
- [x] Wire integration test marker into CI workflow after maturin build.
- [x] Add tracing/log capture fixture to aid debugging slow tests.
- [x] Resolve `cargo test -p barter-python` linker failure caused by missing libpython symbols when
      building with the `extension-module` feature enabled. (2025-10-03)

- [ ] Document new feature flag workflow in developer README once stabilised.

## Planned (2025-10-03)
- [x] Run `cargo test -p barter-python` *(blocked: linker fails to find libpython; see existing TODO)*
- [x] Run `pytest -q tests_py`
- [x] Expose account event constructors in Python bindings
- [x] Add coverage in tests for account event round trip
- [x] Document binding usage in README update

## Later Opportunities
- [ ] Expand bindings for market stream events
- [ ] Add CLI example for Python package
- [ ] Evaluate packaging automation

## Completed
- [x] Expose module version constant to Python consumers.
- [x] Expand `barter-python` bindings to cover engine configuration and system control.
- [x] Mirror key configuration structs (SystemConfig) in Python API.
- [x] Provide runtime helpers to run trading system from Python via async tasks.
- [x] Add Python-level integration tests exercising basic system lifecycle.
