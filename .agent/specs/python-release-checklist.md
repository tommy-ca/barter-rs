# Python Release Checklist

**Last Updated:** 2025-10-03

This checklist captures the repeatable steps required to cut and publish a
`barter-python` release once the project owners are ready to distribute new
bindings to PyPI. It assumes the GitHub Actions workflows described in
`python-packaging-automation.md` are available and that a `PYPI_API_TOKEN`
secret has been configured on the repository.

## Pre-Release
- Confirm `main` is green: CI (Rust + Python) has passed for the target commit.
- Review changelog entries and update `barter-python/README.md` if user-facing
  behaviour changed.
- Bump versions:
  - Update `version` in `barter-python/Cargo.toml` and `barter-python/pyproject.toml`.
  - Run `cargo metadata` to ensure the workspace resolves cleanly.
- Regenerate the lockfile if new dependencies were added: `cargo update -p <crate>`
  as needed, then commit the refreshed `Cargo.lock`.
- Run local smoke tests:
  - `cargo test -p barter-python --features python-tests`
  - `maturin develop --release` followed by `pytest -q tests_py`
- Draft release notes summarising key changes for the version tag.

## Tagging & Build
- Create an annotated tag on the release commit: `git tag -a v<MAJOR.MINOR.PATCH>`.
- Push the tag to origin: `git push origin v<MAJOR.MINOR.PATCH>`.
- Monitor the `Build Python Wheels` workflow:
  - Confirm the matrix builds succeed for all target OS/Python combinations.
  - Ensure artifacts (`*.whl` files) are uploaded for each job.
- If necessary, trigger `workflow_dispatch` reruns for flaky jobs before
  publishing.

## Publish to PyPI
- Verify the `Publish to PyPI` job runs (requires tag + `PYPI_API_TOKEN`).
- Check the job output for successful uploads via `maturin publish`.
- Inspect the PyPI project page to confirm the new version appears and files
  are listed for each platform.

## Post-Release
- Merge any hotfix PRs created during the release back into `main`.
- Update `.agent/todo.md` with follow-up tasks (for example, marketing or
  documentation updates).
- Close or update GitHub issues linked to the release.
- Announce availability in the preferred communication channels (Discord,
  mailing list, etc.).
