"""Unit tests for pure Python instrument data structures."""

import pytest

from barter_python.instrument import (
    Asset,
    AssetNameExchange,
    AssetNameInternal,
    ExchangeId,
    Keyed,
    Side,
    Underlying,
)


class TestSide:
    def test_side_enum_values(self):
        assert Side.BUY.value == "buy"
        assert Side.SELL.value == "sell"

    def test_side_str(self):
        assert str(Side.BUY) == "buy"
        assert str(Side.SELL) == "sell"


class TestExchangeId:
    def test_exchange_id_enum_values(self):
        assert ExchangeId.BINANCE_SPOT.value == "binance_spot"
        assert ExchangeId.KRAKEN.value == "kraken"
        assert ExchangeId.HTX.value == "htx"

    def test_exchange_id_str(self):
        assert str(ExchangeId.BINANCE_SPOT) == "binance_spot"


class TestAssetNameInternal:
    def test_creation(self):
        name = AssetNameInternal("BTC")
        assert name.name == "btc"

    def test_creation_already_lowercase(self):
        name = AssetNameInternal("btc")
        assert name.name == "btc"

    def test_equality(self):
        name1 = AssetNameInternal("BTC")
        name2 = AssetNameInternal("btc")
        name3 = AssetNameInternal("ETH")
        assert name1 == name2
        assert name1 != name3

    def test_hash(self):
        name1 = AssetNameInternal("BTC")
        name2 = AssetNameInternal("btc")
        assert hash(name1) == hash(name2)

    def test_str_repr(self):
        name = AssetNameInternal("BTC")
        assert str(name) == "btc"
        assert repr(name) == "AssetNameInternal('btc')"


class TestAssetNameExchange:
    def test_creation(self):
        name = AssetNameExchange("XBT")
        assert name.name == "XBT"

    def test_equality(self):
        name1 = AssetNameExchange("XBT")
        name2 = AssetNameExchange("XBT")
        name3 = AssetNameExchange("BTC")
        assert name1 == name2
        assert name1 != name3

    def test_str_repr(self):
        name = AssetNameExchange("XBT")
        assert str(name) == "XBT"
        assert repr(name) == "AssetNameExchange('XBT')"


class TestAsset:
    def test_creation(self):
        internal = AssetNameInternal("btc")
        exchange = AssetNameExchange("XBT")
        asset = Asset(internal, exchange)
        assert asset.name_internal == internal
        assert asset.name_exchange == exchange

    def test_creation_from_strings(self):
        asset = Asset("btc", "XBT")
        assert asset.name_internal.name == "btc"
        assert asset.name_exchange.name == "XBT"

    def test_new_from_exchange(self):
        asset = Asset.new_from_exchange("XBT")
        assert asset.name_internal.name == "xbt"
        assert asset.name_exchange.name == "XBT"

    def test_equality(self):
        asset1 = Asset("btc", "XBT")
        asset2 = Asset("btc", "XBT")
        asset3 = Asset("eth", "XBT")
        assert asset1 == asset2
        assert asset1 != asset3

    def test_str_repr(self):
        asset = Asset("btc", "XBT")
        assert str(asset) == "btc"
        assert "Asset(" in repr(asset)


class TestUnderlying:
    def test_creation(self):
        underlying = Underlying("btc", "usdt")
        assert underlying.base == "btc"
        assert underlying.quote == "usdt"

    def test_new_classmethod(self):
        underlying = Underlying.new("btc", "usdt")
        assert underlying.base == "btc"
        assert underlying.quote == "usdt"

    def test_equality(self):
        u1 = Underlying("btc", "usdt")
        u2 = Underlying("btc", "usdt")
        u3 = Underlying("eth", "usdt")
        assert u1 == u2
        assert u1 != u3

    def test_str_repr(self):
        underlying = Underlying("btc", "usdt")
        assert str(underlying) == "btc_usdt"
        assert "Underlying(" in repr(underlying)


class TestKeyed:
    def test_creation(self):
        keyed = Keyed("key", "value")
        assert keyed.key == "key"
        assert keyed.value == "value"

    def test_equality(self):
        k1 = Keyed("key", "value")
        k2 = Keyed("key", "value")
        k3 = Keyed("other", "value")
        assert k1 == k2
        assert k1 != k3

    def test_str_repr(self):
        keyed = Keyed("key", "value")
        assert str(keyed) == "key, value"
        assert "Keyed(" in repr(keyed)