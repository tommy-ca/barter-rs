# Cross-Language Maintenance Workflow

Last updated: 2025-10-03

## Goals
- Keep Rust and Python codepaths delivering equivalent engine capabilities.
- Ensure every new Rust API addition has a conscious binding decision (exported, deferred, or intentionally omitted).
- Maintain fast feedback via unified CI that exercises Rust unit tests, Python unit tests, and packaging smoke checks.

## Branching & Release Cadence
- `main` remains protected: all changes land via PR with CI green.
- Release branches cut monthly (or ad-hoc for urgent fixes) and tagged for both crates.
- Rust crates versioned per semantic expectations; Python package mirrors the engine major/minor version with independent patch cadence.

## Change Workflow Checklist
1. Open issue tracking the capability gap or maintenance task.
2. Capture requirements / spec snippet under `.agent/specs/` when the task introduces new surface area.
3. Follow TDD: add or adjust Rust + Python tests before implementation where practical.
4. Implement change in Rust crate(s), then add bindings in `barter-python` as needed.
5. Run `cargo test --workspace`, `pytest -q tests_py`, and `maturin develop && maturin build --release` locally where relevant.
6. Update docs (Rust crate README snippets, Python README, `.agent` notes).
7. Commit & push each atomic change (docs, specs, code) referencing linked issue.
8. Prepare release notes summarising cross-language impacts.

## CI Expectations
- Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`.
- Python: `pytest -q tests_py`, `ruff check python tests_py`, `maturin build --release --no-sdist` for wheel sanity.
- Workflows should fail fast; ensure caching for maturin and cargo to keep runtime manageable.

## Onboarding Notes
- Document environment setup in top-level README (Rust toolchain + Python requirements).
- Provide scripts under `scripts/` to run combined test suites (`scripts/test-all.sh`).
- Encourage contributors to run `just bootstrap` (to be added) for reproducible env setup.

