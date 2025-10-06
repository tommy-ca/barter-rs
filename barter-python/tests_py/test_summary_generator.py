from __future__ import annotations

from datetime import datetime, timedelta, timezone
from decimal import Decimal
from pathlib import Path

import barter_python as bp


def _load_config(example_paths: dict[str, Path]) -> bp.SystemConfig:
    return bp.SystemConfig.from_json(str(example_paths["system_config"]))


def test_backtest_returns_generator(example_paths: dict[str, Path]) -> None:
    """Ensure generator output matches legacy summary helper."""

    config = _load_config(example_paths)
    market_path = str(example_paths["market_data"])

    summary_only = bp.run_historic_backtest(
        config,
        market_path,
        risk_free_return=0.02,
        interval="annual_252",
    )

    summary_with_gen, generator = bp.run_historic_backtest_with_generator(
        config,
        market_path,
        risk_free_return=0.02,
        interval="annual_252",
    )

    summary_only_dict = summary_only.to_dict()
    summary_with_dict = summary_with_gen.to_dict()

    assert summary_with_dict["instruments"].keys() == summary_only_dict["instruments"].keys()
    assert summary_with_dict["assets"].keys() == summary_only_dict["assets"].keys()

    delta = summary_with_gen.time_engine_end - summary_only.time_engine_end
    assert abs(delta.total_seconds()) < 0.01

    first_instrument = next(iter(summary_only_dict["instruments"].keys()))
    assert (
        summary_with_dict["instruments"][first_instrument]["pnl"]
        == summary_only_dict["instruments"][first_instrument]["pnl"]
    )
    assert generator.risk_free_return == Decimal("0.02")

    annual_summary = generator.generate("annual_252")
    annual_dict = annual_summary.to_dict()
    assert annual_dict["instruments"].keys() == summary_only_dict["instruments"].keys()
    assert annual_dict["assets"].keys() == summary_only_dict["assets"].keys()
    annual_delta = annual_summary.time_engine_end - summary_only.time_engine_end
    assert abs(annual_delta.total_seconds()) < 0.01


def test_generator_balance_update_increments_time(example_paths: dict[str, Path]) -> None:
    """Appending balances to the generator should advance summary time."""

    config = _load_config(example_paths)
    market_path = str(example_paths["market_data"])

    summary, generator = bp.run_historic_backtest_with_generator(
        config,
        market_path,
        risk_free_return=0.015,
    )

    base_summary = summary.to_dict()

    # Determine a known asset index using the execution instrument map
    instrument_map = bp.ExecutionInstrumentMap.from_system_config(
        bp.ExchangeId.BINANCE_SPOT, config
    )
    asset_names = instrument_map.asset_names()
    assert asset_names, "expected assets in execution instrument map"
    asset_index = instrument_map.asset_index(asset_names[0])

    balance = bp.Balance.new(Decimal("1000"), Decimal("975"))
    update_time = summary.time_engine_end + timedelta(hours=1)
    update_time = update_time.replace(tzinfo=timezone.utc)

    asset_balance = bp.AssetBalance.new(asset_index, balance, update_time)
    generator.update_from_balance(asset_balance)
    generator.update_time_now(update_time)

    refreshed = generator.generate()
    assert refreshed.time_engine_end == update_time

    refreshed_dict = refreshed.to_dict()
    assert "assets" in refreshed_dict
    assert refreshed_dict["assets"]

    # Ensure previous metrics are still accessible for comparison
    assert base_summary["instruments"].keys() == refreshed_dict["instruments"].keys()


def test_generator_supports_time_scaling(example_paths: dict[str, Path]) -> None:
    """Generator supports interval labels consistent with existing helpers."""

    config = _load_config(example_paths)
    market_path = str(example_paths["market_data"])

    _, generator = bp.run_historic_backtest_with_generator(
        config,
        market_path,
        risk_free_return=0.0,
    )

    daily = generator.generate()
    annual_365 = generator.generate("annual_365")

    assert daily.time_engine_end <= datetime.now(timezone.utc)
    assert annual_365.to_dict()["instruments"]
