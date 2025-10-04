# Porting Plan (2025-10-03)

## Immediate Objectives
1. Review existing bindings and overall repository state (2025-10-03 ✅).
2. Outline desired Python API surface and prioritise components for the initial release (2025-10-03 ✅).
3. Implement Rust-to-Python wrapper modules and supporting ergonomics (2025-10-03 ✅).
4. Integrate packaging, build scripts, and CI wiring to publish Python artifacts (2025-10-03 ✅).
5. Add essential unit & end-to-end tests alongside updated documentation (initial integration
   suite implemented 2025-10-03; see `.agent/specs/python-integration-tests.md`) (2025-10-03 ✅).

## Completed (2025-10-04)
1. Establish cross-language maintenance workflow (Rust + Python) including branching strategy and release cadence.
2. Identify remaining Rust APIs requiring Python exposure (risk manager configuration ✅ 2025-10-04; portfolio analytics extensions — profit factor, win rate, rate of return — shipped 2025-10-04; initial balance seeding ✅ 2025-10-04).
3. Produce incremental TDD plan emphasising new bindings with paired Rust/Python coverage.
4. Align CI to run `cargo test`, `pytest`, and packaging checks on every push & PR (✅ 2025-10-04 via `.github/workflows/ci.yml`).
5. Prepare developer onboarding notes for maintaining the hybrid workspace (✅ 2025-10-04; see `docs/developer-onboarding.md`).

## Notes
- Maintain commit discipline with atomic changes (commit & push each step).
- Balance effort with ~80% focused on core porting work, ~20% on testing scaffolding.
- Use `.agent` directory for scratch notes and future TODOs.

## Prior Roadmap Snapshot
1. ✅ Audit existing Rust crates and current `barter-python` module.
2. ✅ Design binding architecture, build tooling, and packaging approach.
3. ✅ Implement core binding modules and integrate with Rust components.
4. ✅ Add Python packaging metadata plus unit and end-to-end tests (integration suite landed
   2025-10-03; automated wheel publishing wired up via `python-wheels` workflow).
5. 🔄 Refresh documentation, examples, and CI pipelines for the hybrid project (README python
   quickstart updated 2025-10-03; release notes section added 2025-10-04; further updates pending).
