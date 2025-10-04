# Cross-Language Release Cadence

**Last Updated:** 2025-10-04

## Purpose
- Align release timing for the Rust crates (`barter`, `barter-data`, etc.) and the
  Python bindings (`barter-python`).
- Keep published versions coherent for downstream users who mix Rust and Python
  integrations in the same trading stack.
- Provide a repeatable workflow that meshes with the existing `release-plz`
  automation without introducing redundant manual steps.

## Cadence Overview
- **Primary Driver:** the `barter` crate release train. Any release that bumps
  `barter` (minor or patch) starts a coordinated Python release.
- **Release Rhythm:** target the **first Wednesday of each month** for routine
  releases. Ship sooner if critical fixes land (`priority` label) or whenever a
  `release-plz` PR merges with user-facing changes.
- **Time Budget:** publish the companion Python wheels within **48 hours** of the
  Rust crates hitting `crates.io`.
- **Hotfixes:** treat hotfixes as ad-hoc releases. Patch the Rust crate first,
  then cut a `.postN` Python release aligned to the same Rust version number.

## Versioning Policy
- Set the Python package version to match the released `barter` crate version:
  `barter-python == <barter version>`. Example: Rust release `barter 0.9.3`
  implies `barter-python 0.9.3`.
- If a Python-only fix is required after synchronising with Rust, increment the
  Python version using `post` releases (PEP 440) e.g. `0.9.3.post1`; no Rust
  tag change is required.
- When `barter` publishes a new minor version (e.g. `0.10.0`), bump every
  dependent crate and `barter-python` to the same `0.10.0` version before
  tagging.
- Keep `Cargo.toml` and `pyproject.toml` versions in sync. Update both within the
  same commit to maintain atomicity.

## End-to-End Workflow
1. **Prep Window (T-3 days)**
   - Review merged pull requests and changelog fragments.
   - Ensure CI is green on `main` (Rust + Python).
2. **Release-Plz Run (T-2 days)**
   - Trigger `release-plz` to bump Rust crate versions. Review and merge the
     generated PR.
   - Capture the new `barter` version (e.g. `0.9.3`).
3. **Python Alignment (T-1 day)**
   - Update `barter-python/Cargo.toml` & `pyproject.toml` to `0.9.3`.
   - Regenerate `Cargo.lock` if dependency versions changed.
   - Run smoke tests: `cargo test -p barter-python --features python-tests` and
     `pytest -q tests_py` after `maturin develop --release`.
   - Commit with message `align: release 0.9.3` and push to `main`.
4. **Tag & Publish (Release Day)**
   - Create annotated tag `v0.9.3` pointing at the aligned commit.
   - Push tag; monitor `python-wheels` workflow for build + publish success.
   - Verify crates.io and PyPI listings.
5. **Post-Release (T+1 day)**
   - Update `.agent/todo.md` with any follow-ups.
   - Announce in Discord and update documentation snippets as needed.

## Hotfix Playbook
- If critical issues surface after release:
  1. Apply fixes to Rust crates; run `release-plz` to issue a patch (e.g. `0.9.4`).
  2. Update Python versions to `0.9.4` in the same commit.
  3. Tag and publish as usual within 24 hours.
- For Python-specific bugs that do not require a Rust patch:
  1. Apply the Python fix; run tests.
  2. Bump only `pyproject.toml` & `Cargo.toml` to `0.9.3.post1`.
  3. Tag `v0.9.3.post1` and publish wheels. No crates.io release is needed.

## Changelog Coordination
- Maintain individual crate changelogs via `release-plz`.
- Aggregate key highlights (Rust + Python) in `barter-python/README.md` under a
  "Release Notes" section (TODO).
- For Python post releases, append a short note to the README and `.agent/todo.md`
  describing the fix and referencing the corresponding GitHub issue.

## Responsibilities
- **Release Captain (rotating):** initiates `release-plz`, coordinates reviews,
  and drives the schedule.
- **Python Maintainer:** handles version bumps, runs tests, and monitors the
  wheel build workflow.
- **Documentation Lead:** confirms README/website updates and publishes release
  announcements.

## Tooling Checklist
- `release-plz` (Rust version bumps)
- `python-wheels` workflow (wheel build + PyPI publication)
- `cargo-dist` (optional future integration for combined artifacts)
- `.agent/specs/python-release-checklist.md` for granular validation steps

