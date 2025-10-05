"""Integration tests for barter-python end-to-end examples using TDD approach."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

import barter_python as bp


@pytest.mark.integration
def test_comprehensive_backtest_example(repo_root: Path, example_paths: dict[str, Path]) -> None:
    """TDD test: comprehensive backtest example should execute successfully and produce valid results."""
    example_script = repo_root / "barter-python" / "examples" / "comprehensive_backtest_example.py"

    # Run the example script
    result = subprocess.run(
        [sys.executable, str(example_script)],
        cwd=repo_root / "barter-python",
        capture_output=True,
        text=True,
        timeout=30
    )

    # Should exit successfully
    assert result.returncode == 0, f"Example failed: {result.stderr}"

    # Should print expected output
    assert "Barter Python - Comprehensive Backtest Example" in result.stdout
    assert "Loaded system configuration" in result.stdout
    assert "Backtest Comparison Results:" in result.stdout
    assert "Detailed results saved" in result.stdout

    # Should create result files
    results_dir = repo_root / "barter-python"
    result_files = [
        results_dir / "backtest_results_daily.json",
        results_dir / "backtest_results_annual_252.json",
        results_dir / "backtest_results_annual_365.json"
    ]

    for result_file in result_files:
        assert result_file.exists(), f"Result file {result_file} not created"

        # Load and validate JSON structure
        with open(result_file) as f:
            data = json.load(f)

        assert "time_engine_start" in data
        assert "time_engine_end" in data
        assert "instruments" in data
        assert "assets" in data

    # Clean up created files
    for result_file in result_files:
        result_file.unlink(missing_ok=True)


@pytest.mark.integration
def test_live_system_simulation_example(repo_root: Path) -> None:
    """TDD test: live system simulation should start, process events, and shutdown cleanly."""
    example_script = repo_root / "barter-python" / "examples" / "live_system_simulation.py"

    # Run the example script
    result = subprocess.run(
        [sys.executable, str(example_script)],
        cwd=repo_root / "barter-python",
        capture_output=True,
        text=True,
        timeout=60  # Allow more time for system startup/shutdown
    )

    # Should exit successfully
    assert result.returncode == 0, f"Example failed: {result.stderr}"

    # Should print expected output
    assert "Barter Python - Live System Simulation Example" in result.stdout
    assert "Started system with audit streaming" in result.stdout
    assert "Trading enabled" in result.stdout
    assert "System shutdown complete" in result.stdout
    assert "Summary saved" in result.stdout

    # Should create summary file
    summary_file = repo_root / "barter-python" / "live_simulation_summary.json"
    assert summary_file.exists(), "Summary file not created"

    # Load and validate summary structure
    with open(summary_file) as f:
        data = json.load(f)

    assert "time_engine_start" in data
    assert "time_engine_end" in data
    assert "instruments" in data
    assert "assets" in data

    # Clean up
    summary_file.unlink(missing_ok=True)


@pytest.mark.integration
def test_multi_exchange_backtest_example(repo_root: Path) -> None:
    """TDD test: multi-exchange example should configure exchanges correctly."""
    example_script = repo_root / "barter-python" / "examples" / "multi_exchange_backtest.py"

    # Run the example script
    result = subprocess.run(
        [sys.executable, str(example_script)],
        cwd=repo_root / "barter-python",
        capture_output=True,
        text=True,
        timeout=30
    )

    # Should exit successfully
    assert result.returncode == 0, f"Example failed: {result.stderr}"

    # Should print expected output
    assert "Barter Python - Multi-Exchange Backtest Example" in result.stdout
    assert "Loaded base configuration" in result.stdout
    assert "Would add Coinbase BTC instrument" in result.stdout
    assert "Multi-exchange config setup complete" in result.stdout


@pytest.mark.integration
def test_risk_management_example(repo_root: Path, example_paths: dict[str, Path]) -> None:
    """TDD test: risk management example should configure and persist risk limits."""
    example_script = repo_root / "barter-python" / "examples" / "risk_management_example.py"

    # Run the example script
    result = subprocess.run(
        [sys.executable, str(example_script)],
        cwd=repo_root / "barter-python",
        capture_output=True,
        text=True,
        timeout=30
    )

    # Should exit successfully
    assert result.returncode == 0, f"Example failed: {result.stderr}"

    # Should print expected output
    assert "Barter Python - Risk Management Example" in result.stdout
    assert "Set global risk limits" in result.stdout
    assert "Set per-instrument limits" in result.stdout
    assert "Saved updated config" in result.stdout
    assert "Risk management integration example complete" in result.stdout

    # Should create config file
    config_file = repo_root / "barter-python" / "config_with_risk.json"
    assert config_file.exists(), "Config file not created"

    # Load and validate config has risk settings
    config = bp.SystemConfig.from_json(str(config_file))

    # Check risk limits were set
    global_limits = config.risk_limits()["global"]
    assert global_limits is not None
    assert "max_leverage" in global_limits
    assert "max_position_notional" in global_limits

    # Clean up
    config_file.unlink(missing_ok=True)


@pytest.mark.integration
def test_order_lifecycle_example(repo_root: Path) -> None:
    """TDD test: order lifecycle example should create and manipulate orders correctly."""
    example_script = repo_root / "barter-python" / "examples" / "order_lifecycle_example.py"

    # Run the example script
    result = subprocess.run(
        [sys.executable, str(example_script)],
        cwd=repo_root / "barter-python",
        capture_output=True,
        text=True,
        timeout=30
    )

    # Should exit successfully
    assert result.returncode == 0, f"Example failed: {result.stderr}"

    # Should print expected output
    assert "Barter Python - Order Lifecycle Example" in result.stdout
    assert "Created order key:" in result.stdout
    assert "Created open request:" in result.stdout
    assert "Created order snapshot" in result.stdout
    assert "Created order event" in result.stdout
    assert "Created cancel request" in result.stdout
    assert "Created cancel event" in result.stdout
    assert "Opened order via mock client:" in result.stdout
    assert "Order lifecycle example complete" in result.stdout
