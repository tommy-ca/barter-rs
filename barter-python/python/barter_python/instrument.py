"""Pure Python implementation of barter-instrument data structures."""

from __future__ import annotations

from enum import Enum
from typing import Generic, TypeVar

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