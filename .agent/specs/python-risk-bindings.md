# Python Risk Binding Bridging (2025-10-04)

## Context
- Existing Python risk helpers (`risk.RiskApproved`, `risk.RiskRefused`, `risk.DefaultRiskManager`)
  are implemented purely in Python despite equivalent structs and implementations existing in
  `barter::risk` and `barter_execution` crates.
- Maintaining pure Python duplicates introduces divergence risk and prevents consumers from
  benefiting from Rust-side validation and consistency.

## Requirements
1. Provide PyO3 wrappers for the following Rust concepts:
   - `RiskApproved<T>` newtype wrapping cancel & open order requests.
   - `RiskRefused<T>` error-carrying wrapper (capture reason string).
   - `DefaultRiskManager<State>` implementation that approves all requests.
2. Accept and return `OrderRequestCancel` / `OrderRequestOpen` wrappers already exposed via
   `barter_python.command`.
3. Preserve ergonomic Python API:
   - Constructor accepting a wrapped request (for `RiskApproved`).
   - `RiskRefused.new(item, reason)` constructor and `reason` attribute.
   - `into_item()` accessor returning the wrapped order request.
   - Deterministic `__repr__` & equality semantics matching previous Python behaviour.
4. Update Python-level module to re-export the new bindings without duplicating logic.
5. Extend pytest coverage to assert wrappers originate from the extension module and that
   `DefaultRiskManager.check()` returns the expected tuple of wrapper iterables.

## Testing
- Add / update pytest scenarios in `tests_py/test_risk.py` to cover constructors,
  equality/representation, and manager behaviour using the Rust-backed bindings.
- Run `pytest -q tests_py/test_risk.py` and `cargo test -p barter-python` once bindings compile.

## Notes
- Leverage existing helpers in `command.rs` to clone request inners.
- Ensure conversions respect `OneOrMany` semantics when bridging iterables.
- Keep risk bindings module free of Python fallbacks per "NO LEGACY" principle.
