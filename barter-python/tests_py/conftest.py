"""Pytest configuration for the `barter_python` bindings tests.

This module ensures the native extension is built for the active interpreter
before any tests execute and exposes shared fixtures such as the repository
root path.
"""

from __future__ import annotations

import importlib
import os
import subprocess
from pathlib import Path
from typing import Iterator

import pytest


PACKAGE_ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = PACKAGE_ROOT.parent


def _build_extension() -> None:
    cmd = ["maturin", "develop"]

    if os.environ.get("BARTER_PYTHON_BUILD_RELEASE") == "1":
        cmd.append("--release")

    subprocess.run(cmd, check=True, cwd=PACKAGE_ROOT)


def _ensure_extension_installed() -> None:
    try:
        importlib.import_module("barter_python")
        return
    except ModuleNotFoundError:
        pass

    if os.environ.get("BARTER_PYTHON_SKIP_BUILD") == "1":
        raise RuntimeError(
            "barter_python extension not found and build skipped via environment flag"
        )

    _build_extension()

    # Refresh module discovery caches before importing again.
    importlib.invalidate_caches()
    importlib.import_module("barter_python")


_ensure_extension_installed()


@pytest.fixture(scope="session")
def repo_root() -> Iterator[Path]:
    yield REPO_ROOT


@pytest.fixture(scope="session")
def example_paths(repo_root: Path) -> dict[str, Path]:
    examples = repo_root / "barter" / "examples"
    return {
        "system_config": examples / "config" / "system_config.json",
        "market_data": examples
        / "data"
        / "binance_spot_market_data_with_disconnect_events.json",
    }
