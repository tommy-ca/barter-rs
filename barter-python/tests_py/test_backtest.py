"""Unit tests for the backtest module."""

import asyncio
from datetime import datetime
from decimal import Decimal

from barter_python import backtest
from barter_python.data import Candle, DataKind, Liquidation, MarketEvent, PublicTrade
from barter_python.instrument import Asset, AssetNameExchange, AssetNameInternal, ExchangeId, Instrument, Side, Underlying
from barter_python.statistic import Annual365


class TestMarketDataInMemory:
    """Test MarketDataInMemory implementation."""

    def test_from_json_file(self, repo_root):
        """Test loading market data from JSON file."""
        json_path = repo_root / "barter-python" / "tests_py" / "data" / "synthetic_market_data.json"
        market_data = backtest.MarketDataInMemory.from_json_file(json_path)

        assert len(market_data.events) > 0
        assert isinstance(market_data._time_first_event, datetime)

    def test_stream(self):
        """Test streaming market events."""
        # Create test events
        events = [
            MarketEvent(
                time_exchange=datetime(2023, 1, 1, 12, 0, 0),
                time_received=datetime(2023, 1, 1, 12, 0, 1),
                exchange="binance",
                instrument=0,
                kind=DataKind.trade(PublicTrade(
                    id="1",
                    price=50000.0,
                    amount=1.0,
                    side=Side.BUY
                ))
            )
        ]

        market_data = backtest.MarketDataInMemory(
            _time_first_event=datetime(2023, 1, 1, 12, 0, 0),
            events=events
        )

        async def collect_events():
            collected = []
            async for event in market_data.stream():
                collected.append(event)
            return collected

        collected = asyncio.run(collect_events())
        assert len(collected) == 1
        assert collected[0] == events[0]

    def test_time_first_event(self):
        """Test getting first event time."""
        first_time = datetime(2023, 1, 1, 12, 0, 0)
        market_data = backtest.MarketDataInMemory(
            _time_first_event=first_time,
            events=[]
        )

        async def get_time():
            return await market_data.time_first_event()

        result = asyncio.run(get_time())
        assert result == first_time


class TestBacktestSummary:
    """Test BacktestSummary data structure."""

    def test_creation(self):
        """Test creating a BacktestSummary."""
        trading_summary = backtest.TradingSummary(
            time_engine_start=datetime(2023, 1, 1),
            time_engine_end=datetime(2023, 1, 2),
        )

        summary = backtest.BacktestSummary(
            id="test_backtest",
            risk_free_return=Decimal("0.02"),
            trading_summary=trading_summary
        )

        assert summary.id == "test_backtest"
        assert summary.risk_free_return == Decimal("0.02")
        assert summary.trading_summary == trading_summary


class TestMultiBacktestSummary:
    """Test MultiBacktestSummary data structure."""

    def test_creation(self):
        """Test creating a MultiBacktestSummary."""
        summaries = [
            backtest.BacktestSummary(
                id="test1",
                risk_free_return=Decimal("0.02"),
                trading_summary=backtest.TradingSummary(
                    time_engine_start=datetime(2023, 1, 1),
                    time_engine_end=datetime(2023, 1, 2),
                )
            )
        ]

        multi_summary = backtest.MultiBacktestSummary(
            total_duration=1.5,
            summaries=summaries
        )

        assert multi_summary.total_duration == 1.5
        assert len(multi_summary.summaries) == 1
        assert multi_summary.summaries[0] == summaries[0]


class TestBacktestArgs:
    """Test backtest argument structures."""

    def test_backtest_args_constant(self):
        """Test BacktestArgsConstant creation."""
        instruments = backtest.IndexedInstruments.new([])
        executions = [backtest.ExecutionConfig.mock(backtest.MockExecutionConfig())]
        market_data = backtest.MarketDataInMemory(
            _time_first_event=datetime(2023, 1, 1),
            events=[]
        )
        summary_interval = Annual365()
        engine_state = backtest.EngineState(instruments=[])

        args = backtest.BacktestArgsConstant(
            instruments=instruments,
            executions=executions,
            market_data=market_data,
            summary_interval=summary_interval,
            engine_state=engine_state
        )

        assert args.instruments == instruments
        assert len(args.executions) == 1
        assert args.market_data == market_data
        assert args.summary_interval == summary_interval
        assert args.engine_state == engine_state

    def test_backtest_args_dynamic(self):
        """Test BacktestArgsDynamic creation."""
        args = backtest.BacktestArgsDynamic(
            id="test",
            risk_free_return=Decimal("0.02"),
            strategy=None,  # Placeholder
            risk=None  # Placeholder
        )

        assert args.id == "test"
        assert args.risk_free_return == Decimal("0.02")


class TestIndexedInstruments:
    """Test IndexedInstruments."""

    def test_new_empty(self):
        """Test creating empty IndexedInstruments."""
        indexed = backtest.IndexedInstruments.new([])
        assert len(indexed.instruments()) == 0

    def test_new_with_instruments(self):
        """Test creating IndexedInstruments with instruments."""
        instrument = Instrument.spot(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btc_usdt",
            name_exchange="BTCUSDT",
            underlying=Underlying(
                base=Asset.new_from_exchange("btc"),
                quote=Asset.new_from_exchange("usdt")
            )
        )

        indexed = backtest.IndexedInstruments.new([instrument])
        assert len(indexed.instruments()) == 1


class TestExecutionConfig:
    """Test ExecutionConfig."""

    def test_mock_creation(self):
        """Test creating mock execution config."""
        mock_config = backtest.MockExecutionConfig()
        config = backtest.ExecutionConfig.mock(mock_config)
        assert config.mock_config == mock_config