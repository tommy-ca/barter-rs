from __future__ import annotations

import json
from pathlib import Path

import pytest


@pytest.mark.integration
def test_backtest_cli_outputs_summary(capsys: pytest.CaptureFixture[str]) -> None:
    from barter_python_cli import backtest

    config_path = Path(__file__).parents[2] / "barter" / "examples" / "config" / "system_config.json"
    data_path = (
        Path(__file__).parents[2]
        / "barter"
        / "examples"
        / "data"
        / "binance_spot_market_data_with_disconnect_events.json"
    )

    exit_code = backtest.main(
        [
            "--config",
            str(config_path),
            "--market-data",
            str(data_path),
            "--format",
            "json",
        ]
    )

    assert exit_code == 0

    captured = capsys.readouterr()
    payload = json.loads(captured.out)

    assert payload["instruments"]
    assert payload["assets"]


@pytest.mark.integration
def test_backtest_cli_interval_option(capsys: pytest.CaptureFixture[str]) -> None:
    from barter_python_cli import backtest

    config_path = Path(__file__).parents[2] / "barter" / "examples" / "config" / "system_config.json"
    data_path = (
        Path(__file__).parents[2]
        / "barter"
        / "examples"
        / "data"
        / "binance_spot_market_data_with_disconnect_events.json"
    )

    exit_code = backtest.main(
        [
            "--config",
            str(config_path),
            "--market-data",
            str(data_path),
            "--interval",
            "annual-365",
        ]
    )

    assert exit_code == 0

    payload = json.loads(capsys.readouterr().out)
    instrument_payload = next(iter(payload["instruments"].values()))
    assert instrument_payload["sharpe_ratio"]["interval"] == "Annual(365)"
