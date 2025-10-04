"""Pure Python implementation of barter-data market event structures."""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, TypeVar, Union

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
        best_bid: Level | None = None,
        best_ask: Level | None = None,
    ) -> None:
        self.last_update_time = last_update_time
        self.best_bid = best_bid
        self.best_ask = best_ask

    @classmethod
    def new(
        cls,
        last_update_time: datetime,
        best_bid: Level | None = None,
        best_ask: Level | None = None,
    ) -> OrderBookL1:
        return cls(last_update_time, best_bid, best_ask)

    def mid_price(self) -> Decimal | None:
        """Calculate the mid-price by taking the average of the best bid and ask prices."""
        if self.best_ask is None or self.best_bid is None:
            return None
        return (self.best_bid.price + self.best_ask.price) / Decimal("2")

    def volume_weighted_mid_price(self) -> Decimal | None:
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


class Candle:
    """Normalised Barter OHLCV Candle model."""

    def __init__(
        self,
        close_time: datetime,
        open: float,
        high: float,
        low: float,
        close: float,
        volume: float,
        trade_count: int,
    ) -> None:
        self.close_time = close_time
        self.open = open
        self.high = high
        self.low = low
        self.close = close
        self.volume = volume
        self.trade_count = trade_count

    def __repr__(self) -> str:
        return (
            f"Candle("
            f"close_time={self.close_time!r}, "
            f"open={self.open!r}, "
            f"high={self.high!r}, "
            f"low={self.low!r}, "
            f"close={self.close!r}, "
            f"volume={self.volume!r}, "
            f"trade_count={self.trade_count!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Candle):
            return NotImplemented
        return (
            self.close_time == other.close_time
            and self.open == other.open
            and self.high == other.high
            and self.low == other.low
            and self.close == other.close
            and self.volume == other.volume
            and self.trade_count == other.trade_count
        )

    def __hash__(self) -> int:
        return hash((
            self.close_time,
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.trade_count,
        ))


class Liquidation:
    """Normalised Barter Liquidation model."""

    def __init__(
        self,
        side: Side,
        price: float,
        quantity: float,
        time: datetime,
    ) -> None:
        self.side = side
        self.price = price
        self.quantity = quantity
        self.time = time

    def __repr__(self) -> str:
        return (
            f"Liquidation("
            f"side={self.side!r}, "
            f"price={self.price!r}, "
            f"quantity={self.quantity!r}, "
            f"time={self.time!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Liquidation):
            return NotImplemented
        return (
            self.side == other.side
            and self.price == other.price
            and self.quantity == other.quantity
            and self.time == other.time
        )

    def __hash__(self) -> int:
        return hash((self.side, self.price, self.quantity, self.time))


class Bids:
    """Unit type to tag an OrderBookSide as the bid side (buyers) of an OrderBook."""
    pass


class Asks:
    """Unit type to tag an OrderBookSide as the ask side (sellers) of an OrderBook."""
    pass


class OrderBookSide:
    """Normalised Barter Levels for one Side of the OrderBook."""

    def __init__(self, side: Bids | Asks, levels: list[Level]) -> None:
        self.side = side
        self.levels = levels

    @classmethod
    def bids(cls, levels: list[Level]) -> OrderBookSide:
        """Construct a new OrderBookSide<Bids> from the provided Levels."""
        # Sort bids in descending price order (highest first)
        sorted_levels = sorted(levels, key=lambda level: level.price, reverse=True)
        return cls(Bids(), sorted_levels)

    @classmethod
    def asks(cls, levels: list[Level]) -> OrderBookSide:
        """Construct a new OrderBookSide<Asks> from the provided Levels."""
        # Sort asks in ascending price order (lowest first)
        sorted_levels = sorted(levels, key=lambda level: level.price)
        return cls(Asks(), sorted_levels)

    def best(self) -> Level | None:
        """Get the best Level on this OrderBookSide."""
        return self.levels[0] if self.levels else None

    def __repr__(self) -> str:
        side_name = "Bids" if isinstance(self.side, Bids) else "Asks"
        return f"OrderBookSide({side_name}, levels={self.levels!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderBookSide):
            return NotImplemented
        return self.side.__class__ == other.side.__class__ and self.levels == other.levels


class OrderBook:
    """Normalised Barter OrderBook snapshot."""

    def __init__(
        self,
        sequence: int,
        time_engine: datetime | None,
        bids: OrderBookSide,
        asks: OrderBookSide,
    ) -> None:
        self.sequence = sequence
        self.time_engine = time_engine
        self.bids = bids
        self.asks = asks

    @classmethod
    def new(
        cls,
        sequence: int,
        time_engine: datetime | None,
        bids: list[Level],
        asks: list[Level],
    ) -> OrderBook:
        """Construct a new sorted OrderBook."""
        return cls(
            sequence,
            time_engine,
            OrderBookSide.bids(bids),
            OrderBookSide.asks(asks),
        )

    def mid_price(self) -> Decimal | None:
        """Calculate the mid-price by taking the average of the best bid and ask prices."""
        best_bid = self.bids.best()
        best_ask = self.asks.best()
        if best_bid is None or best_ask is None:
            return None
        return (best_bid.price + best_ask.price) / Decimal("2")

    def volume_weighted_mid_price(self) -> Decimal | None:
        """Calculate the volume weighted mid-price (micro-price)."""
        best_bid = self.bids.best()
        best_ask = self.asks.best()
        if best_bid is None or best_ask is None:
            return None
        return (
            (best_bid.price * best_ask.amount) + (best_ask.price * best_bid.amount)
        ) / (best_bid.amount + best_ask.amount)

    def __repr__(self) -> str:
        return (
            f"OrderBook("
            f"sequence={self.sequence!r}, "
            f"time_engine={self.time_engine!r}, "
            f"bids={self.bids!r}, "
            f"asks={self.asks!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderBook):
            return NotImplemented
        return (
            self.sequence == other.sequence
            and self.time_engine == other.time_engine
            and self.bids == other.bids
            and self.asks == other.asks
        )

    def __hash__(self) -> int:
        return hash((self.sequence, self.time_engine, self.bids, self.asks))


class OrderBookEvent(Enum):
    """Barter OrderBookEvent enum."""
    SNAPSHOT = "snapshot"
    UPDATE = "update"

    def __str__(self) -> str:
        return self.value


DataKindType = Union[PublicTrade, OrderBookL1, OrderBookEvent, Candle, Liquidation, None]


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
    def candle(cls, candle: Candle) -> DataKind:
        return cls("candle", candle)

    @classmethod
    def liquidation(cls, liquidation: Liquidation) -> DataKind:
        return cls("liquidation", liquidation)

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
def as_public_trade(event: MarketEvent[InstrumentKey, DataKind]) -> MarketEvent[InstrumentKey, PublicTrade] | None:
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


def as_order_book_l1(event: MarketEvent[InstrumentKey, DataKind]) -> MarketEvent[InstrumentKey, OrderBookL1] | None:
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


def as_order_book(event: MarketEvent[InstrumentKey, DataKind]) -> MarketEvent[InstrumentKey, OrderBookEvent] | None:
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


def as_candle(event: MarketEvent[InstrumentKey, DataKind]) -> MarketEvent[InstrumentKey, Candle] | None:
    """Return as Candle if applicable."""
    if isinstance(event.kind.data, Candle):
        return MarketEvent(
            event.time_exchange,
            event.time_received,
            event.exchange,
            event.instrument,
            event.kind.data,
        )
    return None


def as_liquidation(event: MarketEvent[InstrumentKey, DataKind]) -> MarketEvent[InstrumentKey, Liquidation] | None:
    """Return as Liquidation if applicable."""
    if isinstance(event.kind.data, Liquidation):
        return MarketEvent(
            event.time_exchange,
            event.time_received,
            event.exchange,
            event.instrument,
            event.kind.data,
        )
    return None
