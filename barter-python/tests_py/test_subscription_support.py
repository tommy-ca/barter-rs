from __future__ import annotations

import datetime as dt

import barter_python as bp

UTC = dt.timezone.utc


def _future_kind(expiry: dt.datetime | None = None) -> dict[str, object]:
    expiry = expiry or dt.datetime(2025, 1, 1, tzinfo=UTC)
    return {"type": "future", "expiry": expiry}


def test_exchange_supports_instrument_kind_spot_vs_future():
    assert bp.exchange_supports_instrument_kind(bp.ExchangeId.BINANCE_SPOT, "spot")

    assert not bp.exchange_supports_instrument_kind(
        bp.ExchangeId.BINANCE_SPOT,
        _future_kind(),
    )


def test_subscription_is_supported_matches_helper():
    spot_subscription = bp.Subscription(
        bp.ExchangeId.BINANCE_SPOT,
        "btc",
        "usdt",
        bp.SubKind.PUBLIC_TRADES,
        instrument_kind="spot",
    )

    assert spot_subscription.is_supported()

    future_subscription = bp.Subscription(
        bp.ExchangeId.BINANCE_SPOT,
        "btc",
        "usdt",
        bp.SubKind.PUBLIC_TRADES,
        instrument_kind=_future_kind(),
    )

    assert not future_subscription.is_supported()
