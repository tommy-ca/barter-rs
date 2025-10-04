"""Unit tests for pure Python execution data structures."""

from datetime import datetime, timezone
from decimal import Decimal

from barter_python.execution import (
    AccountEvent,
    AccountEventKind,
    AccountSnapshot,
    AssetBalance,
    AssetFees,
    Balance,
    CancelInFlight,
    Cancelled,
    ClientOrderId,
    InactiveOrderState,
    InstrumentAccountSnapshot,
    Open,
    OpenInFlight,
    Order,
    OrderError,
    OrderId,
    OrderKey,
    OrderKind,
    OrderResponseCancel,
    OrderState,
    StrategyId,
    TimeInForce,
    Trade,
    TradeId,
)
from barter_python.instrument import ExchangeId, QuoteAsset, Side


class TestOrderKind:
    def test_order_kind_enum_values(self):
        assert OrderKind.MARKET.value == "market"
        assert OrderKind.LIMIT.value == "limit"

    def test_order_kind_str(self):
        assert str(OrderKind.MARKET) == "market"
        assert str(OrderKind.LIMIT) == "limit"


class TestTimeInForce:
    def test_time_in_force_enum_values(self):
        assert TimeInForce.GOOD_UNTIL_CANCELLED.value == "good_until_cancelled"
        assert TimeInForce.GOOD_UNTIL_END_OF_DAY.value == "good_until_end_of_day"
        assert TimeInForce.FILL_OR_KILL.value == "fill_or_kill"
        assert TimeInForce.IMMEDIATE_OR_CANCEL.value == "immediate_or_cancel"

    def test_time_in_force_str(self):
        assert str(TimeInForce.GOOD_UNTIL_CANCELLED) == "good_until_cancelled"


class TestClientOrderId:
    def test_creation(self):
        cid = ClientOrderId.new("test-123")
        assert cid.value == "test-123"

    def test_equality(self):
        cid1 = ClientOrderId.new("test-123")
        cid2 = ClientOrderId.new("test-123")
        cid3 = ClientOrderId.new("test-456")
        assert cid1 == cid2
        assert cid1 != cid3

    def test_str_repr(self):
        cid = ClientOrderId.new("test-123")
        assert str(cid) == "test-123"
        assert repr(cid) == "ClientOrderId('test-123')"


class TestOrderId:
    def test_creation(self):
        oid = OrderId.new("order-123")
        assert oid.value == "order-123"

    def test_equality(self):
        oid1 = OrderId.new("order-123")
        oid2 = OrderId.new("order-123")
        oid3 = OrderId.new("order-456")
        assert oid1 == oid2
        assert oid1 != oid3

    def test_str_repr(self):
        oid = OrderId.new("order-123")
        assert str(oid) == "order-123"
        assert repr(oid) == "OrderId('order-123')"


class TestStrategyId:
    def test_creation(self):
        sid = StrategyId.new("strategy-alpha")
        assert sid.value == "strategy-alpha"

    @classmethod
    def test_unknown(cls):
        sid = StrategyId.unknown()
        assert sid.value == "unknown"

    def test_equality(self):
        sid1 = StrategyId.new("strategy-alpha")
        sid2 = StrategyId.new("strategy-alpha")
        sid3 = StrategyId.new("strategy-beta")
        assert sid1 == sid2
        assert sid1 != sid3

    def test_str_repr(self):
        sid = StrategyId.new("strategy-alpha")
        assert str(sid) == "strategy-alpha"
        assert repr(sid) == "StrategyId('strategy-alpha')"


class TestOrderKey:
    def test_creation(self):
        exchange = ExchangeId.BINANCE_SPOT
        instrument = 42
        strategy = StrategyId.new("strategy-alpha")
        cid = ClientOrderId.new("cid-123")

        key = OrderKey(exchange, instrument, strategy, cid)
        assert key.exchange == exchange
        assert key.instrument == instrument
        assert key.strategy == strategy
        assert key.cid == cid

    def test_equality(self):
        key1 = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("alpha"),
            ClientOrderId.new("cid-123"),
        )
        key2 = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("alpha"),
            ClientOrderId.new("cid-123"),
        )
        key3 = OrderKey(
            ExchangeId.KRAKEN, 42, StrategyId.new("alpha"), ClientOrderId.new("cid-123")
        )
        assert key1 == key2
        assert key1 != key3

    def test_str_repr(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("alpha"),
            ClientOrderId.new("cid-123"),
        )
        assert str(key) == "binance_spot:42:alpha:cid-123"
        assert "OrderKey(" in repr(key)


class TestBalance:
    def test_creation(self):
        balance = Balance.new(Decimal("100.5"), Decimal("95.2"))
        assert balance.total == Decimal("100.5")
        assert balance.free == Decimal("95.2")

    def test_used(self):
        balance = Balance(Decimal("100.5"), Decimal("95.2"))
        assert balance.used() == Decimal("5.3")

    def test_equality(self):
        b1 = Balance(Decimal("100.5"), Decimal("95.2"))
        b2 = Balance(Decimal("100.5"), Decimal("95.2"))
        b3 = Balance(Decimal("101.0"), Decimal("95.2"))
        assert b1 == b2
        assert b1 != b3

    def test_str_repr(self):
        balance = Balance(Decimal("100.5"), Decimal("95.2"))
        assert str(balance) == "Balance(total=100.5, free=95.2)"
        assert "Balance(" in repr(balance)


class TestAssetBalance:
    def test_creation(self):
        asset = "btc"
        balance = Balance(Decimal("100.5"), Decimal("95.2"))
        time_exchange = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        asset_balance = AssetBalance.new(asset, balance, time_exchange)
        assert asset_balance.asset == asset
        assert asset_balance.balance == balance
        assert asset_balance.time_exchange == time_exchange

    def test_equality(self):
        balance = Balance(Decimal("100.5"), Decimal("95.2"))
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        ab1 = AssetBalance("btc", balance, time)
        ab2 = AssetBalance("btc", balance, time)
        ab3 = AssetBalance("eth", balance, time)
        assert ab1 == ab2
        assert ab1 != ab3

    def test_str_repr(self):
        balance = Balance(Decimal("100.5"), Decimal("95.2"))
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        asset_balance = AssetBalance("btc", balance, time)
        assert "AssetBalance(" in repr(asset_balance)


class TestAssetFees:
    def test_creation(self):
        asset = "usdt"
        fees = Decimal("0.001")
        asset_fees = AssetFees(asset, fees)
        assert asset_fees.asset == asset
        assert asset_fees.fees == fees

    def test_quote_fees(self):
        fees = AssetFees.quote_fees(Decimal("0.001"))
        assert isinstance(fees.asset, QuoteAsset)
        assert fees.fees == Decimal("0.001")

    def test_equality(self):
        af1 = AssetFees("usdt", Decimal("0.001"))
        af2 = AssetFees("usdt", Decimal("0.001"))
        af3 = AssetFees("btc", Decimal("0.001"))
        assert af1 == af2
        assert af1 != af3

    def test_str_repr(self):
        asset_fees = AssetFees("usdt", Decimal("0.001"))
        assert "AssetFees(" in repr(asset_fees)


class TestTradeId:
    def test_creation(self):
        tid = TradeId.new("trade-123")
        assert tid.value == "trade-123"

    def test_equality(self):
        tid1 = TradeId.new("trade-123")
        tid2 = TradeId.new("trade-123")
        tid3 = TradeId.new("trade-456")
        assert tid1 == tid2
        assert tid1 != tid3

    def test_str_repr(self):
        tid = TradeId.new("trade-123")
        assert str(tid) == "trade-123"
        assert repr(tid) == "TradeId('trade-123')"


class TestTrade:
    def test_creation(self):
        tid = TradeId.new("trade-123")
        oid = OrderId.new("order-456")
        instrument = 42
        strategy = StrategyId.new("strategy-alpha")
        time_exchange = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        side = Side.BUY
        price = Decimal("50000.0")
        quantity = Decimal("0.1")
        fees = AssetFees(QuoteAsset(), Decimal("0.005"))

        trade = Trade(
            tid, oid, instrument, strategy, time_exchange, side, price, quantity, fees
        )
        assert trade.id == tid
        assert trade.order_id == oid
        assert trade.instrument == instrument
        assert trade.strategy == strategy
        assert trade.time_exchange == time_exchange
        assert trade.side == side
        assert trade.price == price
        assert trade.quantity == quantity
        assert trade.fees == fees

    def test_value_quote(self):
        trade = Trade(
            TradeId.new("trade-123"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        assert trade.value_quote() == Decimal("5000.0")

    def test_equality(self):
        # Create two identical trades
        trade1 = Trade(
            TradeId.new("trade-123"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        trade2 = Trade(
            TradeId.new("trade-123"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        trade3 = Trade(
            TradeId.new("trade-456"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        assert trade1 == trade2
        assert trade1 != trade3

    def test_str_repr(self):
        trade = Trade(
            TradeId.new("trade-123"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        assert "Trade(" in repr(trade)


class TestOpenInFlight:
    def test_creation(self):
        oif = OpenInFlight()
        assert isinstance(oif, OpenInFlight)

    def test_str_repr(self):
        oif = OpenInFlight()
        assert str(oif) == "OpenInFlight"
        assert repr(oif) == "OpenInFlight()"


class TestOpen:
    def test_creation(self):
        oid = OrderId.new("order-123")
        time_exchange = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        filled_quantity = Decimal("0.05")

        open_state = Open(oid, time_exchange, filled_quantity)
        assert open_state.id == oid
        assert open_state.time_exchange == time_exchange
        assert open_state.filled_quantity == filled_quantity

    def test_quantity_remaining(self):
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        assert open_state.quantity_remaining(Decimal("0.1")) == Decimal("0.05")

    def test_equality(self):
        oid = OrderId.new("order-123")
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        filled = Decimal("0.05")

        o1 = Open(oid, time, filled)
        o2 = Open(oid, time, filled)
        o3 = Open(OrderId.new("order-456"), time, filled)
        assert o1 == o2
        assert o1 != o3

    def test_str_repr(self):
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        assert "Open(" in repr(open_state)


class TestCancelInFlight:
    def test_creation(self):
        cif = CancelInFlight.new()
        assert cif.order is None

    def test_creation_with_order(self):
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        cif = CancelInFlight.new(open_state)
        assert cif.order == open_state

    def test_equality(self):
        cif1 = CancelInFlight.new()
        cif2 = CancelInFlight.new()
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        cif3 = CancelInFlight.new(open_state)
        assert cif1 == cif2
        assert cif1 != cif3

    def test_str_repr(self):
        cif = CancelInFlight.new()
        assert "CancelInFlight(" in repr(cif)


class TestCancelled:
    def test_creation(self):
        oid = OrderId.new("order-123")
        time_exchange = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        cancelled = Cancelled(oid, time_exchange)
        assert cancelled.id == oid
        assert cancelled.time_exchange == time_exchange

    def test_equality(self):
        oid = OrderId.new("order-123")
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        c1 = Cancelled(oid, time)
        c2 = Cancelled(oid, time)
        c3 = Cancelled(OrderId.new("order-456"), time)
        assert c1 == c2
        assert c1 != c3

    def test_str_repr(self):
        cancelled = Cancelled(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )
        assert "Cancelled(" in repr(cancelled)


class TestOrderError:
    def test_enum_values(self):
        assert OrderError.INSUFFICIENT_BALANCE.value == "insufficient_balance"
        assert OrderError.INVALID_PRICE.value == "invalid_price"
        assert OrderError.INVALID_QUANTITY.value == "invalid_quantity"
        assert OrderError.UNKNOWN_INSTRUMENT.value == "unknown_instrument"
        assert OrderError.EXCHANGE_ERROR.value == "exchange_error"

    def test_str(self):
        assert str(OrderError.INSUFFICIENT_BALANCE) == "insufficient_balance"


class TestInactiveOrderState:
    def test_cancelled(self):
        cancelled = Cancelled(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )
        ios = InactiveOrderState.cancelled(cancelled)
        assert ios.is_cancelled()
        assert not ios.is_fully_filled()
        assert not ios.is_expired()
        assert not ios.is_open_failed()

    def test_fully_filled(self):
        ios = InactiveOrderState.fully_filled()
        assert not ios.is_cancelled()
        assert ios.is_fully_filled()
        assert not ios.is_expired()
        assert not ios.is_open_failed()

    def test_expired(self):
        ios = InactiveOrderState.expired()
        assert not ios.is_cancelled()
        assert not ios.is_fully_filled()
        assert ios.is_expired()
        assert not ios.is_open_failed()

    def test_open_failed(self):
        ios = InactiveOrderState.open_failed(OrderError.INSUFFICIENT_BALANCE)
        assert not ios.is_cancelled()
        assert not ios.is_fully_filled()
        assert not ios.is_expired()
        assert ios.is_open_failed()

    def test_equality(self):
        ios1 = InactiveOrderState.fully_filled()
        ios2 = InactiveOrderState.fully_filled()
        ios3 = InactiveOrderState.expired()
        assert ios1 == ios2
        assert ios1 != ios3

    def test_str_repr(self):
        ios = InactiveOrderState.fully_filled()
        assert "InactiveOrderState(" in repr(ios)


class TestOrderState:
    def test_active_open_in_flight(self):
        os = OrderState.active(OpenInFlight())
        assert os.is_active()
        assert not os.is_inactive()
        assert os.time_exchange() is None

    def test_active_open(self):
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        os = OrderState.active(open_state)
        assert os.is_active()
        assert not os.is_inactive()
        assert os.time_exchange() == datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

    def test_active_cancel_in_flight(self):
        open_state = Open(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Decimal("0.05"),
        )
        cif = CancelInFlight.new(open_state)
        os = OrderState.active(cif)
        assert os.is_active()
        assert not os.is_inactive()
        assert os.time_exchange() == datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

    def test_inactive_cancelled(self):
        cancelled = Cancelled(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )
        ios = InactiveOrderState.cancelled(cancelled)
        os = OrderState.inactive(ios)
        assert not os.is_active()
        assert os.is_inactive()
        assert os.time_exchange() == datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

    def test_fully_filled(self):
        os = OrderState.fully_filled()
        assert not os.is_active()
        assert os.is_inactive()
        assert os.time_exchange() is None

    def test_expired(self):
        os = OrderState.expired()
        assert not os.is_active()
        assert os.is_inactive()
        assert os.time_exchange() is None

    def test_equality(self):
        os1 = OrderState.fully_filled()
        os2 = OrderState.fully_filled()
        os3 = OrderState.expired()
        assert os1 == os2
        assert os1 != os3

    def test_str_repr(self):
        os = OrderState.fully_filled()
        assert "OrderState(" in repr(os)


class TestOrder:
    def test_creation(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("strategy-alpha"),
            ClientOrderId.new("cid-123"),
        )
        side = Side.BUY
        price = Decimal("50000.0")
        quantity = Decimal("0.1")
        kind = OrderKind.LIMIT
        time_in_force = TimeInForce.GOOD_UNTIL_CANCELLED
        state = OrderState.fully_filled()

        order = Order(key, side, price, quantity, kind, time_in_force, state)
        assert order.key == key
        assert order.side == side
        assert order.price == price
        assert order.quantity == quantity
        assert order.kind == kind
        assert order.time_in_force == time_in_force
        assert order.state == state

    def test_equality(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("strategy-alpha"),
            ClientOrderId.new("cid-123"),
        )
        state = OrderState.fully_filled()

        order1 = Order(
            key,
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            OrderKind.LIMIT,
            TimeInForce.GOOD_UNTIL_CANCELLED,
            state,
        )
        order2 = Order(
            key,
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            OrderKind.LIMIT,
            TimeInForce.GOOD_UNTIL_CANCELLED,
            state,
        )
        order3 = Order(
            key,
            Side.SELL,
            Decimal("50000.0"),
            Decimal("0.1"),
            OrderKind.LIMIT,
            TimeInForce.GOOD_UNTIL_CANCELLED,
            state,
        )
        assert order1 == order2
        assert order1 != order3

    def test_str_repr(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("strategy-alpha"),
            ClientOrderId.new("cid-123"),
        )
        state = OrderState.fully_filled()

        order = Order(
            key,
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            OrderKind.LIMIT,
            TimeInForce.GOOD_UNTIL_CANCELLED,
            state,
        )
        assert "Order(" in repr(order)


class TestOrderResponseCancel:
    def test_creation(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("strategy-alpha"),
            ClientOrderId.new("cid-123"),
        )
        cancelled = Cancelled(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )

        response = OrderResponseCancel(key, cancelled)
        assert response.key == key
        assert response.state == cancelled

    def test_str_repr(self):
        key = OrderKey(
            ExchangeId.BINANCE_SPOT,
            42,
            StrategyId.new("strategy-alpha"),
            ClientOrderId.new("cid-123"),
        )
        cancelled = Cancelled(
            OrderId.new("order-123"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )

        response = OrderResponseCancel(key, cancelled)
        assert "OrderResponseCancel(" in repr(response)


class TestInstrumentAccountSnapshot:
    def test_creation(self):
        instrument = 42
        orders = [
            Order(
                OrderKey(
                    ExchangeId.BINANCE_SPOT,
                    42,
                    StrategyId.new("alpha"),
                    ClientOrderId.new("cid-1"),
                ),
                Side.BUY,
                Decimal("50000.0"),
                Decimal("0.1"),
                OrderKind.LIMIT,
                TimeInForce.GOOD_UNTIL_CANCELLED,
                OrderState.fully_filled(),
            )
        ]

        snapshot = InstrumentAccountSnapshot.new(instrument, orders)
        assert snapshot.instrument == instrument
        assert snapshot.orders == orders

    def test_creation_empty_orders(self):
        snapshot = InstrumentAccountSnapshot.new(42)
        assert snapshot.instrument == 42
        assert snapshot.orders == []

    def test_equality(self):
        snapshot1 = InstrumentAccountSnapshot.new(42)
        snapshot2 = InstrumentAccountSnapshot.new(42)
        snapshot3 = InstrumentAccountSnapshot.new(43)
        assert snapshot1 == snapshot2
        assert snapshot1 != snapshot3

    def test_str_repr(self):
        snapshot = InstrumentAccountSnapshot.new(42)
        assert "InstrumentAccountSnapshot(" in repr(snapshot)


class TestAccountSnapshot:
    def test_creation(self):
        exchange = ExchangeId.BINANCE_SPOT
        balances = [
            AssetBalance.new(
                "btc",
                Balance(Decimal("1.0"), Decimal("0.9")),
                datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            )
        ]
        instruments = [InstrumentAccountSnapshot.new(42)]

        snapshot = AccountSnapshot.new(exchange, balances, instruments)
        assert snapshot.exchange == exchange
        assert snapshot.balances == balances
        assert snapshot.instruments == instruments

    def test_time_most_recent(self):
        time1 = datetime(2024, 1, 1, 11, 0, 0, tzinfo=timezone.utc)
        time2 = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)

        balances = [
            AssetBalance.new("btc", Balance(Decimal("1.0"), Decimal("0.9")), time1)
        ]
        instruments = [
            InstrumentAccountSnapshot.new(
                42,
                [
                    Order(
                        OrderKey(
                            ExchangeId.BINANCE_SPOT,
                            42,
                            StrategyId.new("alpha"),
                            ClientOrderId.new("cid-1"),
                        ),
                        Side.BUY,
                        Decimal("50000.0"),
                        Decimal("0.1"),
                        OrderKind.LIMIT,
                        TimeInForce.GOOD_UNTIL_CANCELLED,
                        OrderState.active(
                            Open(OrderId.new("order-123"), time2, Decimal("0.0"))
                        ),
                    )
                ],
            )
        ]

        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, balances, instruments)
        assert snapshot.time_most_recent() == time2

    def test_assets_instruments_iter(self):
        balances = [
            AssetBalance.new(
                "btc",
                Balance(Decimal("1.0"), Decimal("0.9")),
                datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            ),
            AssetBalance.new(
                "eth",
                Balance(Decimal("10.0"), Decimal("9.0")),
                datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            ),
        ]
        instruments = [
            InstrumentAccountSnapshot.new(42),
            InstrumentAccountSnapshot.new(43),
        ]

        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, balances, instruments)

        assets = list(snapshot.assets())
        assert "btc" in assets
        assert "eth" in assets

        instruments_list = list(snapshot.instruments_iter())
        assert 42 in instruments_list
        assert 43 in instruments_list

    def test_equality(self):
        snapshot1 = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        snapshot2 = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        snapshot3 = AccountSnapshot.new(ExchangeId.KRAKEN, [], [])
        assert snapshot1 == snapshot2
        assert snapshot1 != snapshot3

    def test_str_repr(self):
        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        assert "AccountSnapshot(" in repr(snapshot)


class TestAccountEventKind:
    def test_snapshot(self):
        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        aek = AccountEventKind.snapshot(snapshot)
        assert aek.kind == "snapshot"
        assert aek.data == snapshot

    def test_balance_snapshot(self):
        balance = AssetBalance.new(
            "btc",
            Balance(Decimal("1.0"), Decimal("0.9")),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )
        aek = AccountEventKind.balance_snapshot(balance)
        assert aek.kind == "balance_snapshot"
        assert aek.data == balance

    def test_order_snapshot(self):
        order = Order(
            OrderKey(
                ExchangeId.BINANCE_SPOT,
                42,
                StrategyId.new("alpha"),
                ClientOrderId.new("cid-1"),
            ),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            OrderKind.LIMIT,
            TimeInForce.GOOD_UNTIL_CANCELLED,
            OrderState.fully_filled(),
        )
        aek = AccountEventKind.order_snapshot(order)
        assert aek.kind == "order_snapshot"
        assert aek.data == order

    def test_order_cancelled(self):
        response = OrderResponseCancel(
            OrderKey(
                ExchangeId.BINANCE_SPOT,
                42,
                StrategyId.new("alpha"),
                ClientOrderId.new("cid-1"),
            ),
            Cancelled(
                OrderId.new("order-123"),
                datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            ),
        )
        aek = AccountEventKind.order_cancelled(response)
        assert aek.kind == "order_cancelled"
        assert aek.data == response

    def test_trade(self):
        trade = Trade(
            TradeId.new("trade-123"),
            OrderId.new("order-456"),
            42,
            StrategyId.new("strategy-alpha"),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
            Side.BUY,
            Decimal("50000.0"),
            Decimal("0.1"),
            AssetFees(QuoteAsset(), Decimal("0.005")),
        )
        aek = AccountEventKind.trade(trade)
        assert aek.kind == "trade"
        assert aek.data == trade

    def test_equality(self):
        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        aek1 = AccountEventKind.snapshot(snapshot)
        aek2 = AccountEventKind.snapshot(snapshot)
        balance = AssetBalance.new(
            "btc",
            Balance(Decimal("1.0"), Decimal("0.9")),
            datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
        )
        aek3 = AccountEventKind.balance_snapshot(balance)
        assert aek1 == aek2
        assert aek1 != aek3

    def test_str_repr(self):
        snapshot = AccountSnapshot.new(ExchangeId.BINANCE_SPOT, [], [])
        aek = AccountEventKind.snapshot(snapshot)
        assert "AccountEventKind(" in repr(aek)


class TestAccountEvent:
    def test_creation(self):
        exchange = ExchangeId.BINANCE_SPOT
        snapshot = AccountSnapshot.new(exchange, [], [])
        kind = AccountEventKind.snapshot(snapshot)

        event = AccountEvent.new(exchange, kind)
        assert event.exchange == exchange
        assert event.kind == kind

    def test_equality(self):
        exchange = ExchangeId.BINANCE_SPOT
        snapshot = AccountSnapshot.new(exchange, [], [])
        kind = AccountEventKind.snapshot(snapshot)

        event1 = AccountEvent.new(exchange, kind)
        event2 = AccountEvent.new(exchange, kind)
        event3 = AccountEvent.new(ExchangeId.KRAKEN, kind)
        assert event1 == event2
        assert event1 != event3

    def test_str_repr(self):
        exchange = ExchangeId.BINANCE_SPOT
        snapshot = AccountSnapshot.new(exchange, [], [])
        kind = AccountEventKind.snapshot(snapshot)

        event = AccountEvent.new(exchange, kind)
        assert "AccountEvent(" in repr(event)
