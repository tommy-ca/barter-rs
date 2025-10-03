# Porting Plan (2025-10-03)

## Immediate Objectives
1. Review existing bindings and overall repository state.
2. Outline desired Python API surface and prioritise components for the initial release.
3. Implement Rust-to-Python wrapper modules and supporting ergonomics.
4. Integrate packaging, build scripts, and CI wiring to publish Python artifacts.
5. Add essential unit & end-to-end tests alongside updated documentation (spec captured at
   `.agent/specs/python-integration-tests.md`).

## Notes
- Maintain commit discipline with atomic changes (commit & push each step).
- Balance effort with ~80% focused on core porting work, ~20% on testing scaffolding.
- Use `.agent` directory for scratch notes and future TODOs.

## Prior Roadmap Snapshot
1. ✅ Audit existing Rust crates and current `barter-python` module.
2. ✅ Design binding architecture, build tooling, and packaging approach.
3. ✅ Implement core binding modules and integrate with Rust components.
4. 🚧 Add Python packaging metadata plus unit and end-to-end tests (in progress 2025-10-03).
5. ⏳ Refresh documentation, examples, and CI pipelines for the hybrid project.
