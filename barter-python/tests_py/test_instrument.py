"""Unit tests for pure Python instrument data structures."""

from datetime import datetime, timezone
from decimal import Decimal

import barter_python as bp
from barter_python.instrument import (
    Asset,
    AssetNameExchange,
    AssetNameInternal,
    ExchangeId,
    FutureContract,
    Instrument,
    InstrumentKind,
    InstrumentNameExchange,
    InstrumentNameInternal,
    InstrumentQuoteAsset,
    Keyed,
    OptionContract,
    OptionExercise,
    OptionKind,
    PerpetualContract,
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


class TestInstrumentNameInternal:
    def test_creation(self):
        name = InstrumentNameInternal("BTC_USDT")
        assert name.name == "btc_usdt"

    def test_new_from_exchange(self):
        name = InstrumentNameInternal.new_from_exchange(
            ExchangeId.BINANCE_SPOT, "BTCUSDT"
        )
        assert name.name == "binance_spot-btcusdt"

    def test_equality(self):
        name1 = InstrumentNameInternal("btc_usdt")
        name2 = InstrumentNameInternal("BTC_USDT")
        name3 = InstrumentNameInternal("eth_usdt")
        assert name1 == name2
        assert name1 != name3

    def test_str_repr(self):
        name = InstrumentNameInternal("BTC_USDT")
        assert str(name) == "btc_usdt"
        assert repr(name) == "InstrumentNameInternal('btc_usdt')"


class TestInstrumentNameExchange:
    def test_creation(self):
        name = InstrumentNameExchange("BTCUSDT")
        assert name.name == "BTCUSDT"

    def test_equality(self):
        name1 = InstrumentNameExchange("BTCUSDT")
        name2 = InstrumentNameExchange("BTCUSDT")
        name3 = InstrumentNameExchange("ETHUSDT")
        assert name1 == name2
        assert name1 != name3

    def test_str_repr(self):
        name = InstrumentNameExchange("BTCUSDT")
        assert str(name) == "BTCUSDT"
        assert repr(name) == "InstrumentNameExchange('BTCUSDT')"


class TestInstrumentQuoteAsset:
    def test_enum_values(self):
        assert InstrumentQuoteAsset.UNDERLYING_BASE.value == "underlying_base"
        assert InstrumentQuoteAsset.UNDERLYING_QUOTE.value == "underlying_quote"

    def test_str(self):
        assert str(InstrumentQuoteAsset.UNDERLYING_QUOTE) == "underlying_quote"


class TestOptionKind:
    def test_enum_values(self):
        assert OptionKind.CALL.value == "call"
        assert OptionKind.PUT.value == "put"

    def test_str(self):
        assert str(OptionKind.CALL) == "call"


class TestOptionExercise:
    def test_enum_values(self):
        assert OptionExercise.AMERICAN.value == "american"
        assert OptionExercise.EUROPEAN.value == "european"

    def test_str(self):
        assert str(OptionExercise.AMERICAN) == "american"


class TestPerpetualContract:
    def test_creation(self):
        contract = PerpetualContract(Decimal("1"), "usdt")
        assert contract.contract_size == Decimal("1")
        assert contract.settlement_asset == "usdt"

    def test_equality(self):
        c1 = PerpetualContract(Decimal("1"), "usdt")
        c2 = PerpetualContract(Decimal("1"), "usdt")
        c3 = PerpetualContract(Decimal("2"), "usdt")
        assert c1 == c2
        assert c1 != c3

    def test_repr(self):
        contract = PerpetualContract(Decimal("1"), "usdt")
        assert "PerpetualContract(" in repr(contract)


class TestFutureContract:
    def test_creation(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = FutureContract(Decimal("1"), "usdt", expiry)
        assert contract.contract_size == Decimal("1")
        assert contract.settlement_asset == "usdt"
        assert contract.expiry == expiry

    def test_equality(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        c1 = FutureContract(Decimal("1"), "usdt", expiry)
        c2 = FutureContract(Decimal("1"), "usdt", expiry)
        c3 = FutureContract(Decimal("2"), "usdt", expiry)
        assert c1 == c2
        assert c1 != c3

    def test_repr(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = FutureContract(Decimal("1"), "usdt", expiry)
        assert "FutureContract(" in repr(contract)


class TestOptionContract:
    def test_creation(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.CALL,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        assert contract.contract_size == Decimal("1")
        assert contract.settlement_asset == "usdt"
        assert contract.kind == OptionKind.CALL
        assert contract.exercise == OptionExercise.AMERICAN
        assert contract.expiry == expiry
        assert contract.strike == Decimal("50000")

    def test_equality(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        c1 = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.CALL,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        c2 = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.CALL,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        c3 = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.PUT,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        assert c1 == c2
        assert c1 != c3

    def test_repr(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.CALL,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        assert "OptionContract(" in repr(contract)


class TestInstrumentKind:
    def test_spot(self):
        kind = InstrumentKind.spot()
        assert kind.kind == "spot"
        assert kind.data is None
        assert kind.contract_size() == Decimal("1")
        assert kind.settlement_asset() is None

    def test_perpetual(self):
        contract = PerpetualContract(Decimal("1"), "usdt")
        kind = InstrumentKind.perpetual(contract)
        assert kind.kind == "perpetual"
        assert kind.data == contract
        assert kind.contract_size() == Decimal("1")
        assert kind.settlement_asset() == "usdt"

    def test_future(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = FutureContract(Decimal("1"), "usdt", expiry)
        kind = InstrumentKind.future(contract)
        assert kind.kind == "future"
        assert kind.data == contract
        assert kind.contract_size() == Decimal("1")
        assert kind.settlement_asset() == "usdt"

    def test_option(self):
        expiry = datetime(2025, 12, 31, tzinfo=timezone.utc)
        contract = OptionContract(
            Decimal("1"),
            "usdt",
            OptionKind.CALL,
            OptionExercise.AMERICAN,
            expiry,
            Decimal("50000"),
        )
        kind = InstrumentKind.option(contract)
        assert kind.kind == "option"
        assert kind.data == contract
        assert kind.contract_size() == Decimal("1")
        assert kind.settlement_asset() == "usdt"

    def test_equality(self):
        k1 = InstrumentKind.spot()
        k2 = InstrumentKind.spot()
        contract = PerpetualContract(Decimal("1"), "usdt")
        k3 = InstrumentKind.perpetual(contract)
        assert k1 == k2
        assert k1 != k3


class TestInstrument:
    def test_creation(self):
        underlying = Underlying("btc", "usdt")
        instrument = Instrument(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
            quote=InstrumentQuoteAsset.UNDERLYING_QUOTE,
            kind=InstrumentKind.spot(),
        )
        assert instrument.exchange == ExchangeId.BINANCE_SPOT
        assert instrument.name_internal.name == "binance_spot-btcusdt"
        assert instrument.name_exchange.name == "BTCUSDT"
        assert instrument.underlying == underlying
        assert instrument.quote == InstrumentQuoteAsset.UNDERLYING_QUOTE
        assert instrument.kind.kind == "spot"

    def test_spot_classmethod(self):
        underlying = Underlying("btc", "usdt")
        instrument = Instrument.spot(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
        )
        assert instrument.quote == InstrumentQuoteAsset.UNDERLYING_QUOTE
        assert instrument.kind.kind == "spot"

    def test_map_exchange_key(self):
        underlying = Underlying("btc", "usdt")
        instrument = Instrument.spot(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
        )
        new_instrument = instrument.map_exchange_key(ExchangeId.KRAKEN)
        assert new_instrument.exchange == ExchangeId.KRAKEN
        assert new_instrument.name_internal == instrument.name_internal

    def test_equality(self):
        underlying = Underlying("btc", "usdt")
        i1 = Instrument.spot(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
        )
        i2 = Instrument.spot(
            exchange=ExchangeId.BINANCE_SPOT,
            name_internal="binance_spot-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
        )
        i3 = Instrument.spot(
            exchange=ExchangeId.KRAKEN,
            name_internal="kraken-btcusdt",
            name_exchange="BTCUSDT",
            underlying=underlying,
        )
        assert i1 == i2
        assert i1 != i3


class TestPyAsset:
    def test_creation(self):
        asset = bp.Asset("btc", "XBT")
        assert asset.name_internal == "btc"
        assert asset.name_exchange == "XBT"

    def test_from_exchange_name(self):
        asset = bp.Asset.from_exchange_name("XBT")
        assert asset.name_internal == "xbt"
        assert asset.name_exchange == "XBT"

    def test_str_repr(self):
        asset = bp.Asset("btc", "XBT")
        assert str(asset) == "Asset(name_internal='btc', name_exchange='XBT')"
        assert "Asset(" in repr(asset)


class TestPySide:
    def test_side_enum_values(self):
        assert bp.Side.BUY == bp.Side.BUY
        assert bp.Side.SELL == bp.Side.SELL
        assert bp.Side.BUY != bp.Side.SELL

    def test_side_str(self):
        assert str(bp.Side.BUY) == "buy"
        assert str(bp.Side.SELL) == "sell"

    def test_side_repr(self):
        assert repr(bp.Side.BUY) == "Side.Buy"
        assert repr(bp.Side.SELL) == "Side.Sell"


class TestPyAssetIndex:
    def test_creation(self):
        index = bp.AssetIndex(42)
        assert index.index == 42

    def test_equality(self):
        i1 = bp.AssetIndex(1)
        i2 = bp.AssetIndex(1)
        i3 = bp.AssetIndex(2)
        assert i1 == i2
        assert i1 != i3

    def test_hash(self):
        i1 = bp.AssetIndex(1)
        i2 = bp.AssetIndex(1)
        assert hash(i1) == hash(i2)

    def test_str_repr(self):
        index = bp.AssetIndex(42)
        assert str(index) == "AssetIndex(42)"
        assert "AssetIndex(" in repr(index)
