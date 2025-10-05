# Rust-Backed Instrument Name Bindings (2025-10-05)

## Motivation
- Python module `barter_python.instrument` currently defines pure Python implementations for
  `AssetNameInternal`, `AssetNameExchange`, `InstrumentNameInternal`, and
  `InstrumentNameExchange`.
- The Rust crates (`barter_instrument::asset::name` and `barter_instrument::instrument::name`)
  already expose the canonical logic for normalising identifiers, including lowercase handling
  and exchange-prefixed generation helpers.
- Maintaining duplicate logic in Python risks drift as new exchanges or naming rules are added
  in Rust.

## Goals
- Expose Rust-backed PyO3 wrappers for the four name types, mirroring their constructors and
  accessor APIs.
- Ensure Python callers can create names from strings or exchange-aware helpers while reusing the
  Rust validation and normalisation rules.
- Update the pure Python façade to reuse the bindings rather than re-implementing behaviour.

## Requirements
1. Add PyO3 classes:
   - `AssetNameInternal`
   - `AssetNameExchange`
   - `InstrumentNameInternal` (including `new_from_exchange` helper)
   - `InstrumentNameExchange`
2. Expose properties and dunder implementations to match the existing pure Python API:
   - `.name` property returning `str`
   - `__str__`, `__repr__`, equality, and hashing semantics.
3. Update `python/barter_python/instrument.py` to re-export the Rust-backed classes without
   breaking user-facing imports.
4. Extend pytest coverage to assert that `barter_python.instrument` re-exports the new bindings
   and that the behaviour matches the existing expectations (normalisation, exchange helper).

## Testing
- Add targeted tests in `tests_py/test_instrument.py` validating:
  - `barter_python.AssetNameInternal` exists and normalises uppercase inputs.
  - `barter_python.instrument.AssetNameInternal is barter_python.AssetNameInternal`.
  - `InstrumentNameInternal.new_from_exchange` correctly prefixes the exchange ID while using the
    Rust binding.
- Run existing instrument-related tests to ensure no regressions.

## Rollout
- Keep pure Python name classes removed/aliased in favour of the bindings to prevent future
  divergence.
- Document the change in `.agent/plan.md` notes if further follow-up is required (e.g. porting
  remaining instrument structures).
