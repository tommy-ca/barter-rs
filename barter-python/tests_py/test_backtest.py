"""Tests for the Rust-backed backtest bindings."""

from __future__ import annotations

import asyncio
from datetime import datetime
from decimal import Decimal

import pytest

from barter_python import SystemConfig, backtest
from barter_python.data import DataKind, MarketEvent, PublicTrade
from barter_python.instrument import Side


class TestMarketDataInMemory:
    """Validate the Python wrapper around Rust MarketDataInMemory."""

    def test_from_json_file(self, repo_root):
        """Loading market data from JSON yields populated events and metadata."""
        json_path = (
            repo_root
            / "barter-python"
            / "tests_py"
            / "data"
            / "synthetic_market_data.json"
        )
        market_data = backtest.MarketDataInMemory.from_json_file(json_path)

        assert len(market_data.events) > 0
        assert isinstance(market_data._time_first_event, datetime)
        assert market_data._inner is not None

        first_event = market_data.events[0]
        assert isinstance(first_event, MarketEvent)
        assert first_event.kind.kind == "trade"
        assert market_data._inner.time_first_event == market_data._time_first_event

    def test_stream_iteration(self):
        """Streaming over events surfaces the backing sequence."""
        event = MarketEvent(
            time_exchange=datetime(2025, 1, 1, 0, 0, 0),
            time_received=datetime(2025, 1, 1, 0, 0, 1),
            exchange="binance",
            instrument=0,
            kind=DataKind.trade(
                PublicTrade(id="1", price=50000.0, amount=1.0, side=Side.BUY)
            ),
        )
        market_data = backtest.MarketDataInMemory(
            _time_first_event=event.time_exchange,
            events=[event],
        )

        async def collect_events() -> list[MarketEvent[int, DataKind]]:
            collected: list[MarketEvent[int, DataKind]] = []
            async for item in market_data.stream():
                collected.append(item)
            return collected

        collected = asyncio.run(collect_events())
        assert collected == [event]

    def test_time_first_event_async(self):
        """The async helper returns the cached first event timestamp."""
        first_time = datetime(2024, 12, 31, 23, 59, 59)
        market_data = backtest.MarketDataInMemory(
            _time_first_event=first_time,
            events=[],
        )

        async def fetch_time() -> datetime:
            return await market_data.time_first_event()

        assert asyncio.run(fetch_time()) == first_time


class TestBacktestArgs:
    """Exercise constant and dynamic backtest argument wrappers."""

    def test_backtest_args_constant_metadata(self, example_paths):
        system_config = SystemConfig.from_json(str(example_paths["system_config"]))
        market_data = backtest.MarketDataInMemory.from_json_file(
            example_paths["market_data"]
        )

        args = backtest.BacktestArgsConstant(
            system_config=system_config,
            market_data=market_data,
            summary_interval="annual_365",
        )

        assert args.instrument_count > 0
        assert args.execution_count > 0
        assert args.summary_interval == "annual_365"
        assert args.market_data is market_data

    def test_backtest_args_constant_rejects_interval(self, example_paths):
        system_config = SystemConfig.from_json(str(example_paths["system_config"]))
        market_data = backtest.MarketDataInMemory.from_json_file(
            example_paths["market_data"]
        )

        with pytest.raises(ValueError):
            backtest.BacktestArgsConstant(
                system_config=system_config,
                market_data=market_data,
                summary_interval="fortnightly",
            )

    def test_backtest_args_dynamic_validation(self):
        args = backtest.BacktestArgsDynamic(
            id="baseline",
            risk_free_return=Decimal("0.02"),
        )

        assert args.id == "baseline"
        assert args.risk_free_return == Decimal("0.02")
        assert args.strategy is None
        assert args.risk is None

        with pytest.raises(ValueError):
            backtest.BacktestArgsDynamic(id="  ", risk_free_return=Decimal("0.01"))


class TestBacktestExecution:
    """End-to-end tests for the synchronous backtest wrappers."""

    def _build_args(self, example_paths, *, interval: str = None):
        system_config = SystemConfig.from_json(str(example_paths["system_config"]))
        market_data = backtest.MarketDataInMemory.from_json_file(
            example_paths["market_data"]
        )

        args_constant = backtest.BacktestArgsConstant(
            system_config=system_config,
            market_data=market_data,
            summary_interval=interval,
        )
        return args_constant

    def test_backtest_produces_summary(self, example_paths):
        args_constant = self._build_args(example_paths)
        args_dynamic = backtest.BacktestArgsDynamic(
            id="baseline",
            risk_free_return=Decimal("0.03"),
        )

        summary = backtest.backtest(args_constant, args_dynamic)

        assert summary.id == "baseline"
        assert summary.risk_free_return == Decimal("0.03")

        trading_summary = summary.trading_summary
        assert trading_summary.time_engine_start <= trading_summary.time_engine_end

        instruments = trading_summary.instruments
        assert isinstance(instruments, dict)
        assert instruments

        summary_dict = summary.to_dict()
        assert summary_dict["id"] == "baseline"
        assert Decimal(summary_dict["risk_free_return"]) == Decimal("0.03")

    def test_run_backtests_aggregates_results(self, example_paths):
        args_constant = self._build_args(example_paths, interval="annual_252")
        dynamics = [
            backtest.BacktestArgsDynamic(
                id="baseline",
                risk_free_return=Decimal("0.01"),
            ),
            backtest.BacktestArgsDynamic(
                id="alt",
                risk_free_return=Decimal("0.02"),
            ),
        ]

        multi = backtest.run_backtests(args_constant, dynamics)

        assert multi.num_backtests == 2
        assert multi.duration_ms >= 0

        summaries = multi.summaries
        assert len(summaries) == 2
        assert {summary.id for summary in summaries} == {"baseline", "alt"}

