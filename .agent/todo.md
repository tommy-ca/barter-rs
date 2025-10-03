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
- [ ] Add tracing/log capture fixture to aid debugging slow tests.
- [ ] Resolve `cargo test -p barter-python` linker failure caused by missing libpython symbols when
      building with the `extension-module` feature enabled.

## Completed
- [x] Expose module version constant to Python consumers.
- [x] Expand `barter-python` bindings to cover engine configuration and system control.
- [x] Mirror key configuration structs (SystemConfig) in Python API.
- [x] Provide runtime helpers to run trading system from Python via async tasks.
- [x] Add Python-level integration tests exercising basic system lifecycle.
