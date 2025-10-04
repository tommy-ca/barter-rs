"""Unit tests for pure Python strategy implementations."""

from barter_python.execution import ClientOrderId, OrderKind, StrategyId, TimeInForce
from barter_python.instrument import Side
from barter_python.strategy import (
    DefaultStrategy,
    EngineState,
    InstrumentState,
    Position,
    build_ioc_market_order_to_close_position,
    cancel_all_orders_on_disconnect,
    close_all_positions_on_trading_disabled,
    close_open_positions_with_market_orders,
)


class TestClosePositionsStrategy:
    def test_build_ioc_market_order_to_close_long_position(self):
        """Test building IOC market order to close a long position."""
        position = Position(
            instrument=1, side=Side.BUY, quantity_abs=100.0, entry_price=50000.0
        )
        strategy_id = StrategyId.new("test-strategy")

        def gen_cid():
            return ClientOrderId.new("close-123")

        request = build_ioc_market_order_to_close_position(
            exchange=0,
            position=position,
            strategy_id=strategy_id,
            price=51000.0,
            gen_cid=gen_cid,
        )

        assert request.key.exchange == 0
        assert request.key.instrument == 1
        assert request.key.strategy == strategy_id
        assert request.key.cid.value == "close-123"
        assert request.state.side == Side.SELL
        assert str(request.state.price) == "51000.0"
        assert str(request.state.quantity) == "100.0"
        assert request.state.kind == OrderKind.MARKET
        assert request.state.time_in_force == TimeInForce.IMMEDIATE_OR_CANCEL

    def test_build_ioc_market_order_to_close_short_position(self):
        """Test building IOC market order to close a short position."""
        position = Position(
            instrument=2, side=Side.SELL, quantity_abs=50.0, entry_price=30000.0
        )
        strategy_id = StrategyId.new("test-strategy")

        def gen_cid():
            return ClientOrderId.new("close-456")

        request = build_ioc_market_order_to_close_position(
            exchange=1,
            position=position,
            strategy_id=strategy_id,
            price=29500.0,
            gen_cid=gen_cid,
        )

        assert request.key.exchange == 1
        assert request.key.instrument == 2
        assert request.state.side == Side.BUY
        assert str(request.state.price) == "29500.0"
        assert str(request.state.quantity) == "50.0"

    def test_close_open_positions_with_market_orders(self):
        """Test closing multiple open positions with market orders."""
        strategy_id = StrategyId.new("close-strategy")

        # Create test instruments with positions
        instruments = [
            InstrumentState(
                instrument=0,
                exchange=0,
                position=Position(0, Side.BUY, 100.0, 50000.0),
                price=51000.0,
            ),
            InstrumentState(
                instrument=1,
                exchange=0,
                position=Position(1, Side.SELL, 50.0, 30000.0),
                price=29500.0,
            ),
            InstrumentState(
                instrument=2,
                exchange=0,
                position=None,  # No position
                price=40000.0,
            ),
            InstrumentState(
                instrument=3,
                exchange=0,
                position=Position(3, Side.BUY, 25.0, 20000.0),
                price=None,  # No price
            ),
        ]

        state = EngineState(instruments)
        cancel_requests, open_requests = close_open_positions_with_market_orders(
            strategy_id, state
        )

        # Should have no cancel requests
        assert list(cancel_requests) == []

        # Should have 2 open requests (for instruments 0 and 1)
        open_requests = list(open_requests)
        assert len(open_requests) == 2

        # Check first request (closing long position)
        req1 = open_requests[0]
        assert req1.key.instrument == 0
        assert req1.state.side == Side.SELL
        assert str(req1.state.quantity) == "100.0"

        # Check second request (closing short position)
        req2 = open_requests[1]
        assert req2.key.instrument == 1
        assert req2.state.side == Side.BUY
        assert str(req2.state.quantity) == "50.0"

    def test_close_open_positions_no_positions(self):
        """Test closing positions when no positions exist."""
        strategy_id = StrategyId.new("close-strategy")

        instruments = [
            InstrumentState(instrument=0, exchange=0, position=None, price=50000.0),
            InstrumentState(instrument=1, exchange=0, position=None, price=30000.0),
        ]

        state = EngineState(instruments)
        cancel_requests, open_requests = close_open_positions_with_market_orders(
            strategy_id, state
        )

        assert list(cancel_requests) == []
        assert list(open_requests) == []

    def test_close_open_positions_custom_cid_generator(self):
        """Test closing positions with custom client ID generator."""
        strategy_id = StrategyId.new("close-strategy")

        instruments = [
            InstrumentState(
                instrument=0,
                exchange=0,
                position=Position(0, Side.BUY, 100.0, 50000.0),
                price=51000.0,
            ),
        ]

        state = EngineState(instruments)

        def custom_cid_gen(inst_state):
            return ClientOrderId.new(f"custom-{inst_state.instrument}")

        cancel_requests, open_requests = close_open_positions_with_market_orders(
            strategy_id, state, gen_cid=custom_cid_gen
        )

        open_requests = list(open_requests)
        assert len(open_requests) == 1
        assert open_requests[0].key.cid.value == "custom-0"


class TestDisconnectStrategies:
    def test_cancel_all_orders_on_disconnect(self):
        """Test canceling all orders on disconnect."""
        cancel_requests = cancel_all_orders_on_disconnect("binance_spot")

        # Currently returns empty list as placeholder
        assert cancel_requests == []


class TestTradingDisabledStrategies:
    def test_close_all_positions_on_trading_disabled(self):
        """Test closing all positions when trading is disabled."""
        cancel_requests, open_requests = close_all_positions_on_trading_disabled()

        # Currently returns empty lists as placeholders
        assert cancel_requests == []
        assert open_requests == []


class TestAlgoStrategy:
    def test_default_strategy_creation(self):
        """Test creating a DefaultStrategy instance."""
        strategy = DefaultStrategy()
        assert strategy.id.value == "default"

        custom_strategy = DefaultStrategy("custom-strategy")
        assert custom_strategy.id.value == "custom-strategy"

    def test_default_strategy_default_method(self):
        """Test the default() class method."""
        strategy = DefaultStrategy.default()
        assert strategy.id.value == "default"

    def test_default_strategy_generate_algo_orders(self):
        """Test that DefaultStrategy generates no algorithmic orders."""
        strategy = DefaultStrategy()
        state = EngineState([])  # Empty state

        cancel_requests, open_requests = strategy.generate_algo_orders(state)

        assert cancel_requests == []
        assert open_requests == []

    def test_default_strategy_close_positions_requests(self):
        """Test that DefaultStrategy closes positions using market orders."""
        strategy = DefaultStrategy("close-test")

        instruments = [
            InstrumentState(
                instrument=0,
                exchange=0,
                position=Position(0, Side.BUY, 100.0, 50000.0),
                price=51000.0,
            ),
        ]
        state = EngineState(instruments)

        cancel_requests, open_requests = strategy.close_positions_requests(state)

        assert cancel_requests == []
        assert len(open_requests) == 1
        assert open_requests[0].key.strategy == strategy.id
        assert open_requests[0].state.side == Side.SELL

    def test_default_strategy_on_disconnect(self):
        """Test that DefaultStrategy does nothing on disconnect."""
        strategy = DefaultStrategy()
        # Should not raise any exceptions
        strategy.on_disconnect("binance_spot")

    def test_default_strategy_on_trading_disabled(self):
        """Test that DefaultStrategy does nothing when trading is disabled."""
        strategy = DefaultStrategy()
        # Should not raise any exceptions
        strategy.on_trading_disabled()
