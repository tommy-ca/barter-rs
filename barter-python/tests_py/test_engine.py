"""Tests for the pure Python engine module."""

from datetime import datetime
from decimal import Decimal

from barter_python.data import Candle, DataKind, MarketEvent, PublicTrade
from barter_python.engine import (
    AllInstrumentsFilter,
    DefaultInstrumentMarketData,
    Engine,
    EngineState,
    ExchangeFilter,
    InstrumentState,
    TradingState,
)
from barter_python.instrument import Side
from barter_python.risk import DefaultRiskManager
from barter_python.strategy import DefaultStrategy


class TestTradingState:
    """Test TradingState functionality."""

    def test_enabled(self):
        """Test enabled trading state."""
        state = TradingState(enabled=True)
        assert state.enabled is True

    def test_disabled(self):
        """Test disabled trading state."""
        state = TradingState(enabled=False)
        assert state.enabled is False

    def test_trading_enabled_classmethod(self):
        """Test trading_enabled classmethod."""
        state = TradingState.trading_enabled()
        assert state.enabled is True

    def test_trading_disabled_classmethod(self):
        """Test trading_disabled classmethod."""
        state = TradingState.trading_disabled()
        assert state.enabled is False


class TestDefaultInstrumentMarketData:
    """Test DefaultInstrumentMarketData functionality."""

    def test_creation(self):
        """Test creation with default values."""
        data = DefaultInstrumentMarketData()
        assert data.last_price is None
        assert data.last_update_time is None
        assert data.order_book_l1 is None
        assert data.recent_candle is None

    def test_creation_with_values(self):
        """Test creation with values."""
        time = datetime(2023, 1, 1)
        data = DefaultInstrumentMarketData(
            last_price=Decimal('100.5'),
            last_update_time=time,
        )
        assert data.last_price == Decimal('100.5')
        assert data.last_update_time == time


class TestInstrumentState:
    """Test InstrumentState functionality."""

    def test_creation(self):
        """Test instrument state creation."""
        state = InstrumentState(
            instrument=1,  # type: ignore
            exchange=0,  # type: ignore
        )
        assert state.instrument == 1  # type: ignore
        assert state.exchange == 0  # type: ignore
        assert state.position is None
        assert isinstance(state.market_data, DefaultInstrumentMarketData)
        assert state.orders == {}

    def test_has_position(self):
        """Test has_position property."""
        state_no_pos = InstrumentState(instrument=1, exchange=0)
        assert not state_no_pos.has_position

        # Note: Position creation would require more setup
        # For now, just test the basic property


class TestEngineState:
    """Test EngineState functionality."""

    def test_creation(self):
        """Test engine state creation."""
        state = EngineState()
        assert state.instruments == {}
        assert isinstance(state.trading_state, TradingState)
        assert state.balances == {}

    def test_get_instrument_state(self):
        """Test getting instrument state."""
        state = EngineState()
        inst_state = state.get_instrument_state(1)
        assert inst_state is None

        # Add an instrument state
        state.instruments[1] = InstrumentState(instrument=1, exchange=0)  # type: ignore
        inst_state = state.get_instrument_state(1)  # type: ignore
        assert inst_state is not None
        assert inst_state.instrument == 1

    def test_update_instrument_state(self):
        """Test updating instrument state."""
        state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)  # type: ignore
        state.update_instrument_state(1, inst_state)  # type: ignore
        assert 1 in state.instruments  # type: ignore
        assert state.instruments[1].instrument == 1  # type: ignore

    def test_is_trading_enabled(self):
        """Test is_trading_enabled."""
        state = EngineState()
        assert state.is_trading_enabled()

        state.trading_state = TradingState(enabled=False)
        assert not state.is_trading_enabled()


class TestFilters:
    """Test instrument filters."""

    def test_all_instruments_filter(self):
        """Test AllInstrumentsFilter."""
        filter = AllInstrumentsFilter()
        assert filter.matches(0, 1)
        assert filter.matches(1, 2)

    def test_exchange_filter(self):
        """Test ExchangeFilter."""
        filter = ExchangeFilter(1)
        assert filter.matches(1, 1)
        assert filter.matches(1, 2)
        assert not filter.matches(0, 1)
        assert not filter.matches(2, 1)


class TestEngine:
    """Test Engine functionality."""

    def test_creation(self):
        """Test engine creation."""
        initial_state = EngineState()
        strategy = DefaultStrategy()
        risk_manager = DefaultRiskManager()
        engine = Engine(initial_state, strategy, risk_manager)
        assert engine.state is initial_state
        assert engine.strategy is strategy
        assert engine.risk_manager is risk_manager

    def test_process_market_event_trade(self):
        """Test processing market trade event."""
        initial_state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)
        initial_state.update_instrument_state(1, inst_state)

        strategy = DefaultStrategy()
        risk_manager = DefaultRiskManager()
        engine = Engine(initial_state, strategy, risk_manager)

        # Create a trade event
        time_exchange = datetime(2023, 1, 1, 12, 0, 0)
        time_received = datetime(2023, 1, 1, 12, 0, 1)
        trade = PublicTrade(id="trade1", price=100.5, amount=1.0, side=Side.BUY)
        event = MarketEvent(
            time_exchange=time_exchange,
            time_received=time_received,
            exchange="binance_spot",
            instrument=1,  # type: ignore
            kind=DataKind.trade(trade),
        )

        engine.process_market_event(event)

        # Check that market data was updated
        updated_state = engine.state.get_instrument_state(1)  # type: ignore
        assert updated_state is not None
        assert updated_state.market_data.last_price == Decimal('100.5')
        assert updated_state.market_data.last_update_time == time_exchange

    def test_process_market_event_candle(self):
        """Test processing market candle event."""
        initial_state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)
        initial_state.update_instrument_state(1, inst_state)

        strategy = DefaultStrategy()
        risk_manager = DefaultRiskManager()
        engine = Engine(initial_state, strategy, risk_manager)

        # Create a candle event
        time_exchange = datetime(2023, 1, 1, 12, 0, 0)
        time_received = datetime(2023, 1, 1, 12, 0, 1)
        candle = Candle(
            close_time=datetime(2023, 1, 1, 12, 5, 0),
            open=100.0,
            high=101.0,
            low=99.0,
            close=100.5,
            volume=100.0,
            trade_count=10,
        )
        event = MarketEvent(
            time_exchange=time_exchange,
            time_received=time_received,
            exchange="binance_spot",
            instrument=1,  # type: ignore
            kind=DataKind.candle(candle),
        )

        engine.process_market_event(event)

        # Check that market data was updated
        updated_state = engine.state.get_instrument_state(1)  # type: ignore
        assert updated_state is not None
        assert updated_state.market_data.last_price == Decimal('100.5')
        assert updated_state.market_data.last_update_time == time_exchange
        assert updated_state.market_data.recent_candle == candle

    def test_set_trading_enabled(self):
        """Test setting trading enabled/disabled."""
        initial_state = EngineState()
        strategy = DefaultStrategy()
        risk_manager = DefaultRiskManager()
        engine = Engine(initial_state, strategy, risk_manager)

        engine.set_trading_enabled(False)
        assert not engine.state.is_trading_enabled()

        engine.set_trading_enabled(True)
        assert engine.state.is_trading_enabled()
