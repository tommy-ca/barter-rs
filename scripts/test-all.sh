#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "[test-all] Running cargo test --workspace --tests --lib"
(
  cd "$repo_root"
  cargo test --workspace --tests --lib
)

echo "[test-all] Building Python extension via maturin"
(
  cd "$repo_root/barter-python"
  maturin develop
)

echo "[test-all] Executing pytest"
(
  cd "$repo_root/barter-python"
  pytest -q tests_py
)

echo "[test-all] All checks passed"
