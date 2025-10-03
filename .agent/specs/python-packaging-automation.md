# Python Packaging Automation

**Last Updated:** 2025-10-03

## Current State
- `barter-python` builds via `maturin develop` during local development and
  pytest runs (`tests_py/conftest.py`).
- CI runs a `python-tests` job that performs a `maturin develop --release`
  build followed by `pytest -m integration`.
- Release automation (`release-plz`) currently targets the Rust crates only;
  no workflow builds or publishes Python wheels.
- There is no multi-platform matrix for Python builds (Linux/macOS/Windows) or
  Python version coverage beyond 3.11 in CI.
- `.github/workflows/python-wheels.yml` builds wheels for CPython 3.9–3.12 across
  Linux, macOS, and Windows on pushes to `main`, `v*` tags, or manual dispatches. Tag runs with a
  configured `PYPI_API_TOKEN` now publish all collected wheels to PyPI automatically.

## Goals
- Automate building wheels (`maturin build`) for CPython 3.9–3.12 across all
  three major platforms.
- Upload built wheels as workflow artifacts for validation and release
  packaging.
- Publish wheels to PyPI (or an internal index) when releases are tagged once
  credentials are configured.
- Keep automation lightweight and aligned with existing release-plz cadence.

## Proposed Workflow
1. **Build Stage**
   - Trigger on pushes to `main` and release tags.
   - Matrix over `os: [ubuntu-latest, macos-13, windows-latest]` and
     `python-version: ["3.9", "3.10", "3.11", "3.12"]`.
   - Install maturin, run `maturin build --release --strip` using the matching
     interpreter, and upload wheels via `actions/upload-artifact`.
2. **Publish Stage**
   - Conditional on tagged releases and availability of `PYPI_API_TOKEN`.
   - Download artifacts, run `maturin publish` (or `twine upload`) once across
     all wheels.
3. **Verification**
   - Optionally execute a smoke test that installs the wheel into a fresh
     venv and exercises `barter_python.__version__` and `shutdown_event()`.

## Open Questions
- Should release-plz drive tagging for Python artifacts or should a dedicated
  workflow watch for Rust crate releases?
- Are nightly wheels required for downstream testers? (Out of scope for MVP.)
- Will Windows builds require the MSVC toolchain in CI or can wheels target
  `x86_64-pc-windows-msvc` only?

## Next Actions
- [x] Draft GitHub Actions workflow implementing the build + artifact capture.
- [x] Decide on publish trigger semantics (manual vs. auto on tags).
- [x] Document packaging workflow in `barter-python/README.md` once automated
      release flow is active.
- [ ] Capture a release checklist once PyPI credentials are configured and a tagged publish is executed.
