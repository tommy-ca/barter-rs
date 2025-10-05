from __future__ import annotations

import datetime as dt

import pytest

import barter_python as bp
from barter_python.data import (
    DataKind,
    MarketEvent,
    MarketStreamItem,
    MarketStreamReconnecting,
    PublicTrade,
)
from barter_python.instrument import Side

UTC = dt.timezone.utc


def build_trade_event(
    *,
    exchange: str = "binance_spot",
    instrument: int = 7,
    trade_id: str = "trade-1",
    price: float = 101.25,
    amount: float = 0.5,
    side: str = "buy",
    time_exchange: dt.datetime | None = None,
    time_received: dt.datetime | None = None,
) -> dict:
    time_exchange = time_exchange or dt.datetime(2025, 10, 4, 12, 0, tzinfo=UTC)
    time_received = time_received or dt.datetime(2025, 10, 4, 12, 0, 1, tzinfo=UTC)

    return {
        "type": "item",
        "exchange": exchange,
        "instrument": instrument,
        "time_exchange": time_exchange,
        "time_received": time_received,
        "trade": {
            "id": trade_id,
            "price": price,
            "amount": amount,
            "side": side,
        },
    }


def build_reconnect_event(exchange: str = "binance_spot") -> dict:
    return {"type": "reconnecting", "exchange": exchange}


def build_error_event(exchange: str = "binance_spot", message: str = "socket error") -> dict:
    return {"type": "error", "exchange": exchange, "message": message}


def test_dynamic_trade_stream_yields_market_event():
    streams = bp._testing_dynamic_trades([build_trade_event()])

    stream = streams.select_trades(bp.ExchangeId.BINANCE_SPOT)
    assert stream is not None

    event = stream.recv()
    assert isinstance(event, MarketStreamItem)
    assert event.kind == "item"

    market_event = event.event
    assert market_event.exchange == "binance_spot"
    assert market_event.instrument == 7
    assert market_event.kind.kind == "trade"

    # Stream should now be exhausted
    assert stream.recv() is None


def test_dynamic_stream_handles_reconnect():
    streams = bp._testing_dynamic_trades([build_reconnect_event()])

    stream = streams.select_trades(bp.ExchangeId.BINANCE_SPOT)
    assert stream is not None

    reconnect = stream.recv()
    assert isinstance(reconnect, MarketStreamReconnecting)
    assert reconnect.kind == "reconnecting"
    assert reconnect.exchange == "binance_spot"


def test_dynamic_stream_propagates_errors():
    streams = bp._testing_dynamic_trades([build_error_event(message="down")])
    stream = streams.select_trades(bp.ExchangeId.BINANCE_SPOT)
    assert stream is not None

    with pytest.raises(ValueError) as exc:
        stream.recv()

    assert "down" in str(exc.value)


def test_market_stream_item_equality_and_repr():
    trade = PublicTrade("t-1", 101.25, 0.5, Side.BUY)
    kind = DataKind.trade(trade)
    time_exchange = dt.datetime(2025, 10, 4, 12, 0, tzinfo=UTC)
    event = MarketEvent(time_exchange, time_exchange, "binance_spot", 42, kind)

    item_a = MarketStreamItem(event)
    item_b = MarketStreamItem(event)

    assert item_a == item_b
    assert hash(item_a) == hash(item_b)
    assert "MarketStreamItem" in repr(item_a)


def test_market_stream_reconnecting_equality_and_repr():
    reconnect_a = MarketStreamReconnecting("binance_spot")
    reconnect_b = MarketStreamReconnecting("binance_spot")
    reconnect_c = MarketStreamReconnecting("kraken")

    assert reconnect_a == reconnect_b
    assert hash(reconnect_a) == hash(reconnect_b)
    assert reconnect_a != reconnect_c
    assert "MarketStreamReconnecting" in repr(reconnect_a)
