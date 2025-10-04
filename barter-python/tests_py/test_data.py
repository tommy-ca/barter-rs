"""Unit tests for pure Python data structures."""


from datetime import datetime, timezone
from decimal import Decimal

from barter_python.data import (
    Asks,
    Bids,
    Candle,
    DataKind,
    Level,
    Liquidation,
    MarketEvent,
    OrderBook,
    OrderBookEvent,
    OrderBookL1,
    OrderBookSide,
    PublicTrade,
    as_candle,
    as_liquidation,
    as_order_book_l1,
    as_public_trade,
)
from barter_python.instrument import (
    MarketDataInstrument,
    MarketDataInstrumentKind,
    Side,
)


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


class TestCandle:
    def test_creation(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        candle = Candle(time, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        assert candle.close_time == time
        assert candle.open == 50000.0
        assert candle.high == 51000.0
        assert candle.low == 49000.0
        assert candle.close == 50500.0
        assert candle.volume == 100.0
        assert candle.trade_count == 50

    def test_equality(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        candle1 = Candle(time, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        candle2 = Candle(time, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        candle3 = Candle(time, 50001.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        assert candle1 == candle2
        assert candle1 != candle3

    def test_repr(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        candle = Candle(time, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        assert "Candle(" in repr(candle)


class TestLiquidation:
    def test_creation(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        liquidation = Liquidation(Side.BUY, 50000.0, 0.1, time)
        assert liquidation.side == Side.BUY
        assert liquidation.price == 50000.0
        assert liquidation.quantity == 0.1
        assert liquidation.time == time

    def test_equality(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        liq1 = Liquidation(Side.BUY, 50000.0, 0.1, time)
        liq2 = Liquidation(Side.BUY, 50000.0, 0.1, time)
        liq3 = Liquidation(Side.SELL, 50000.0, 0.1, time)
        assert liq1 == liq2
        assert liq1 != liq3

    def test_repr(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        liquidation = Liquidation(Side.BUY, 50000.0, 0.1, time)
        assert "Liquidation(" in repr(liquidation)


class TestOrderBookSide:
    def test_bids_creation(self):
        levels = [
            Level(Decimal("100"), Decimal("1")),
            Level(Decimal("90"), Decimal("2")),
        ]
        side = OrderBookSide.bids(levels)
        assert isinstance(side.side, Bids)
        # Should be sorted descending
        assert side.levels[0].price == Decimal("100")
        assert side.levels[1].price == Decimal("90")

    def test_asks_creation(self):
        levels = [
            Level(Decimal("90"), Decimal("2")),
            Level(Decimal("100"), Decimal("1")),
        ]
        side = OrderBookSide.asks(levels)
        assert isinstance(side.side, Asks)
        # Should be sorted ascending
        assert side.levels[0].price == Decimal("90")
        assert side.levels[1].price == Decimal("100")

    def test_best(self):
        levels = [
            Level(Decimal("100"), Decimal("1")),
            Level(Decimal("90"), Decimal("2")),
        ]
        side = OrderBookSide.bids(levels)
        assert side.best() == levels[0]

    def test_best_empty(self):
        side = OrderBookSide.bids([])
        assert side.best() is None


class TestOrderBook:
    def test_creation(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bids = [Level(Decimal("100"), Decimal("1"))]
        asks = [Level(Decimal("101"), Decimal("1"))]
        ob = OrderBook.new(1, time, bids, asks)
        assert ob.sequence == 1
        assert ob.time_engine == time
        assert len(ob.bids.levels) == 1
        assert len(ob.asks.levels) == 1

    def test_mid_price(self):
        bids = [Level(Decimal("100"), Decimal("1"))]
        asks = [Level(Decimal("101"), Decimal("1"))]
        ob = OrderBook.new(1, None, bids, asks)
        assert ob.mid_price() == Decimal("100.5")

    def test_mid_price_none(self):
        ob = OrderBook.new(1, None, [], [])
        assert ob.mid_price() is None

    def test_volume_weighted_mid_price(self):
        bids = [Level(Decimal("100"), Decimal("2"))]
        asks = [Level(Decimal("101"), Decimal("1"))]
        ob = OrderBook.new(1, None, bids, asks)
        # (100*1 + 101*2) / (2+1) = (100 + 202) / 3 = 302 / 3 = 100.666...
        expected = Decimal("302") / Decimal("3")
        assert ob.volume_weighted_mid_price() == expected

    def test_equality(self):
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        bids = [Level(Decimal("100"), Decimal("1"))]
        asks = [Level(Decimal("101"), Decimal("1"))]
        ob1 = OrderBook.new(1, time, bids, asks)
        ob2 = OrderBook.new(1, time, bids, asks)
        ob3 = OrderBook.new(2, time, bids, asks)
        assert ob1 == ob2
        assert ob1 != ob3

    def test_repr(self):
        bids = [Level(Decimal("100"), Decimal("1"))]
        asks = [Level(Decimal("101"), Decimal("1"))]
        ob = OrderBook.new(1, None, bids, asks)
        assert "OrderBook(" in repr(ob)


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
        from barter_python.data import Candle
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        candle = Candle(time, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        dk = DataKind.candle(candle)
        assert dk.kind == "candle"
        assert dk.data == candle
        assert dk.kind_name() == "candle"

    def test_liquidation(self):
        from barter_python.data import Liquidation
        time = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        liquidation = Liquidation(Side.BUY, 50000.0, 0.1, time)
        dk = DataKind.liquidation(liquidation)
        assert dk.kind == "liquidation"
        assert dk.data == liquidation
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

    def test_as_candle(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        candle = Candle(time_ex, 50000.0, 51000.0, 49000.0, 50500.0, 100.0, 50)
        dk = DataKind.candle(candle)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_candle(event)
        assert result is not None
        assert result.kind == candle

    def test_as_candle_none(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        trade = PublicTrade("123", 50000.0, 0.1, Side.BUY)
        dk = DataKind.trade(trade)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_candle(event)
        assert result is None

    def test_as_liquidation(self):
        time_ex = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        time_rec = datetime(2024, 1, 1, 12, 0, 1, tzinfo=timezone.utc)
        instrument = MarketDataInstrument.new("btc", "usdt", MarketDataInstrumentKind.spot())
        liquidation = Liquidation(Side.BUY, 50000.0, 0.1, time_ex)
        dk = DataKind.liquidation(liquidation)
        event = MarketEvent(time_ex, time_rec, "binance", instrument, dk)

        result = as_liquidation(event)
        assert result is not None
        assert result.kind == liquidation
