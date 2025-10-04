"""Unit tests for the backtest module."""

import asyncio
from datetime import datetime, timedelta
from decimal import Decimal

from barter_python import backtest
from barter_python.data import DataKind, MarketEvent, PublicTrade
from barter_python.instrument import (
    Asset,
    AssetIndex,
    AssetNameInternal,
    ExchangeId,
    ExchangeIndex,
    Instrument,
    InstrumentIndex,
    InstrumentNameInternal,
    Keyed,
    Side,
    Underlying,
)
from barter_python.statistic import Annual365


class TestMarketDataInMemory:
    """Test MarketDataInMemory implementation."""

    def test_from_json_file(self, repo_root):
        """Test loading market data from JSON file."""
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

    def test_stream(self):
        """Test streaming market events."""
        # Create test events
        events = [
            MarketEvent(
                time_exchange=datetime(2023, 1, 1, 12, 0, 0),
                time_received=datetime(2023, 1, 1, 12, 0, 1),
                exchange="binance",
                instrument=0,
                kind=DataKind.trade(
                    PublicTrade(id="1", price=50000.0, amount=1.0, side=Side.BUY)
                ),
            )
        ]

        market_data = backtest.MarketDataInMemory(
            _time_first_event=datetime(2023, 1, 1, 12, 0, 0), events=events
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
            _time_first_event=first_time, events=[]
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
            instruments={},
            assets={},
        )

        summary = backtest.BacktestSummary(
            id="test_backtest",
            risk_free_return=Decimal("0.02"),
            trading_summary=trading_summary,
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
                ),
            )
        ]

        multi_summary = backtest.MultiBacktestSummary(
            total_duration=1.5, summaries=summaries
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
            _time_first_event=datetime(2023, 1, 1), events=[]
        )
        summary_interval = Annual365()
        engine_state = backtest.EngineEngineState()

        args = backtest.BacktestArgsConstant(
            instruments=instruments,
            executions=executions,
            market_data=market_data,
            summary_interval=summary_interval,
            engine_state=engine_state,
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
            risk=None,  # Placeholder
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
                quote=Asset.new_from_exchange("usdt"),
            ),
        )

        indexed = backtest.IndexedInstruments.new([instrument])
        assert len(indexed.instruments()) == 1

    def test_lookup_helpers(self):
        """Ensure exchange, asset, and instrument lookups operate with indices."""
        instrument = Instrument.spot(
            exchange=ExchangeId.GATEIO_SPOT,
            name_internal="gateio_spot-eth_usdt",
            name_exchange="ETH_USDT",
            underlying=Underlying(
                base=Asset.new_from_exchange("eth"),
                quote=Asset.new_from_exchange("usdt"),
            ),
        )

        indexed = backtest.IndexedInstruments.new([instrument])

        exchanges = indexed.exchanges()
        assert len(exchanges) == 1
        keyed_exchange = exchanges[0]
        assert isinstance(keyed_exchange, Keyed)
        assert isinstance(keyed_exchange.key, ExchangeIndex)
        assert keyed_exchange.value == ExchangeId.GATEIO_SPOT

        assets = indexed.assets()
        assert len(assets) == 2
        base_asset_entry, quote_asset_entry = assets
        assert isinstance(base_asset_entry.key, AssetIndex)
        assert isinstance(quote_asset_entry.key, AssetIndex)

        base_index = indexed.find_asset_index(
            instrument.exchange,
            instrument.underlying.base.name_internal,
        )
        quote_index = indexed.find_asset_index(
            instrument.exchange,
            instrument.underlying.quote.name_internal,
        )
        assert base_index == base_asset_entry.key
        assert quote_index == quote_asset_entry.key
        assert indexed.find_asset(base_index).asset.name_internal == AssetNameInternal(
            "eth"
        )
        assert indexed.find_asset(quote_index).asset.name_internal == AssetNameInternal(
            "usdt"
        )

        keyed_instruments = indexed.instruments()
        assert len(keyed_instruments) == 1
        keyed_instrument = keyed_instruments[0]
        assert isinstance(keyed_instrument.key, InstrumentIndex)
        assert keyed_instrument.value.name_internal == InstrumentNameInternal(
            "gateio_spot-eth_usdt"
        )

        instrument_index = indexed.find_instrument_index(
            instrument.exchange,
            instrument.name_internal,
        )
        assert instrument_index == keyed_instrument.key
        indexed_instrument = indexed.find_instrument(instrument_index)
        assert indexed_instrument.exchange.key == keyed_exchange.key
        assert indexed_instrument.exchange.value == ExchangeId.GATEIO_SPOT
        assert indexed_instrument.underlying.base == base_index
        assert indexed_instrument.underlying.quote == quote_index

    def test_builder_round_trip(self):
        """Verify the builder deduplicates exchanges and assets."""
        instruments = [
            Instrument.spot(
                exchange=ExchangeId.BINANCE_SPOT,
                name_internal="binance_spot-btc_usdt",
                name_exchange="BTCUSDT",
                underlying=Underlying(
                    base=Asset.new_from_exchange("btc"),
                    quote=Asset.new_from_exchange("usdt"),
                ),
            ),
            Instrument.spot(
                exchange=ExchangeId.BINANCE_SPOT,
                name_internal="binance_spot-eth_usdt",
                name_exchange="ETHUSDT",
                underlying=Underlying(
                    base=Asset.new_from_exchange("eth"),
                    quote=Asset.new_from_exchange("usdt"),
                ),
            ),
        ]

        builder = backtest.IndexedInstruments.builder()
        for inst in instruments:
            builder.add_instrument(inst)
        indexed = builder.build()

        assert len(indexed.exchanges()) == 1
        assert len(indexed.assets()) == 3
        assert len(indexed.instruments()) == 2


class TestExecutionConfig:
    """Test ExecutionConfig."""

    def test_mock_creation(self):
        """Test creating mock execution config."""
        mock_config = backtest.MockExecutionConfig()
        config = backtest.ExecutionConfig.mock(mock_config)
        assert config.mock_config == mock_config


class TestBalance:
    """Test Balance structure."""

    def test_creation(self):
        """Test creating a Balance."""
        balance = backtest.Balance(total=Decimal("1000"), free=Decimal("800"))
        assert balance.total == Decimal("1000")
        assert balance.free == Decimal("800")
        assert balance.used == Decimal("200")


class TestAssetBalance:
    """Test AssetBalance structure."""

    def test_creation(self):
        """Test creating an AssetBalance."""
        balance = backtest.Balance(total=Decimal("1000"), free=Decimal("800"))
        asset_balance = backtest.AssetBalance(
            asset="BTC", balance=balance, time_exchange=datetime(2023, 1, 1, 12, 0, 0)
        )
        assert asset_balance.asset == "BTC"
        assert asset_balance.balance == balance
        assert asset_balance.time_exchange == datetime(2023, 1, 1, 12, 0, 0)


class TestDrawdown:
    """Test Drawdown structure."""

    def test_creation(self):
        """Test creating a Drawdown."""
        start_time = datetime(2023, 1, 1, 12, 0, 0)
        end_time = datetime(2023, 1, 1, 13, 0, 0)
        drawdown = backtest.Drawdown(
            value=Decimal("-0.05"), time_start=start_time, time_end=end_time
        )
        assert drawdown.value == Decimal("-0.05")
        assert drawdown.time_start == start_time
        assert drawdown.time_end == end_time
        assert drawdown.duration == timedelta(hours=1)


class TestMeanDrawdown:
    """Test MeanDrawdown structure."""

    def test_creation(self):
        """Test creating a MeanDrawdown."""
        mean_drawdown = backtest.MeanDrawdown(
            mean_drawdown=Decimal("-0.03"),
            mean_drawdown_ms=3600000,  # 1 hour in milliseconds
        )
        assert mean_drawdown.mean_drawdown == Decimal("-0.03")
        assert mean_drawdown.mean_drawdown_ms == 3600000


class TestMaxDrawdown:
    """Test MaxDrawdown structure."""

    def test_creation(self):
        """Test creating a MaxDrawdown."""
        drawdown = backtest.Drawdown(
            value=Decimal("-0.1"),
            time_start=datetime(2023, 1, 1, 10, 0, 0),
            time_end=datetime(2023, 1, 1, 11, 0, 0),
        )
        max_drawdown = backtest.MaxDrawdown(drawdown)
        assert max_drawdown.drawdown == drawdown


class TestRange:
    """Test Range structure."""

    def test_creation(self):
        """Test creating a Range."""
        range_obj = backtest.Range(min=Decimal("0"), max=Decimal("100"))
        assert range_obj.min == Decimal("0")
        assert range_obj.max == Decimal("100")


class TestDispersion:
    """Test Dispersion structure."""

    def test_creation(self):
        """Test creating a Dispersion."""
        range_obj = backtest.Range(min=Decimal("0"), max=Decimal("100"))
        dispersion = backtest.Dispersion(
            range=range_obj,
            recurrence_relation_m=Decimal("50"),
            variance=Decimal("25"),
            std_dev=Decimal("5"),
        )
        assert dispersion.range == range_obj
        assert dispersion.recurrence_relation_m == Decimal("50")
        assert dispersion.variance == Decimal("25")
        assert dispersion.std_dev == Decimal("5")


class TestDataSetSummary:
    """Test DataSetSummary structure."""

    def test_creation(self):
        """Test creating a DataSetSummary."""
        range_obj = backtest.Range(min=Decimal("0"), max=Decimal("100"))
        dispersion = backtest.Dispersion(
            range=range_obj,
            recurrence_relation_m=Decimal("50"),
            variance=Decimal("25"),
            std_dev=Decimal("5"),
        )
        summary = backtest.DataSetSummary(
            count=Decimal("10"),
            sum=Decimal("500"),
            mean=Decimal("50"),
            dispersion=dispersion,
        )
        assert summary.count == Decimal("10")
        assert summary.sum == Decimal("500")
        assert summary.mean == Decimal("50")
        assert summary.dispersion == dispersion


class TestPnLReturns:
    """Test PnLReturns structure."""

    def test_creation(self):
        """Test creating PnLReturns."""
        range_obj = backtest.Range(min=Decimal("0"), max=Decimal("100"))
        dispersion = backtest.Dispersion(
            range=range_obj,
            recurrence_relation_m=Decimal("50"),
            variance=Decimal("25"),
            std_dev=Decimal("5"),
        )
        total_summary = backtest.DataSetSummary(
            count=Decimal("10"),
            sum=Decimal("500"),
            mean=Decimal("50"),
            dispersion=dispersion,
        )
        losses_summary = backtest.DataSetSummary(
            count=Decimal("3"),
            sum=Decimal("-100"),
            mean=Decimal("-33.33"),
            dispersion=dispersion,
        )
        pnl_returns = backtest.PnLReturns(
            pnl_raw=Decimal("400"), total=total_summary, losses=losses_summary
        )
        assert pnl_returns.pnl_raw == Decimal("400")
        assert pnl_returns.total == total_summary
        assert pnl_returns.losses == losses_summary


class TestTearSheet:
    """Test TearSheet structure."""

    def test_creation(self):
        """Test creating a TearSheet."""
        from barter_python.statistic import (
            Annual365,
            CalmarRatio,
            ProfitFactor,
            RateOfReturn,
            SharpeRatio,
            SortinoRatio,
            WinRate,
        )

        interval = Annual365()
        tear_sheet = backtest.TearSheet(
            pnl=Decimal("100"),
            pnl_return=RateOfReturn.calculate(Decimal("0.1"), interval),
            sharpe_ratio=SharpeRatio.calculate(
                Decimal("0.02"), Decimal("0.1"), Decimal("0.05"), interval
            ),
            sortino_ratio=SortinoRatio.calculate(
                Decimal("0.02"), Decimal("0.1"), Decimal("0.03"), interval
            ),
            calmar_ratio=CalmarRatio.calculate(
                Decimal("0.02"), Decimal("0.1"), Decimal("0.02"), interval
            ),
            pnl_drawdown=None,
            pnl_drawdown_mean=None,
            pnl_drawdown_max=None,
            win_rate=WinRate.calculate(Decimal("7"), Decimal("10")),
            profit_factor=ProfitFactor.calculate(Decimal("200"), Decimal("100")),
        )
        assert tear_sheet.pnl == Decimal("100")
        assert tear_sheet.pnl_drawdown is None
        assert tear_sheet.pnl_drawdown_mean is None
        assert tear_sheet.pnl_drawdown_max is None
        assert tear_sheet.win_rate is not None
        assert tear_sheet.profit_factor is not None


class TestTearSheetAsset:
    """Test TearSheetAsset structure."""

    def test_creation(self):
        """Test creating a TearSheetAsset."""
        balance = backtest.Balance(total=Decimal("1000"), free=Decimal("900"))
        asset_balance = backtest.AssetBalance(
            asset="USDT", balance=balance, time_exchange=datetime(2023, 1, 1, 12, 0, 0)
        )
        drawdown = backtest.Drawdown(
            value=Decimal("-0.02"),
            time_start=datetime(2023, 1, 1, 10, 0, 0),
            time_end=datetime(2023, 1, 1, 11, 0, 0),
        )
        mean_drawdown = backtest.MeanDrawdown(
            mean_drawdown=Decimal("-0.015"), mean_drawdown_ms=1800000
        )
        max_drawdown = backtest.MaxDrawdown(drawdown)

        tear_sheet = backtest.TearSheetAsset(
            balance_end=asset_balance,
            drawdown=drawdown,
            drawdown_mean=mean_drawdown,
            drawdown_max=max_drawdown,
        )
        assert tear_sheet.balance_end == asset_balance
        assert tear_sheet.drawdown == drawdown
        assert tear_sheet.drawdown_mean == mean_drawdown
        assert tear_sheet.drawdown_max == max_drawdown


class TestTradingSummary:
    """Test TradingSummary structure."""

    def test_creation(self):
        """Test creating a TradingSummary."""
        from barter_python.statistic import (
            Annual365,
            CalmarRatio,
            ProfitFactor,
            RateOfReturn,
            SharpeRatio,
            SortinoRatio,
            WinRate,
        )

        start_time = datetime(2023, 1, 1, 9, 0, 0)
        end_time = datetime(2023, 1, 1, 17, 0, 0)
        interval = Annual365()

        # Create a tear sheet
        tear_sheet = backtest.TearSheet(
            pnl=Decimal("50"),
            pnl_return=RateOfReturn.calculate(Decimal("0.05"), interval),
            sharpe_ratio=SharpeRatio.calculate(
                Decimal("0.02"), Decimal("0.05"), Decimal("0.03"), interval
            ),
            sortino_ratio=SortinoRatio.calculate(
                Decimal("0.02"), Decimal("0.05"), Decimal("0.02"), interval
            ),
            calmar_ratio=CalmarRatio.calculate(
                Decimal("0.02"), Decimal("0.05"), Decimal("0.01"), interval
            ),
            pnl_drawdown=None,
            pnl_drawdown_mean=None,
            pnl_drawdown_max=None,
            win_rate=WinRate.calculate(Decimal("6"), Decimal("10")),
            profit_factor=ProfitFactor.calculate(Decimal("150"), Decimal("100")),
        )

        summary = backtest.TradingSummary(
            time_engine_start=start_time,
            time_engine_end=end_time,
            instruments={"instrument_0": tear_sheet},
            assets={},
        )

        assert summary.time_engine_start == start_time
        assert summary.time_engine_end == end_time
        assert len(summary.instruments) == 1
        assert "instrument_0" in summary.instruments
        assert len(summary.assets) == 0
        assert summary.trading_duration == timedelta(hours=8)
