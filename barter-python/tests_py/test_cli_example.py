"""CLI example tests for the `barter_python` package.

These tests exercise the example command-line interface script to ensure it can
load configuration and market data files, execute a backtest using the Python
bindings, and produce a structured trading summary for downstream consumption.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def _run_cli(script_path: Path, *args: str) -> subprocess.CompletedProcess[str]:
    command = [sys.executable, str(script_path), *args]
    return subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
    )


def test_backtest_cli_example(repo_root: Path, example_paths: dict[str, Path]) -> None:
    script = repo_root / "barter-python" / "examples" / "backtest_cli.py"

    assert script.exists(), "Example CLI script is missing"

    result = _run_cli(
        script,
        "--config",
        str(example_paths["system_config"]),
        "--market-data",
        str(example_paths["market_data"]),
        "--format",
        "json",
    )

    assert result.returncode == 0, result.stderr

    payload = json.loads(result.stdout)

    assert "time_engine_start" in payload
    assert "time_engine_end" in payload
    assert payload["instruments"]
    assert payload["assets"]

    instruments = set(payload["instruments"].keys())
    expected = {
        "binancespot-btc_usdt",
        "binancespot-eth_usdt",
        "binancespot-sol_usdt",
    }

    assert instruments == expected
