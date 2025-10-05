"""Pure Python implementation of barter-instrument data structures."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, TypeVar, Union

from .barter_python import (
    AssetNameExchange as _AssetNameExchange,
    AssetNameInternal as _AssetNameInternal,
    InstrumentNameExchange as _InstrumentNameExchange,
    InstrumentNameInternal as _InstrumentNameInternal,
)

AssetKey = TypeVar("AssetKey")

AssetNameInternal = _AssetNameInternal
AssetNameExchange = _AssetNameExchange
InstrumentNameInternal = _InstrumentNameInternal
InstrumentNameExchange = _InstrumentNameExchange


class Side(Enum):
    """Side of a trade or position - Buy or Sell."""

    BUY = "buy"
    SELL = "sell"

    def __str__(self) -> str:
        return self.value


class ExchangeId(Enum):
    """Unique identifier for an execution server."""

    OTHER = "other"
    SIMULATED = "simulated"
    MOCK = "mock"
    BINANCE_FUTURES_COIN = "binance_futures_coin"
    BINANCE_FUTURES_USD = "binance_futures_usd"
    BINANCE_OPTIONS = "binance_options"
    BINANCE_PORTFOLIO_MARGIN = "binance_portfolio_margin"
    BINANCE_SPOT = "binance_spot"
    BINANCE_US = "binance_us"
    BITAZZA = "bitazza"
    BITFINEX = "bitfinex"
    BITFLYER = "bitflyer"
    BITGET = "bitget"
    BITMART = "bitmart"
    BITMART_FUTURES_USD = "bitmart_futures_usd"
    BITMEX = "bitmex"
    BITSO = "bitso"
    BITSTAMP = "bitstamp"
    BITVAVO = "bitvavo"
    BITHUMB = "bithumb"
    BYBIT_PERPETUALS_USD = "bybit_perpetuals_usd"
    BYBIT_SPOT = "bybit_spot"
    CEXIO = "cexio"
    COINBASE = "coinbase"
    COINBASE_INTERNATIONAL = "coinbase_international"
    CRYPTOCOM = "cryptocom"
    DERIBIT = "deribit"
    GATEIO_FUTURES_BTC = "gateio_futures_btc"
    GATEIO_FUTURES_USD = "gateio_futures_usd"
    GATEIO_OPTIONS = "gateio_options"
    GATEIO_PERPETUALS_BTC = "gateio_perpetuals_btc"
    GATEIO_PERPETUALS_USD = "gateio_perpetuals_usd"
    GATEIO_SPOT = "gateio_spot"
    GEMINI = "gemini"
    HITBTC = "hitbtc"
    HTX = "htx"  # huobi alias
    KRAKEN = "kraken"
    KUCOIN = "kucoin"
    LIQUID = "liquid"
    MEXC = "mexc"
    OKX = "okx"
    POLONIEX = "poloniex"

    def __str__(self) -> str:
        return self.value


class Asset:
    """Asset data structure."""

    def __init__(
        self,
        name_internal: str | AssetNameInternal,
        name_exchange: str | AssetNameExchange,
    ) -> None:
        self.name_internal = (
            name_internal
            if isinstance(name_internal, AssetNameInternal)
            else AssetNameInternal(name_internal)
        )
        self.name_exchange = (
            name_exchange
            if isinstance(name_exchange, AssetNameExchange)
            else AssetNameExchange(name_exchange)
        )

    @classmethod
    def new_from_exchange(cls, name_exchange: str | AssetNameExchange) -> Asset:
        """Create an Asset from exchange name, using it for both internal and exchange names."""
        name_exchange = (
            name_exchange
            if isinstance(name_exchange, AssetNameExchange)
            else AssetNameExchange(name_exchange)
        )
        name_internal = AssetNameInternal(name_exchange.name)
        return cls(name_internal, name_exchange)

    def __str__(self) -> str:
        return f"{self.name_internal}"

    def __repr__(self) -> str:
        return f"Asset(name_internal={self.name_internal!r}, name_exchange={self.name_exchange!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Asset):
            return NotImplemented
        return (
            self.name_internal == other.name_internal
            and self.name_exchange == other.name_exchange
        )

    def __hash__(self) -> int:
        return hash((self.name_internal, self.name_exchange))


class Underlying(Generic[AssetKey]):
    """Instrument Underlying containing a base and quote asset."""

    def __init__(self, base: AssetKey, quote: AssetKey) -> None:
        self.base = base
        self.quote = quote

    @classmethod
    def new(cls, base: AssetKey, quote: AssetKey) -> Underlying[AssetKey]:
        return cls(base, quote)

    def __str__(self) -> str:
        return f"{self.base}_{self.quote}"

    def __repr__(self) -> str:
        return f"Underlying(base={self.base!r}, quote={self.quote!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Underlying):
            return NotImplemented
        return self.base == other.base and self.quote == other.quote

    def __hash__(self) -> int:
        return hash((self.base, self.quote))


KeyType = TypeVar("KeyType")
ValueType = TypeVar("ValueType")


class Keyed(Generic[KeyType, ValueType]):
    """A keyed value."""

    def __init__(self, key: KeyType, value: ValueType) -> None:
        self.key = key
        self.value = value

    @classmethod
    def new(cls, key: KeyType, value: ValueType) -> Keyed[KeyType, ValueType]:
        return cls(key, value)

    def __str__(self) -> str:
        return f"{self.key}, {self.value}"

    def __repr__(self) -> str:
        return f"Keyed(key={self.key!r}, value={self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Keyed):
            return NotImplemented
        return self.key == other.key and self.value == other.value

    def __hash__(self) -> int:
        return hash((self.key, self.value))


class InstrumentQuoteAsset(Enum):
    """Instrument quote asset."""

    UNDERLYING_BASE = "underlying_base"
    UNDERLYING_QUOTE = "underlying_quote"

    def __str__(self) -> str:
        return self.value


class QuoteAsset:
    """Special type that represents a quote asset."""

    def __init__(self) -> None:
        pass

    def __str__(self) -> str:
        return "QuoteAsset"

    def __repr__(self) -> str:
        return "QuoteAsset()"

    def __eq__(self, other: object) -> bool:
        return isinstance(other, QuoteAsset)

    def __hash__(self) -> int:
        return hash("QuoteAsset")


class OptionKind(Enum):
    """Option contract kind - Put or Call."""

    CALL = "call"
    PUT = "put"

    def __str__(self) -> str:
        return self.value


class OptionExercise(Enum):
    """Option contract exercise style."""

    AMERICAN = "american"
    BERMUDAN = "bermudan"
    EUROPEAN = "european"

    def __str__(self) -> str:
        return self.value


class PerpetualContract(Generic[AssetKey]):
    """Perpetual contract specification."""

    def __init__(self, contract_size: Decimal, settlement_asset: AssetKey) -> None:
        self.contract_size = contract_size
        self.settlement_asset = settlement_asset

    def __repr__(self) -> str:
        return f"PerpetualContract(contract_size={self.contract_size!r}, settlement_asset={self.settlement_asset!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, PerpetualContract):
            return NotImplemented
        return (
            self.contract_size == other.contract_size
            and self.settlement_asset == other.settlement_asset
        )

    def __hash__(self) -> int:
        return hash((self.contract_size, self.settlement_asset))


class FutureContract(Generic[AssetKey]):
    """Future contract specification."""

    def __init__(
        self, contract_size: Decimal, settlement_asset: AssetKey, expiry: datetime
    ) -> None:
        self.contract_size = contract_size
        self.settlement_asset = settlement_asset
        self.expiry = expiry

    def __repr__(self) -> str:
        return f"FutureContract(contract_size={self.contract_size!r}, settlement_asset={self.settlement_asset!r}, expiry={self.expiry!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, FutureContract):
            return NotImplemented
        return (
            self.contract_size == other.contract_size
            and self.settlement_asset == other.settlement_asset
            and self.expiry == other.expiry
        )

    def __hash__(self) -> int:
        return hash((self.contract_size, self.settlement_asset, self.expiry))


class OptionContract(Generic[AssetKey]):
    """Option contract specification."""

    def __init__(
        self,
        contract_size: Decimal,
        settlement_asset: AssetKey,
        kind: OptionKind,
        exercise: OptionExercise,
        expiry: datetime,
        strike: Decimal,
    ) -> None:
        self.contract_size = contract_size
        self.settlement_asset = settlement_asset
        self.kind = kind
        self.exercise = exercise
        self.expiry = expiry
        self.strike = strike

    def __repr__(self) -> str:
        return (
            f"OptionContract("
            f"contract_size={self.contract_size!r}, "
            f"settlement_asset={self.settlement_asset!r}, "
            f"kind={self.kind!r}, "
            f"exercise={self.exercise!r}, "
            f"expiry={self.expiry!r}, "
            f"strike={self.strike!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OptionContract):
            return NotImplemented
        return (
            self.contract_size == other.contract_size
            and self.settlement_asset == other.settlement_asset
            and self.kind == other.kind
            and self.exercise == other.exercise
            and self.expiry == other.expiry
            and self.strike == other.strike
        )

    def __hash__(self) -> int:
        return hash(
            (
                self.contract_size,
                self.settlement_asset,
                self.kind,
                self.exercise,
                self.expiry,
                self.strike,
            )
        )


InstrumentKindType = Union[
    None,  # For Spot (no args)
    PerpetualContract[AssetKey],
    FutureContract[AssetKey],
    OptionContract[AssetKey],
]


class InstrumentKind(Generic[AssetKey]):
    """Instrument kind enum."""

    def __init__(
        self, kind: str, data: InstrumentKindType[AssetKey] | None = None
    ) -> None:
        self._kind = kind
        self._data = data

    @classmethod
    def spot(cls) -> InstrumentKind[AssetKey]:
        return cls("spot")

    @classmethod
    def perpetual(
        cls, contract: PerpetualContract[AssetKey]
    ) -> InstrumentKind[AssetKey]:
        return cls("perpetual", contract)

    @classmethod
    def future(cls, contract: FutureContract[AssetKey]) -> InstrumentKind[AssetKey]:
        return cls("future", contract)

    @classmethod
    def option(cls, contract: OptionContract[AssetKey]) -> InstrumentKind[AssetKey]:
        return cls("option", contract)

    @property
    def kind(self) -> str:
        return self._kind

    @property
    def data(self) -> InstrumentKindType[AssetKey] | None:
        return self._data

    def contract_size(self) -> Decimal:
        """Returns the contract size."""
        if self._kind == "spot":
            return Decimal("1")
        elif self._kind == "perpetual":
            return self._data.contract_size  # type: ignore
        elif self._kind == "future":
            return self._data.contract_size  # type: ignore
        elif self._kind == "option":
            return self._data.contract_size  # type: ignore
        else:
            raise ValueError(f"Unknown instrument kind: {self._kind}")

    def settlement_asset(self) -> AssetKey | None:
        """Returns the settlement asset if applicable."""
        if self._kind == "spot":
            return None
        elif self._kind in ("perpetual", "future", "option"):
            return self._data.settlement_asset  # type: ignore
        else:
            raise ValueError(f"Unknown instrument kind: {self._kind}")

    def __repr__(self) -> str:
        if self._data is None:
            return f"InstrumentKind.{self._kind}()"
        else:
            return f"InstrumentKind.{self._kind}({self._data!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InstrumentKind):
            return NotImplemented
        return self._kind == other._kind and self._data == other._data

    def __hash__(self) -> int:
        return hash((self._kind, self._data))


ExchangeKey = TypeVar("ExchangeKey")


class Instrument(Generic[AssetKey]):
    """Comprehensive Instrument model."""

    def __init__(
        self,
        exchange: ExchangeId,
        name_internal: str | InstrumentNameInternal,
        name_exchange: str | InstrumentNameExchange,
        underlying: Underlying[AssetKey],
        quote: InstrumentQuoteAsset,
        kind: InstrumentKind[AssetKey],
        spec: object | None = None,  # Structured specs available via Rust bindings
    ) -> None:
        self.exchange = exchange
        self.name_internal = (
            name_internal
            if isinstance(name_internal, InstrumentNameInternal)
            else InstrumentNameInternal(name_internal)
        )
        self.name_exchange = (
            name_exchange
            if isinstance(name_exchange, InstrumentNameExchange)
            else InstrumentNameExchange(name_exchange)
        )
        self.underlying = underlying
        self.quote = quote
        self.kind = kind
        self.spec = spec

    @classmethod
    def spot(
        cls,
        exchange: ExchangeId,
        name_internal: str | InstrumentNameInternal,
        name_exchange: str | InstrumentNameExchange,
        underlying: Underlying[AssetKey],
        spec: object | None = None,
    ) -> Instrument[AssetKey]:
        """Create a spot instrument."""
        return cls(
            exchange=exchange,
            name_internal=name_internal,
            name_exchange=name_exchange,
            underlying=underlying,
            quote=InstrumentQuoteAsset.UNDERLYING_QUOTE,
            kind=InstrumentKind.spot(),
            spec=spec,
        )

    def map_exchange_key(self, new_exchange: ExchangeId) -> Instrument[AssetKey]:
        """Map to a new exchange key."""
        return Instrument(
            exchange=new_exchange,
            name_internal=self.name_internal,
            name_exchange=self.name_exchange,
            underlying=self.underlying,
            quote=self.quote,
            kind=self.kind,
            spec=self.spec,
        )

    def __repr__(self) -> str:
        return (
            f"Instrument("
            f"exchange={self.exchange!r}, "
            f"name_internal={self.name_internal!r}, "
            f"name_exchange={self.name_exchange!r}, "
            f"underlying={self.underlying!r}, "
            f"quote={self.quote!r}, "
            f"kind={self.kind!r}, "
            f"spec={self.spec!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Instrument):
            return NotImplemented
        return (
            self.exchange == other.exchange
            and self.name_internal == other.name_internal
            and self.name_exchange == other.name_exchange
            and self.underlying == other.underlying
            and self.quote == other.quote
            and self.kind == other.kind
            and self.spec == other.spec
        )

    def __hash__(self) -> int:
        return hash(
            (
                self.exchange,
                self.name_internal,
                self.name_exchange,
                self.underlying,
                self.quote,
                self.kind,
                self.spec,
            )
        )


class MarketDataFutureContract:
    """Future contract specification for market data."""

    def __init__(self, expiry: datetime) -> None:
        self.expiry = expiry

    def __repr__(self) -> str:
        return f"MarketDataFutureContract(expiry={self.expiry!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketDataFutureContract):
            return NotImplemented
        return self.expiry == other.expiry

    def __hash__(self) -> int:
        return hash(self.expiry)


class MarketDataOptionContract:
    """Option contract specification for market data."""

    def __init__(
        self,
        kind: OptionKind,
        exercise: OptionExercise,
        expiry: datetime,
        strike: Decimal,
    ) -> None:
        self.kind = kind
        self.exercise = exercise
        self.expiry = expiry
        self.strike = strike

    def __repr__(self) -> str:
        return (
            f"MarketDataOptionContract("
            f"kind={self.kind!r}, "
            f"exercise={self.exercise!r}, "
            f"expiry={self.expiry!r}, "
            f"strike={self.strike!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketDataOptionContract):
            return NotImplemented
        return (
            self.kind == other.kind
            and self.exercise == other.exercise
            and self.expiry == other.expiry
            and self.strike == other.strike
        )

    def __hash__(self) -> int:
        return hash((self.kind, self.exercise, self.expiry, self.strike))


MarketDataInstrumentKindType = Union[
    None,  # For Spot (no args)
    MarketDataFutureContract,
    MarketDataOptionContract,
]


class MarketDataInstrumentKind:
    """Instrument kind enum for market data."""

    def __init__(
        self, kind: str, data: MarketDataInstrumentKindType | None = None
    ) -> None:
        self._kind = kind
        self._data = data

    @classmethod
    def spot(cls) -> MarketDataInstrumentKind:
        return cls("spot")

    @classmethod
    def perpetual(cls) -> MarketDataInstrumentKind:
        return cls("perpetual")

    @classmethod
    def future(cls, contract: MarketDataFutureContract) -> MarketDataInstrumentKind:
        return cls("future", contract)

    @classmethod
    def option(cls, contract: MarketDataOptionContract) -> MarketDataInstrumentKind:
        return cls("option", contract)

    @property
    def kind(self) -> str:
        return self._kind

    @property
    def data(self) -> MarketDataInstrumentKindType | None:
        return self._data

    def __repr__(self) -> str:
        if self._data is None:
            return f"MarketDataInstrumentKind.{self._kind}()"
        else:
            return f"MarketDataInstrumentKind.{self._kind}({self._data!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketDataInstrumentKind):
            return NotImplemented
        return self._kind == other._kind and self._data == other._data

    def __hash__(self) -> int:
        return hash((self._kind, self._data))

    def __str__(self) -> str:
        if self._kind == "spot":
            return "spot"
        elif self._kind == "perpetual":
            return "perpetual"
        elif self._kind == "future":
            return f"future_{self._data.expiry.date()}-UTC"  # type: ignore
        elif self._kind == "option":
            return f"option_{self._data.kind}_{self._data.exercise}_{self._data.expiry.date()}-UTC_{self._data.strike}"  # type: ignore
        else:
            return self._kind


class MarketDataInstrument:
    """Barter representation of a MarketDataInstrument."""

    def __init__(
        self,
        base: str | AssetNameInternal,
        quote: str | AssetNameInternal,
        kind: MarketDataInstrumentKind,
    ) -> None:
        self.base = (
            base if isinstance(base, AssetNameInternal) else AssetNameInternal(base)
        )
        self.quote = (
            quote if isinstance(quote, AssetNameInternal) else AssetNameInternal(quote)
        )
        self.kind = kind

    @classmethod
    def new(
        cls,
        base: str | AssetNameInternal,
        quote: str | AssetNameInternal,
        kind: MarketDataInstrumentKind,
    ) -> MarketDataInstrument:
        return cls(base, quote, kind)

    def __str__(self) -> str:
        return f"{self.base}_{self.quote}_{self.kind}"

    def __repr__(self) -> str:
        return f"MarketDataInstrument(base={self.base!r}, quote={self.quote!r}, kind={self.kind!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketDataInstrument):
            return NotImplemented
        return (
            self.base == other.base
            and self.quote == other.quote
            and self.kind == other.kind
        )

    def __hash__(self) -> int:
        return hash((self.base, self.quote, self.kind))


class ExchangeIndex:
    """Index for an exchange in IndexedInstruments."""

    def __init__(self, index: int) -> None:
        self._index = index

    @property
    def index(self) -> int:
        return self._index

    @classmethod
    def new(cls, index: int) -> ExchangeIndex:
        return cls(index)

    def __repr__(self) -> str:
        return f"ExchangeIndex({self._index})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ExchangeIndex):
            return NotImplemented
        return self._index == other._index

    def __hash__(self) -> int:
        return hash(self._index)


class AssetIndex:
    """Index for an asset in IndexedInstruments."""

    def __init__(self, index: int) -> None:
        self._index = index

    @property
    def index(self) -> int:
        return self._index

    @classmethod
    def new(cls, index: int) -> AssetIndex:
        return cls(index)

    def __repr__(self) -> str:
        return f"AssetIndex({self._index})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetIndex):
            return NotImplemented
        return self._index == other._index

    def __hash__(self) -> int:
        return hash(self._index)


class InstrumentIndex:
    """Index for an instrument in IndexedInstruments."""

    def __init__(self, index: int) -> None:
        self._index = index

    @property
    def index(self) -> int:
        return self._index

    @classmethod
    def new(cls, index: int) -> InstrumentIndex:
        return cls(index)

    def __repr__(self) -> str:
        return f"InstrumentIndex({self._index})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InstrumentIndex):
            return NotImplemented
        return self._index == other._index

    def __hash__(self) -> int:
        return hash(self._index)


class ExchangeAsset:
    """Asset associated with a specific exchange."""

    def __init__(self, exchange: ExchangeId, asset: Asset) -> None:
        self.exchange = exchange
        self.asset = asset

    @classmethod
    def new(cls, exchange: ExchangeId, asset: Asset) -> ExchangeAsset:
        return cls(exchange, asset)

    def __repr__(self) -> str:
        return f"ExchangeAsset(exchange={self.exchange!r}, asset={self.asset!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ExchangeAsset):
            return NotImplemented
        return self.exchange == other.exchange and self.asset == other.asset

    def __hash__(self) -> int:
        return hash((self.exchange, self.asset))


@dataclass(frozen=True)
class IndexedInstrument:
    """Indexed representation of an instrument linking to exchange and asset indices."""

    exchange: Keyed[ExchangeIndex, ExchangeId]
    name_internal: InstrumentNameInternal
    name_exchange: InstrumentNameExchange
    underlying: Underlying[AssetIndex]
    quote: InstrumentQuoteAsset
    kind: InstrumentKind
    spec: object | None = None


class IndexedInstruments:
    """Indexed collection of exchanges, assets, and instruments."""

    def __init__(
        self,
        exchanges: list[Keyed[ExchangeIndex, ExchangeId]],
        assets: list[Keyed[AssetIndex, ExchangeAsset]],
        instruments: list[Keyed[InstrumentIndex, IndexedInstrument]],
        exchange_lookup: dict[ExchangeId, ExchangeIndex],
        asset_lookup: dict[tuple[ExchangeId, AssetNameInternal], AssetIndex],
        instrument_lookup: dict[tuple[ExchangeId, InstrumentNameInternal], InstrumentIndex],
    ) -> None:
        self._exchanges = exchanges
        self._assets = assets
        self._instruments = instruments
        self._exchange_lookup = exchange_lookup
        self._asset_lookup = asset_lookup
        self._instrument_lookup = instrument_lookup
        self._asset_by_index = {entry.key: entry.value for entry in assets}
        self._instrument_by_index = {entry.key: entry.value for entry in instruments}

    @classmethod
    def new(cls, instruments: Iterable[Instrument]) -> IndexedInstruments:
        """Construct `IndexedInstruments` from an iterable of `Instrument` objects."""
        builder = cls.builder()
        for instrument in instruments:
            builder.add_instrument(instrument)
        return builder.build()

    @classmethod
    def builder(cls) -> IndexedInstrumentsBuilder:
        """Return a builder for incremental construction."""
        return IndexedInstrumentsBuilder()

    def exchanges(self) -> list[Keyed[ExchangeIndex, ExchangeId]]:
        """Return the indexed exchanges."""
        return list(self._exchanges)

    def assets(self) -> list[Keyed[AssetIndex, ExchangeAsset]]:
        """Return the indexed assets."""
        return list(self._assets)

    def instruments(self) -> list[Keyed[InstrumentIndex, IndexedInstrument]]:
        """Return the indexed instruments."""
        return list(self._instruments)

    def find_exchange_index(self, exchange: ExchangeId) -> ExchangeIndex:
        """Find the index associated with an `ExchangeId`."""
        try:
            return self._exchange_lookup[exchange]
        except KeyError as exc:  # pragma: no cover - defensive
            raise ValueError(
                f"ExchangeId {exchange} is not present in indexed exchanges"
            ) from exc

    def find_exchange(self, index: ExchangeIndex) -> ExchangeId:
        """Find the `ExchangeId` for an `ExchangeIndex`."""
        for entry in self._exchanges:
            if entry.key == index:
                return entry.value
        raise ValueError(f"ExchangeIndex {index} is not present in indexed exchanges")

    def find_asset_index(
        self, exchange: ExchangeId, name_internal: AssetNameInternal | str
    ) -> AssetIndex:
        """Find the asset index for an exchange and internal asset name."""
        if isinstance(name_internal, str):
            name_internal = AssetNameInternal(name_internal)
        key = (exchange, name_internal)
        try:
            return self._asset_lookup[key]
        except KeyError as exc:
            raise ValueError(
                f"Asset ({exchange}, {name_internal}) not present in indexed assets"
            ) from exc

    def find_asset(self, index: AssetIndex) -> ExchangeAsset:
        """Find the exchange asset for an asset index."""
        try:
            return self._asset_by_index[index]
        except KeyError as exc:
            raise ValueError(
                f"AssetIndex {index} is not present in indexed assets"
            ) from exc

    def find_instrument_index(
        self, exchange: ExchangeId, name_internal: InstrumentNameInternal | str
    ) -> InstrumentIndex:
        """Find the instrument index for an exchange and internal instrument name."""
        if isinstance(name_internal, str):
            name_internal = InstrumentNameInternal(name_internal)
        key = (exchange, name_internal)
        try:
            return self._instrument_lookup[key]
        except KeyError as exc:
            raise ValueError(
                f"Instrument ({exchange}, {name_internal}) not present in indexed instruments"
            ) from exc

    def find_instrument(self, index: InstrumentIndex) -> IndexedInstrument:
        """Find the indexed instrument for an instrument index."""
        try:
            return self._instrument_by_index[index]
        except KeyError as exc:
            raise ValueError(
                f"InstrumentIndex {index} is not present in indexed instruments"
            ) from exc


class IndexedInstrumentsBuilder:
    """Builder for constructing `IndexedInstruments` incrementally."""

    def __init__(self) -> None:
        self._exchanges: list[Keyed[ExchangeIndex, ExchangeId]] = []
        self._exchanges_by_id: dict[ExchangeId, ExchangeIndex] = {}
        self._assets: list[Keyed[AssetIndex, ExchangeAsset]] = []
        self._assets_by_key: dict[tuple[ExchangeId, AssetNameInternal], AssetIndex] = {}
        self._instruments: list[Keyed[InstrumentIndex, IndexedInstrument]] = []
        self._instruments_by_key: dict[
            tuple[ExchangeId, InstrumentNameInternal], InstrumentIndex
        ] = {}

    def add_instrument(self, instrument: Instrument) -> IndexedInstrumentsBuilder:
        """Add an instrument to the builder."""
        if not isinstance(instrument, Instrument):  # pragma: no cover - defensive
            raise TypeError("IndexedInstrumentsBuilder only accepts Instrument instances")

        exchange_index = self._ensure_exchange(instrument.exchange)
        base_index = self._ensure_asset(instrument.exchange, instrument.underlying.base)
        quote_index = self._ensure_asset(instrument.exchange, instrument.underlying.quote)
        underlying_indices = Underlying(base_index, quote_index)
        kind_indices = self._convert_kind(instrument.exchange, instrument.kind)

        instrument_index = InstrumentIndex.new(len(self._instruments))
        record = IndexedInstrument(
            exchange=Keyed(exchange_index, instrument.exchange),
            name_internal=instrument.name_internal,
            name_exchange=instrument.name_exchange,
            underlying=underlying_indices,
            quote=instrument.quote,
            kind=kind_indices,
            spec=instrument.spec,
        )

        keyed_instrument = Keyed(instrument_index, record)
        self._instruments.append(keyed_instrument)
        self._instruments_by_key[(instrument.exchange, instrument.name_internal)] = (
            instrument_index
        )

        return self

    def build(self) -> IndexedInstruments:
        """Create an immutable `IndexedInstruments` instance from the builder state."""
        return IndexedInstruments(
            list(self._exchanges),
            list(self._assets),
            list(self._instruments),
            dict(self._exchanges_by_id),
            dict(self._assets_by_key),
            dict(self._instruments_by_key),
        )

    def _ensure_exchange(self, exchange: ExchangeId) -> ExchangeIndex:
        if exchange in self._exchanges_by_id:
            return self._exchanges_by_id[exchange]

        exchange_index = ExchangeIndex.new(len(self._exchanges))
        keyed_exchange = Keyed(exchange_index, exchange)
        self._exchanges.append(keyed_exchange)
        self._exchanges_by_id[exchange] = exchange_index
        return exchange_index

    def _ensure_asset(self, exchange: ExchangeId, asset: Asset) -> AssetIndex:
        key = (exchange, asset.name_internal)
        if key in self._assets_by_key:
            return self._assets_by_key[key]

        asset_index = AssetIndex.new(len(self._assets))
        asset_copy = Asset(asset.name_internal, asset.name_exchange)
        keyed_asset = Keyed(asset_index, ExchangeAsset.new(exchange, asset_copy))
        self._assets.append(keyed_asset)
        self._assets_by_key[key] = asset_index
        return asset_index

    def _convert_kind(self, exchange: ExchangeId, kind: InstrumentKind) -> InstrumentKind:
        kind_name = kind.kind
        if kind_name == "spot":
            return InstrumentKind.spot()

        if kind_name == "perpetual":
            contract = kind.data
            if not isinstance(contract, PerpetualContract):
                raise ValueError("Perpetual instrument missing contract data")
            settlement_index = self._ensure_asset(exchange, contract.settlement_asset)
            return InstrumentKind.perpetual(
                PerpetualContract(contract.contract_size, settlement_index)
            )

        if kind_name == "future":
            contract = kind.data
            if not isinstance(contract, FutureContract):
                raise ValueError("Future instrument missing contract data")
            settlement_index = self._ensure_asset(exchange, contract.settlement_asset)
            return InstrumentKind.future(
                FutureContract(
                    contract.contract_size,
                    settlement_index,
                    contract.expiry,
                )
            )

        if kind_name == "option":
            contract = kind.data
            if not isinstance(contract, OptionContract):
                raise ValueError("Option instrument missing contract data")
            settlement_index = self._ensure_asset(exchange, contract.settlement_asset)
            return InstrumentKind.option(
                OptionContract(
                    contract.contract_size,
                    settlement_index,
                    contract.kind,
                    contract.exercise,
                    contract.expiry,
                    contract.strike,
                )
            )

        raise ValueError(f"Unsupported instrument kind: {kind.kind}")
