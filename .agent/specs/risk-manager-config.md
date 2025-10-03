# Risk Manager Configuration Exposure

Last updated: 2025-10-03

## Context
Python bindings currently allow loading `SystemConfig` JSON definitions but offer no ergonomic way
to inspect or tweak risk manager parameters before launching a system. Upcoming extensions to the
Rust risk modules introduce configurable position sizing, leverage limits, and per-instrument
exposure caps that should be surfaced in Python to keep parity with Rust clients.

## Requirements
- Provide readonly accessors for risk configuration embedded within `SystemConfig`.
- Allow Python code to override risk thresholds programmatically prior to starting a system.
- Preserve serialization symmetry: any mutated configuration should round-trip via `to_dict()` and
  `to_json()`.
- Maintain validation guarantees from Rust (reject invalid thresholds with informative errors).

## Proposed Approach
1. Extend Rust `SystemConfig` (or associated structs) with builder-style helpers for risk options
   when available.
2. Implement PyO3 wrappers exposing getters/setters for the new configuration fields.
3. Add unit tests in Rust verifying Python-driven modifications reflect in underlying config.
4. Add Python integration tests that:
   - Load baseline config.
   - Override risk limits from Python.
   - Launch engine in dry-run mode and assert enforcement (via simulated violations).
5. Update documentation and examples to demonstrate adjusting risk parameters in Python.

## Open Questions
- Do we require partial updates (per instrument) or whole-structure replacement?
- Should Python bindings expose typed dataclasses / enums for risk policies for improved DX?
- How to stage changes without breaking existing JSON configurations.
