from datetime import datetime, timezone
from decimal import Decimal

import barter_python as bp
from barter_python.instrument import Side
from barter_python.trade_bindings import AssetFees, Trade, TradeId


def test_trade_id_wrapper_behaviour():
    tid = TradeId.new("trade-123")

    assert tid.value == "trade-123"
    assert str(tid) == "trade-123"
    assert repr(tid) == "TradeId('trade-123')"

    tid_same = TradeId.new("trade-123")
    tid_other = TradeId.new("trade-456")

    assert tid == tid_same
    assert tid != tid_other
    assert len({tid, tid_same, tid_other}) == 2


def test_asset_fees_wrapper_handles_quote_assets():
    from barter_python.instrument import QuoteAsset

    fees = AssetFees.quote_fees(Decimal("0.001"))

    assert isinstance(fees.asset, QuoteAsset)
    assert fees.fees == Decimal("0.001")

    fees_str = AssetFees("usdt", Decimal("0.002"))
    assert fees_str.asset == "usdt"
    assert fees_str.fees == Decimal("0.002")
    assert fees != fees_str


def test_trade_wrapper_properties_and_value_quote():
    trade_id = TradeId.new("trade-123")
    order_id = bp.OrderId.new("order-456")
    strategy = bp.StrategyId.new("strategy-alpha")
    time_exchange = datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
    side = Side.BUY
    price = Decimal("50000.0")
    quantity = Decimal("0.1")
    fees = AssetFees.quote_fees(Decimal("0.005"))

    trade = Trade(
        trade_id,
        order_id,
        42,
        strategy,
        time_exchange,
        side,
        price,
        quantity,
        fees,
    )

    assert trade.id == trade_id
    assert trade.order_id == order_id
    assert trade.instrument == 42
    assert trade.strategy == strategy
    assert trade.time_exchange == time_exchange
    assert trade.side == side
    assert trade.price == price
    assert trade.quantity == quantity
    assert trade.fees == fees
    assert trade.value_quote() == Decimal("5000.0")

    trade_same = Trade(
        trade_id,
        order_id,
        42,
        strategy,
        time_exchange,
        side,
        price,
        quantity,
        fees,
    )

    assert trade == trade_same
    assert hash(trade) == hash(trade_same)
