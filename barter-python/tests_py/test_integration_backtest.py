from __future__ import annotations

from decimal import Decimal
from pathlib import Path
from typing import Callable

import pytest

import barter_python as bp
from barter_python.backtest import MarketDataInMemory, BacktestArgsConstant, BacktestArgsDynamic
from barter_python.statistic import Daily


@pytest.mark.integration
def test_historic_backtest_summary(
    example_paths: dict[str, Path],
    tracing_log_capture: Callable[[], str],
) -> None:
    """Ensure historic backtests produce deterministic trading summaries."""

    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    market_data_path = str(example_paths["market_data"])

    summary = bp.run_historic_backtest(
        config,
        market_data_path,
        risk_free_return=0.01,
    )

    assert isinstance(summary, bp.TradingSummary)
    assert summary.time_engine_start <= summary.time_engine_end

    instruments = summary.instruments
    assert instruments

    instrument_name, tear_sheet = next(iter(instruments.items()))
    assert tear_sheet.pnl == Decimal("0")
    assert tear_sheet.pnl_return.value == Decimal("0")
    assert tear_sheet.pnl_return.interval == "Daily"
    assert tear_sheet.sharpe_ratio.interval == "Daily"
    assert tear_sheet.sortino_ratio.interval == "Daily"
    assert tear_sheet.calmar_ratio.interval == "Daily"

    summary_annual = bp.run_historic_backtest(
        config,
        market_data_path,
        risk_free_return=0.01,
        interval="annual_252",
    )

    annual_sheet = next(iter(summary_annual.instruments.values()))
    assert annual_sheet.pnl_return.interval == "Annual(252)"
    assert annual_sheet.sharpe_ratio.interval == "Annual(252)"
    assert annual_sheet.sortino_ratio.interval == "Annual(252)"
    assert annual_sheet.calmar_ratio.interval == "Annual(252)"

    with pytest.raises(ValueError):
        bp.run_historic_backtest(
            config,
            market_data_path,
            interval="unknown",
        )

    assets = summary.assets
    assert assets

    asset_name, asset_sheet = next(iter(assets.items()))
    assert isinstance(asset_name, str)

    balance_end = asset_sheet.balance_end
    assert balance_end is not None
    assert balance_end.total == Decimal("0.1")
    assert balance_end.free == Decimal("0.1")

    summary_dict = summary.to_dict()
    assert instrument_name in summary_dict["instruments"]
    assert asset_name in summary_dict["assets"]

    instrument_dict = summary_dict["instruments"][instrument_name]
    assert instrument_dict["pnl"] == 0
    assert instrument_dict["pnl_return"]["value"] == Decimal("0")

    logs = tracing_log_capture()
    assert "sending historical event to Engine" in logs


@pytest.mark.asyncio
async def test_market_data_in_memory_from_json(example_paths: dict[str, Path]) -> None:
    """Test loading market data from JSON file."""
    market_data_path = example_paths["market_data"]
    market_data = MarketDataInMemory.from_json_file(market_data_path)

    assert len(market_data.events) > 0

    first_time = await market_data.time_first_event()
    assert isinstance(first_time, bp.datetime)

    # Test streaming
    events = []
    async for event in market_data.stream():
        events.append(event)

    assert len(events) == len(market_data.events)
    assert events[0].time_exchange == first_time


@pytest.mark.asyncio
async def test_backtest_basic() -> None:
    """Test basic backtest functionality with minimal setup."""
    # This is a placeholder test - full implementation would require more setup
    pass
