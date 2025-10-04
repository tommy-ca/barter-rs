# Developer Onboarding

Welcome to the Barter hybrid Rust ⟷ Python workspace. This guide captures the
minimum you need to get productive when maintaining the shared codebase.

## Prerequisites

- Install the latest stable Rust toolchain via [`rustup`](https://rustup.rs/).
- Install Python 3.11 (project tests target 3.11 in CI; 3.9–3.12 are supported).
- Install [maturin](https://github.com/PyO3/maturin) globally or be ready to
  invoke it through `pipx` / virtual environments.
- Install `pip`, `virtualenv`, and your preferred task runner (`just` support
  is planned; today we use the provided shell scripts).

## Bootstrapping the Repository

1. Clone the repository and enter the workspace.
2. Optionally create a virtual environment for Python tooling.
3. Install Python dependencies used by the bindings test suite:

   ```bash
   python -m pip install --upgrade pip
   python -m pip install maturin pytest
   ```

4. Verify Rust compilation succeeds before diving into changes:

   ```bash
   cargo check --workspace
   ```

5. Build the Python extension in development mode so `barter_python` is
   importable from your interpreter:

   ```bash
   (cd barter-python && maturin develop)
   ```

## Day-to-Day Workflow

- Follow TDD: write or adjust the relevant test before implementing code.
- Prefer end-to-end coverage in `barter-python/tests_py` where behaviour spans
  Rust and Python boundaries; complement with targeted unit tests as needed.
- Use the shared helper script to exercise the full suite prior to pushing:

  ```bash
  scripts/test-all.sh
  ```

  This script runs `cargo fmt`, `cargo clippy`, `cargo test --workspace`, and
  `pytest -q tests_py` with the extension built in release mode.

- When iterating locally you can focus on a subset of checks:

  ```bash
  cargo test --workspace
  (cd barter-python && pytest -q tests_py)
  (cd barter-python && maturin build --release --no-sdist)
  ```

- Keep commits atomic: stage, commit, and push after each isolated change. The
  repository provides a `local` remote (`.agent/local-remote.git`) so you can
  push even when working offline from GitHub.
- Document new requirements or interface changes by updating `.agent/specs/`
  alongside the implementation, then link the spec in your commit message or
  PR description.

## Continuous Integration Expectations

- Every push to `main` and pull request triggers the CI workflow that runs:
  - `cargo check`, `cargo test`, `cargo fmt`, and `cargo clippy` across the
    Rust workspace.
  - `pytest -q tests_py` with the extension built in release mode.
  - `maturin build --release --no-sdist` as a packaging smoke test.
- Keep CI fast by respecting existing caching (Rust builds are cached via
  `Swatinem/rust-cache` and maturin reuses artefacts under
  `barter-python/target`).

## Additional Resources

- `.agent/plan.md`: long-term roadmap snapshot for the porting effort.
- `.agent/specs/`: individual requirement documents for recent features.
- `barter-python/README.md`: Python-focused quickstart and release process.
- `scripts/test-all.sh`: combined Rust + Python validation shortcut.

Reach out in Discord or open a GitHub discussion if you discover workflows that
could be automated or clarified further.
