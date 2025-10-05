"""Rust-backed trade binding helpers.

These wrappers expose the new PyO3 trade primitives while keeping a Pythonic
surface area that mirrors the original pure Python dataclasses. They can be
adopted incrementally by callers that need access to the Rust implementations
without disrupting existing usage of :mod:`barter_python.execution`.
"""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal
from typing import Generic, TypeVar

from .barter_python import (
    AssetFees as _AssetFees,
)
from .barter_python import (
    QuoteAsset as _QuoteAsset,
)
from .barter_python import (
    Trade as _Trade,
)
from .barter_python import (
    TradeId as _TradeId,
)
from .execution import OrderId, StrategyId
from .instrument import Side

AssetKey = TypeVar("AssetKey")
InstrumentKey = TypeVar("InstrumentKey")


class TradeId:
    """Wrapper around the PyO3-backed :class:`barter_python.TradeId`."""

    __slots__ = ("_inner",)

    def __init__(self, inner: _TradeId) -> None:
        self._inner = inner

    @classmethod
    def new(cls, value: str) -> TradeId:
        return cls(_TradeId.new(value))

    @property
    def value(self) -> str:
        return self._inner.value

    def __str__(self) -> str:
        return str(self._inner)

    def __repr__(self) -> str:
        return repr(self._inner)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TradeId):
            return NotImplemented
        return self._inner == other._inner

    def __hash__(self) -> int:
        return hash(self._inner)


class AssetFees(Generic[AssetKey]):
    """Wrapper around the PyO3-backed :class:`barter_python.AssetFees`."""

    __slots__ = ("_inner",)

    def __init__(self, asset: AssetKey, fees: Decimal) -> None:
        self._inner = _AssetFees(asset, fees)

    @classmethod
    def quote_fees(cls, fees: Decimal) -> AssetFees[AssetKey]:
        return cls(_QuoteAsset(), fees)

    @property
    def asset(self) -> AssetKey:
        return self._inner.asset

    @property
    def fees(self) -> Decimal:
        return self._inner.fees

    def __str__(self) -> str:
        return str(self._inner)

    def __repr__(self) -> str:
        return repr(self._inner)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetFees):
            return NotImplemented
        return self._inner == other._inner

    def __hash__(self) -> int:
        return hash(self._inner)


class Trade(Generic[AssetKey, InstrumentKey]):
    """Wrapper around the PyO3-backed :class:`barter_python.Trade`."""

    __slots__ = (
        "_inner",
        "_id",
        "_order_id",
        "_instrument",
        "_strategy",
        "_time_exchange",
        "_side",
        "_price",
        "_quantity",
        "_fees",
    )

    def __init__(
        self,
        trade_id: TradeId,
        order_id: OrderId,
        instrument: InstrumentKey,
        strategy: StrategyId,
        time_exchange: datetime,
        side: Side,
        price: Decimal,
        quantity: Decimal,
        fees: AssetFees[AssetKey],
    ) -> None:
        side_value = side.value if isinstance(side, Side) else str(side)

        self._inner = _Trade(
            trade_id._inner,
            order_id,
            instrument,
            strategy,
            time_exchange,
            side_value,
            price,
            quantity,
            fees._inner,
        )

        self._id = trade_id
        self._order_id = order_id
        self._instrument = instrument
        self._strategy = strategy
        self._time_exchange = time_exchange
        self._side = side
        self._price = price
        self._quantity = quantity
        self._fees = fees

    @property
    def id(self) -> TradeId:
        return self._id

    @property
    def order_id(self) -> OrderId:
        return self._order_id

    @property
    def instrument(self) -> InstrumentKey:
        return self._instrument

    @property
    def strategy(self) -> StrategyId:
        return self._strategy

    @property
    def time_exchange(self) -> datetime:
        return self._time_exchange

    @property
    def side(self) -> Side:
        return self._side

    @property
    def price(self) -> Decimal:
        return self._price

    @property
    def quantity(self) -> Decimal:
        return self._quantity

    @property
    def fees(self) -> AssetFees[AssetKey]:
        return self._fees

    def value_quote(self) -> Decimal:
        return self._inner.value_quote()

    def __str__(self) -> str:
        return str(self._inner)

    def __repr__(self) -> str:
        return repr(self._inner)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Trade):
            return NotImplemented
        return self._inner == other._inner

    def __hash__(self) -> int:
        return hash(self._inner)
