"""Tests for the pure Python engine module."""

from datetime import datetime, timezone
from decimal import Decimal

import barter_python as bp
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
from barter_python.execution import (
    AccountEvent,
    AccountEventKind,
    AccountSnapshot,
    AssetFees,
    CancelInFlight,
    Cancelled,
    ClientOrderId,
    OpenInFlight,
    Order,
    OrderId,
    OrderKey,
    OrderKind,
    OrderRequestCancel,
    OrderRequestOpen,
    OrderResponseCancel,
    OrderState,
    RequestOpen,
    StrategyId,
    TimeInForce,
    Trade,
    TradeId,
)
from barter_python.execution import (
    AssetBalance as ExecutionAssetBalance,
)
from barter_python.execution import (
    Balance as ExecutionBalance,
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
            last_price=Decimal("100.5"),
            last_update_time=time,
        )
        assert data.last_price == Decimal("100.5")
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
        assert updated_state.market_data.last_price == Decimal("100.5")
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
        assert updated_state.market_data.last_price == Decimal("100.5")
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

    def test_process_account_event_snapshot_updates_balances_and_orders(self):
        """Snapshot events should refresh balances and instrument orders."""
        initial_state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)
        initial_state.update_instrument_state(1, inst_state)  # type: ignore[arg-type]

        engine = Engine(initial_state, DefaultStrategy(), DefaultRiskManager())

        # Prepare account snapshot with one balance and one order
        time_exchange = datetime(2024, 1, 1, 12, 0, tzinfo=timezone.utc)
        balance_wrapper = ExecutionAssetBalance.new(
            asset=bp.AssetIndex(0),
            balance=ExecutionBalance.new(Decimal("1000"), Decimal("400")),
            time_exchange=time_exchange,
        )

        order_key = OrderKey(
            exchange=0,  # type: ignore[arg-type]
            instrument=1,  # type: ignore[arg-type]
            strategy=StrategyId.new("strat-1"),
            cid=ClientOrderId.new("order-1"),
        )
        order = Order(
            key=order_key,
            side=Side.BUY,
            price=Decimal("25000"),
            quantity=Decimal("0.1"),
            kind=OrderKind.LIMIT,
            time_in_force=TimeInForce.GOOD_UNTIL_CANCELLED,
            state=OrderState.active(OpenInFlight()),  # type: ignore[arg-type]
        )

        snapshot = AccountSnapshot(
            exchange=0,  # type: ignore[arg-type]
            balances=[(0, 1000.0, 400.0, time_exchange)],
            instruments=[],
        )
        account_event = AccountEvent.new(
            exchange=0,  # type: ignore[arg-type]
            kind=AccountEventKind.snapshot(snapshot),
        )

        engine.process_account_event(account_event)

        assert "0" in engine.state.balances
        balance_state = engine.state.balances["0"]
        assert balance_state == balance_wrapper
        assert balance_state.asset == 0
        assert balance_state.balance.total == Decimal("1000")
        assert balance_state.balance.free == Decimal("400")

        order_event = AccountEvent.new(
            exchange=0,  # type: ignore[arg-type]
            kind=AccountEventKind.order_snapshot(order),
        )
        engine.process_account_event(order_event)

        inst_state_after = engine.state.get_instrument_state(1)  # type: ignore[arg-type]
        assert inst_state_after is not None
        assert order_key in inst_state_after.orders
        assert inst_state_after.orders[order_key].price == Decimal("25000")

    def test_process_account_event_order_snapshot_updates_single_order(self):
        """Order snapshot event should upsert order in instrument state."""
        initial_state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)
        initial_state.update_instrument_state(1, inst_state)  # type: ignore[arg-type]

        engine = Engine(initial_state, DefaultStrategy(), DefaultRiskManager())

        order_key = OrderKey(
            exchange=0,  # type: ignore[arg-type]
            instrument=1,  # type: ignore[arg-type]
            strategy=StrategyId.new("strat-1"),
            cid=ClientOrderId.new("order-2"),
        )
        order = Order(
            key=order_key,
            side=Side.SELL,
            price=Decimal("26000"),
            quantity=Decimal("0.2"),
            kind=OrderKind.LIMIT,
            time_in_force=TimeInForce.GOOD_UNTIL_CANCELLED,
            state=OrderState.active(OpenInFlight()),  # type: ignore[arg-type]
        )

        account_event = AccountEvent.new(
            exchange=0,  # type: ignore[arg-type]
            kind=AccountEventKind.order_snapshot(order),
        )

        engine.process_account_event(account_event)

        inst_state_after = engine.state.get_instrument_state(1)  # type: ignore[arg-type]
        assert inst_state_after is not None
        assert order_key in inst_state_after.orders
        assert inst_state_after.orders[order_key].quantity == Decimal("0.2")

    def test_process_account_event_order_cancelled_removes_order(self):
        """Order cancellation should remove the order from instrument state."""
        initial_state = EngineState()

        order_key = OrderKey(
            exchange=0,  # type: ignore[arg-type]
            instrument=1,  # type: ignore[arg-type]
            strategy=StrategyId.new("strat-2"),
            cid=ClientOrderId.new("order-3"),
        )

        order = Order(
            key=order_key,
            side=Side.BUY,
            price=Decimal("25500"),
            quantity=Decimal("0.05"),
            kind=OrderKind.LIMIT,
            time_in_force=TimeInForce.GOOD_UNTIL_CANCELLED,
            state=OrderState.active(OpenInFlight()),  # type: ignore[arg-type]
        )

        inst_state = InstrumentState(instrument=1, exchange=0, orders={order_key: order})
        initial_state.update_instrument_state(1, inst_state)  # type: ignore[arg-type]

        engine = Engine(initial_state, DefaultStrategy(), DefaultRiskManager())

        cancel_response = OrderResponseCancel(
            key=order_key,
            state=Cancelled(
                id=OrderId.new("ex-order-3"),
                time_exchange=datetime(2024, 1, 1, 12, 5, tzinfo=timezone.utc),
            ),
        )
        account_event = AccountEvent.new(
            exchange=0,  # type: ignore[arg-type]
            kind=AccountEventKind.order_cancelled(cancel_response),
        )

        engine.process_account_event(account_event)

        inst_state_after = engine.state.get_instrument_state(1)  # type: ignore[arg-type]
        assert inst_state_after is not None
        assert order_key not in inst_state_after.orders

    def test_process_account_event_trade_updates_position(self):
        """Trades should update instrument position quantity and side."""
        initial_state = EngineState()
        inst_state = InstrumentState(instrument=1, exchange=0)
        initial_state.update_instrument_state(1, inst_state)  # type: ignore[arg-type]

        engine = Engine(initial_state, DefaultStrategy(), DefaultRiskManager())

        trade_event = Trade(
            TradeId.new("trade-1"),
            OrderId.new("order-1"),
            1,  # type: ignore[arg-type]
            StrategyId.new("strat-1"),
            datetime(2024, 1, 1, 12, 10, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("25000"),
            Decimal("0.1"),
            AssetFees.quote_fees(Decimal("5")),
        )
        account_event = AccountEvent.new(
            exchange=0,  # type: ignore[arg-type]
            kind=AccountEventKind.trade(trade_event),
        )

        engine.process_account_event(account_event)

        inst_state_after = engine.state.get_instrument_state(1)  # type: ignore[arg-type]
        assert inst_state_after is not None
        assert inst_state_after.position is not None
        assert inst_state_after.position.side == "buy"
        assert inst_state_after.position.quantity_abs == Decimal("0.1")
        assert inst_state_after.position.entry_price == Decimal("25000")

    def test_send_requests_records_open_orders(self):
        """SendRequests should track approved opens as OpenInFlight orders."""

        exchange_index = 0
        instrument_index = 5
        state = EngineState()
        instrument_state = InstrumentState(
            instrument=instrument_index,
            exchange=exchange_index,
        )
        state.update_instrument_state(instrument_index, instrument_state)

        class StubStrategy:
            def generate_algo_orders(self, *_):
                return [], []

        class StubRiskManager:
            class _Approved:
                def __init__(self, item):
                    self.item = item

            def check(self, _state, cancel_requests, open_requests):
                approved = self._Approved
                return (
                    [approved(item) for item in cancel_requests],
                    [approved(item) for item in open_requests],
                    [],
                    [],
                )

        engine = Engine(state, StubStrategy(), StubRiskManager())

        from dataclasses import dataclass

        @dataclass(frozen=True)
        class _Key:
            exchange: int
            instrument: int
            strategy: str
            cid: str

        key = _Key(
            exchange=exchange_index,
            instrument=instrument_index,
            strategy="send-requests",
            cid="open-001",
        )
        request_state = RequestOpen(
            side=Side.BUY,
            price=Decimal("100.0"),
            quantity=Decimal("2.0"),
            kind=OrderKind.MARKET,
            time_in_force=TimeInForce.IMMEDIATE_OR_CANCEL,
        )
        open_request = OrderRequestOpen(key, request_state)

        engine.send_requests([open_request], [])

        stored = engine.state.instruments[instrument_index].orders[key]
        assert stored.side == Side.BUY
        assert stored.quantity == Decimal("2.0")
        assert stored.state.is_active()
        assert isinstance(stored.state.state, OpenInFlight)

    def test_send_requests_marks_cancel_in_flight(self):
        """SendRequests should mark existing orders as CancelInFlight when cancelling."""

        exchange_index = 1
        instrument_index = 9
        state = EngineState()
        instrument_state = InstrumentState(
            instrument=instrument_index,
            exchange=exchange_index,
        )
        state.update_instrument_state(instrument_index, instrument_state)

        class StubStrategy:
            def generate_algo_orders(self, *_):
                return [], []

        class StubRiskManager:
            class _Approved:
                def __init__(self, item):
                    self.item = item

            def check(self, _state, cancel_requests, open_requests):
                approved = self._Approved
                return (
                    [approved(item) for item in cancel_requests],
                    [approved(item) for item in open_requests],
                    [],
                    [],
                )

        engine = Engine(state, StubStrategy(), StubRiskManager())

        from dataclasses import dataclass

        @dataclass(frozen=True)
        class _Key:
            exchange: int
            instrument: int
            strategy: str
            cid: str

        key = _Key(
            exchange=exchange_index,
            instrument=instrument_index,
            strategy="send-requests",
            cid="open-002",
        )
        request_state = RequestOpen(
            side=Side.SELL,
            price=Decimal("101.0"),
            quantity=Decimal("1.5"),
            kind=OrderKind.LIMIT,
            time_in_force=TimeInForce.GOOD_UNTIL_CANCELLED,
        )
        order = Order(
            key,
            request_state.side,
            request_state.price,
            request_state.quantity,
            request_state.kind,
            request_state.time_in_force,
            OrderState.active(OpenInFlight()),
        )
        engine.state.instruments[instrument_index].orders[key] = order

        cancel_request = OrderRequestCancel(key, None)

        engine.send_requests([], [cancel_request])

        stored = engine.state.instruments[instrument_index].orders[key]
        assert stored.state.is_active()
        assert isinstance(stored.state.state, CancelInFlight)
