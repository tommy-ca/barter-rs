"""Pure Python implementation of barter-data market event structures."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, Optional, TypeVar, Union

from .instrument import Side

InstrumentKey = TypeVar("InstrumentKey")


class PublicTrade:
    """Normalised Barter PublicTrade model."""

    def __init__(self, id: str, price: float, amount: float, side: Side) -> None:
        self.id = id
        self.price = price
        self.amount = amount
        self.side = side

    def __repr__(self) -> str:
        return (
            f"PublicTrade("
            f"id={self.id!r}, "
            f"price={self.price!r}, "
            f"amount={self.amount!r}, "
            f"side={self.side!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, PublicTrade):
            return NotImplemented
        return (
            self.id == other.id
            and self.price == other.price
            and self.amount == other.amount
            and self.side == other.side
        )

    def __hash__(self) -> int:
        return hash((self.id, self.price, self.amount, self.side))


class Level:
    """Normalised Barter OrderBook Level."""

    def __init__(self, price: Decimal, amount: Decimal) -> None:
        self.price = price
        self.amount = amount

    @classmethod
    def new(cls, price: Decimal, amount: Decimal) -> Level:
        return cls(price, amount)

    def __repr__(self) -> str:
        return f"Level(price={self.price!r}, amount={self.amount!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Level):
            return NotImplemented
        return self.price == other.price and self.amount == other.amount

    def __hash__(self) -> int:
        return hash((self.price, self.amount))


class OrderBookL1:
    """Normalised Barter OrderBookL1 snapshot containing the latest best bid and ask."""

    def __init__(
        self,
        last_update_time: datetime,
        best_bid: Optional[Level] = None,
        best_ask: Optional[Level] = None,
    ) -> None:
        self.last_update_time = last_update_time
        self.best_bid = best_bid
        self.best_ask = best_ask

    @classmethod
    def new(
        cls,
        last_update_time: datetime,
        best_bid: Optional[Level] = None,
        best_ask: Optional[Level] = None,
    ) -> OrderBookL1:
        return cls(last_update_time, best_bid, best_ask)

    def mid_price(self) -> Optional[Decimal]:
        """Calculate the mid-price by taking the average of the best bid and ask prices."""
        if self.best_ask is None or self.best_bid is None:
            return None
        return (self.best_bid.price + self.best_ask.price) / Decimal("2")

    def volume_weighted_mid_price(self) -> Optional[Decimal]:
        """Calculate the volume weighted mid-price (micro-price)."""
        if self.best_ask is None or self.best_bid is None:
            return None
        return (
            (self.best_bid.price * self.best_ask.amount) + (self.best_ask.price * self.best_bid.amount)
        ) / (self.best_bid.amount + self.best_ask.amount)

    def __repr__(self) -> str:
        return (
            f"OrderBookL1("
            f"last_update_time={self.last_update_time!r}, "
            f"best_bid={self.best_bid!r}, "
            f"best_ask={self.best_ask!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderBookL1):
            return NotImplemented
        return (
            self.last_update_time == other.last_update_time
            and self.best_bid == other.best_bid
            and self.best_ask == other.best_ask
        )

    def __hash__(self) -> int:
        return hash((self.last_update_time, self.best_bid, self.best_ask))


# Placeholder for OrderBook - to be implemented later
@dataclass(frozen=True)
class OrderBook:
    """Placeholder for OrderBook - full implementation later."""
    sequence: int
    time_engine: Optional[datetime]
    # bids and asks would be added later

    def __str__(self) -> str:
        return f"OrderBook(sequence={self.sequence}, time_engine={self.time_engine})"


class OrderBookEvent(Enum):
    """Barter OrderBookEvent enum."""
    SNAPSHOT = "snapshot"
    UPDATE = "update"

    def __str__(self) -> str:
        return self.value


DataKindType = Union[PublicTrade, OrderBookL1, OrderBookEvent, None, None, None]  # Candle and Liquidation placeholders


class DataKind:
    """Available kinds of normalised Barter MarketEvent."""

    def __init__(self, kind: str, data: DataKindType) -> None:
        self._kind = kind
        self._data = data

    @classmethod
    def trade(cls, trade: PublicTrade) -> DataKind:
        return cls("trade", trade)

    @classmethod
    def order_book_l1(cls, orderbook: OrderBookL1) -> DataKind:
        return cls("order_book_l1", orderbook)

    @classmethod
    def order_book(cls, event: OrderBookEvent) -> DataKind:
        return cls("order_book", event)

    @classmethod
    def candle(cls) -> DataKind:
        return cls("candle", None)

    @classmethod
    def liquidation(cls) -> DataKind:
        return cls("liquidation", None)

    @property
    def kind(self) -> str:
        return self._kind

    @property
    def data(self) -> DataKindType:
        return self._data

    def kind_name(self) -> str:
        if self._kind == "trade":
            return "public_trade"
        elif self._kind == "order_book_l1":
            return "l1"
        elif self._kind == "order_book":
            return "l2"
        elif self._kind == "candle":
            return "candle"
        elif self._kind == "liquidation":
            return "liquidation"
        else:
            return self._kind

    def __repr__(self) -> str:
        return f"DataKind({self._kind!r}, {self._data!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, DataKind):
            return NotImplemented
        return self._kind == other._kind and self._data == other._data

    def __hash__(self) -> int:
        return hash((self._kind, self._data))


T = TypeVar("T")


class MarketEvent(Generic[InstrumentKey, T]):
    """Normalised Barter MarketEvent wrapping the data in metadata."""

    def __init__(
        self,
        time_exchange: datetime,
        time_received: datetime,
        exchange: str,  # Simplified, use str for now
        instrument: InstrumentKey,
        kind: T,
    ) -> None:
        self.time_exchange = time_exchange
        self.time_received = time_received
        self.exchange = exchange
        self.instrument = instrument
        self.kind = kind

    def map_kind(self, op):
        """Map the kind using the provided operation."""
        new_kind = op(self.kind)
        return MarketEvent(
            self.time_exchange,
            self.time_received,
            self.exchange,
            self.instrument,
            new_kind,
        )

    def __repr__(self) -> str:
        return (
            f"MarketEvent("
            f"time_exchange={self.time_exchange!r}, "
            f"time_received={self.time_received!r}, "
            f"exchange={self.exchange!r}, "
            f"instrument={self.instrument!r}, "
            f"kind={self.kind!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MarketEvent):
            return NotImplemented
        return (
            self.time_exchange == other.time_exchange
            and self.time_received == other.time_received
            and self.exchange == other.exchange
            and self.instrument == other.instrument
            and self.kind == other.kind
        )

    def __hash__(self) -> int:
        return hash((
            self.time_exchange,
            self.time_received,
            self.exchange,
            self.instrument,
            self.kind,
        ))


# For convenience, define typed versions
def as_public_trade(event: MarketEvent[InstrumentKey, DataKind]) -> Optional[MarketEvent[InstrumentKey, PublicTrade]]:
    """Return as PublicTrade if applicable."""
    if isinstance(event.kind.data, PublicTrade):
        return MarketEvent(
            event.time_exchange,
            event.time_received,
            event.exchange,
            event.instrument,
            event.kind.data,
        )
    return None


def as_order_book_l1(event: MarketEvent[InstrumentKey, DataKind]) -> Optional[MarketEvent[InstrumentKey, OrderBookL1]]:
    """Return as OrderBookL1 if applicable."""
    if isinstance(event.kind.data, OrderBookL1):
        return MarketEvent(
            event.time_exchange,
            event.time_received,
            event.exchange,
            event.instrument,
            event.kind.data,
        )
    return None


def as_order_book(event: MarketEvent[InstrumentKey, DataKind]) -> Optional[MarketEvent[InstrumentKey, OrderBookEvent]]:
    """Return as OrderBookEvent if applicable."""
    if isinstance(event.kind.data, OrderBookEvent):
        return MarketEvent(
            event.time_exchange,
            event.time_received,
            event.exchange,
            event.instrument,
            event.kind.data,
        )
    return None