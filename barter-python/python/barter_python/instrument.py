"""Pure Python implementation of barter-instrument data structures."""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, Optional, TypeVar, Union

AssetKey = TypeVar("AssetKey")


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


class AssetNameInternal:
    """Barter lowercase string representation for an Asset."""

    def __init__(self, name: str) -> None:
        self._name = name.lower()

    @property
    def name(self) -> str:
        return self._name

    def __str__(self) -> str:
        return self._name

    def __repr__(self) -> str:
        return f"AssetNameInternal({self._name!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetNameInternal):
            return NotImplemented
        return self._name == other._name

    def __hash__(self) -> int:
        return hash(self._name)


class AssetNameExchange:
    """Exchange string representation for an Asset."""

    def __init__(self, name: str) -> None:
        self._name = name

    @property
    def name(self) -> str:
        return self._name

    def __str__(self) -> str:
        return self._name

    def __repr__(self) -> str:
        return f"AssetNameExchange({self._name!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetNameExchange):
            return NotImplemented
        return self._name == other._name

    def __hash__(self) -> int:
        return hash(self._name)


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


class InstrumentNameInternal:
    """Barter lowercase string representation for an Instrument."""

    def __init__(self, name: str) -> None:
        self._name = name.lower()

    @property
    def name(self) -> str:
        return self._name

    @classmethod
    def new_from_exchange(cls, exchange: ExchangeId, name_exchange: str | InstrumentNameExchange) -> InstrumentNameInternal:
        """Create from exchange and exchange name."""
        name_exchange = name_exchange if isinstance(name_exchange, str) else name_exchange.name
        return cls(f"{exchange.value}-{name_exchange}")

    def __str__(self) -> str:
        return self._name

    def __repr__(self) -> str:
        return f"InstrumentNameInternal({self._name!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InstrumentNameInternal):
            return NotImplemented
        return self._name == other._name

    def __hash__(self) -> int:
        return hash(self._name)


class InstrumentNameExchange:
    """Exchange string representation for an Instrument."""

    def __init__(self, name: str) -> None:
        self._name = name

    @property
    def name(self) -> str:
        return self._name

    def __str__(self) -> str:
        return self._name

    def __repr__(self) -> str:
        return f"InstrumentNameExchange({self._name!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InstrumentNameExchange):
            return NotImplemented
        return self._name == other._name

    def __hash__(self) -> int:
        return hash(self._name)


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

    def __init__(self, contract_size: Decimal, settlement_asset: AssetKey, expiry: datetime) -> None:
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
        return hash((
            self.contract_size,
            self.settlement_asset,
            self.kind,
            self.exercise,
            self.expiry,
            self.strike,
        ))


InstrumentKindType = Union[
    type(...),  # For Spot (no args)
    PerpetualContract[AssetKey],
    FutureContract[AssetKey],
    OptionContract[AssetKey],
]


class InstrumentKind(Generic[AssetKey]):
    """Instrument kind enum."""

    def __init__(self, kind: str, data: Optional[InstrumentKindType[AssetKey]] = None) -> None:
        self._kind = kind
        self._data = data

    @classmethod
    def spot(cls) -> InstrumentKind[AssetKey]:
        return cls("spot")

    @classmethod
    def perpetual(cls, contract: PerpetualContract[AssetKey]) -> InstrumentKind[AssetKey]:
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
    def data(self) -> Optional[InstrumentKindType[AssetKey]]:
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

    def settlement_asset(self) -> Optional[AssetKey]:
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
AssetKey = TypeVar("AssetKey")


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
        spec: Optional[object] = None,  # TODO: Add spec structures later
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
        spec: Optional[object] = None,
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
        return hash((
            self.exchange,
            self.name_internal,
            self.name_exchange,
            self.underlying,
            self.quote,
            self.kind,
            self.spec,
        ))


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
    type(...),  # For Spot (no args)
    MarketDataFutureContract,
    MarketDataOptionContract,
]


class MarketDataInstrumentKind:
    """Instrument kind enum for market data."""

    def __init__(self, kind: str, data: Optional[MarketDataInstrumentKindType] = None) -> None:
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
    def data(self) -> Optional[MarketDataInstrumentKindType]:
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
        self.base = base if isinstance(base, AssetNameInternal) else AssetNameInternal(base)
        self.quote = quote if isinstance(quote, AssetNameInternal) else AssetNameInternal(quote)
        self.kind = kind

    @classmethod
    def new(cls, base: str | AssetNameInternal, quote: str | AssetNameInternal, kind: MarketDataInstrumentKind) -> MarketDataInstrument:
        return cls(base, quote, kind)

    def __str__(self) -> str:
        return f"{self.base}_{self.quote}_{self.kind}"

    def __repr__(self) -> str:
        return f"MarketDataInstrument(base={self.base!r}, quote={self.quote!r}, kind={self.kind!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketDataInstrument):
            return NotImplemented
        return self.base == other.base and self.quote == other.quote and self.kind == other.kind

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





