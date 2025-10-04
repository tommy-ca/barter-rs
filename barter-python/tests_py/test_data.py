"""Unit tests for pure Python data structures."""

import pytest

from datetime import datetime, timezone
from decimal import Decimal

from barter_python.data import (
    DataKind,
    Level,
    MarketEvent,
    OrderBookEvent,
    OrderBookL1,
    PublicTrade,
    as_order_book_l1,
    as_public_trade,
)
from barter_python.instrument import ExchangeId, MarketDataInstrument, MarketDataInstrumentKind, Side


class TestPublicTrade:
    def test_creation(self):
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        assert trade.id == "123"
        assert trade.price == 50000.0
        assert trade.amount == 0.1
        assert trade.side == Side.BUY

    def test_equality(self):
        trade1 = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        trade2 = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        trade3 = PublicTrade("456", 50000.0, 0.1, Side.BUY)
        assert trade1 == trade2
        assert trade1 != trade3

    def test_repr(self):
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        assert "PublicTrade(" in repr(trade)


class TestLevel:
    def test_creation(self):
        level = Level(Decimal("50000"), Decimal("0.1"))
        assert level.price == Decimal("50000")
        assert level.amount == Decimal("0.1")

    def test_new(self):
        level = Level.new(Decimal("50000"), Decimal("0.1"))
        assert level.price == Decimal("50000")
        assert level.amount == Decimal("0.1")

    def test_equality(self):
        level1 = Level(Decimal("50000"), Decimal("0.1"))
        level2 = Level(Decimal("50000"), Decimal("0.1"))
        level3 = Level(Decimal("50001"), Decimal("0.1"))
        assert level1 == level2
        assert level1 != level3

    def test_repr(self):
        level = Level(Decimal("50000"), Decimal("0.1"))
        assert "Level(" in repr(level)


class TestOrderBookL1:
    def test_creation(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1 = OrderBookL1(time, bid, ask)
        assert obl1.last_update_time == time
        assert obl1.best_bid == bid
        assert obl1.best_ask == ask

    def test_new(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1 = OrderBookL1.new(time, bid, ask)
        assert obl1.last_update_time == time
        assert obl1.best_bid == bid
        assert obl1.best_ask == ask

    def test_mid_price(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1 = OrderBookL1(time, bid, ask)
        assert obl1.mid_price() == Decimal("50000")

    def test_mid_price_none(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        obl1 = OrderBookL1(time, None, None)
        assert obl1.mid_price() is None

    def test_volume_weighted_mid_price(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1 = OrderBookL1(time, bid, ask)
        # (49999 * 0.3 + 50001 * 0.5) / (0.5 + 0.3) = (14999.7 + 25000.5) / 0.8 = 40000.2 / 0.8 = 50000.25
        assert obl1.volume_weighted_mid_price() == Decimal("50000.25")

    def test_equality(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1_1 = OrderBookL1(time, bid, ask)
        obl1_2 = OrderBookL1(time, bid, ask)
        obl1_3 = OrderBookL1(time, None, ask)
        assert obl1_1 == obl1_2
        assert obl1_1 != obl1_3

    def test_repr(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bid = Level(Decimal("49999"), Decimal("0.5"))
        ask = Level(Decimal("50001"), Decimal("0.3"))
        obl1 = OrderBookL1(time, bid, ask)
        assert "OrderBookL1(" in repr(obl1)


class TestOrderBookEvent:
    def test_enum_values(self):
        assert OrderBookEvent.SNAPSHOT.value == "snapshot"
        assert OrderBookEvent.UPDATE.value == "update"

    def test_str(self):
        assert str(OrderBookEvent.SNAPSHOT) == "snapshot"


class TestDataKind:
    def test_trade(self):
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        dk = DataKind.trade(trade)
        assert dk.kind == "trade"
        assert dk.data == trade
        assert dk.kind_name() == "public_trade"

    def test_order_book_l1(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        obl1 = OrderBookL1(time)
        dk = DataKind.order_book_l1(obl1)
        assert dk.kind == "order_book_l1"
        assert dk.data == obl1
        assert dk.kind_name() == "l1"

    def test_order_book(self):
        dk = DataKind.order_book(OrderBookEvent.SNAPSHOT)
        assert dk.kind == "order_book"
        assert dk.data == OrderBookEvent.SNAPSHOT
        assert dk.kind_name() == "l2"

    def test_candle(self):
        dk = DataKind.candle()
        assert dk.kind == "candle"
        assert dk.data is None
        assert dk.kind_name() == "candle"

    def test_liquidation(self):
        dk = DataKind.liquidation()
        assert dk.kind == "liquidation"
        assert dk.data is None
        assert dk.kind_name() == "liquidation"

    def test_equality(self):
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        dk1 = DataKind.trade(trade)
        dk2 = DataKind.trade(trade)
        dk3 = DataKind.order_book_l1(OrderBookL1(datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)))
        assert dk1 == dk2
        assert dk1 != dk3

    def test_repr(self):
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        dk = DataKind.trade(trade)
        assert "DataKind(" in repr(dk)


class TestMarketEvent:
    def test_creation(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, trade)
        assert event.time_exchange == time_ex
        assert event.time_received == time_rec
        assert event.exchange == "binance"
        assert event.instrument == instrument
        assert event.kind == trade

    def test_map_kind(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, trade)

        def double_price(t):
            return PublicTrade(t.id, t.price * 2, t.amount, t.side)

        new_event = event.map_kind(double_price)
        assert new_event.kind.price == 100000.0

    def test_equality(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        event1 = MarketEvent(time_ex, time_rec, "binance", instrument, trade)
        event2 = MarketEvent(time_ex, time_rec, "binance", instrument, trade)
        event3 = MarketEvent(time_ex, time_rec, "kraken", instrument, trade)
        assert event1 == event2
        assert event1 != event3

    def test_repr(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, trade)
        assert "MarketEvent(" in repr(event)


class TestAsFunctions:
    def test_as_public_trade(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        dk = DataKind.trade(trade)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_public_trade(event)
        assert result is not None
        assert result.kind == trade

    def test_as_public_trade_none(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        obl1 = OrderBookL1(time_ex)
        dk = DataKind.order_book_l1(obl1)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_public_trade(event)
        assert result is None

    def test_as_order_book_l1(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        obl1 = OrderBookL1(time_ex)
        dk = DataKind.order_book_l1(obl1)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_order_book_l1(event)
        assert result is not None
        assert result.kind == obl1