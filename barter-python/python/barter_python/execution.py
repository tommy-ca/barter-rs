"""Pure Python implementation of barter-execution data structures."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, TypeVar, Union

from .instrument import QuoteAsset, Side
from .barter_python import (
    AccountSnapshot as _AccountSnapshot,
    AssetBalance as _AssetBalance,
    asset_balance_new as _asset_balance_new,
    ClientOrderId as _ClientOrderId,
    InstrumentAccountSnapshot as _InstrumentAccountSnapshot,
    ExecutionInstrumentMap as _ExecutionInstrumentMap,
    OrderId as _OrderId,
    OrderKey as _OrderKey,
    OrderEvent as _OrderEvent,
    balance_new as _balance_new,
    Balance as _Balance,
    StrategyId as _StrategyId,
)

ClientOrderId = _ClientOrderId
OrderId = _OrderId
StrategyId = _StrategyId
OrderKey = _OrderKey
InstrumentAccountSnapshot = _InstrumentAccountSnapshot
AccountSnapshot = _AccountSnapshot
OrderEvent = _OrderEvent
balance_new = _balance_new
asset_balance_new = _asset_balance_new


class ExecutionInstrumentMap:
    """High-level wrapper around the Rust execution instrument map."""

    __slots__ = ("_inner",)

    def __init__(self, inner: _ExecutionInstrumentMap):
        self._inner = inner

    @classmethod
    def from_definitions(cls, exchange, definitions):
        inner = _ExecutionInstrumentMap.from_definitions(exchange, definitions)
        return cls(inner)

    @classmethod
    def from_system_config(cls, exchange, config):
        inner = _ExecutionInstrumentMap.from_system_config(exchange, config)
        return cls(inner)

    @property
    def exchange_id(self):
        return self._inner.exchange_id

    @property
    def exchange_index(self):
        return self._inner.exchange_index

    def asset_names(self) -> list[str]:
        return list(self._inner.asset_names())

    def instrument_names(self) -> list[str]:
        return list(self._inner.instrument_names())

    def asset_index(self, name: str):
        return self._inner.asset_index(name)

    def asset_name(self, index):
        return self._inner.asset_name(index)

    def instrument_index(self, name: str):
        return self._inner.instrument_index(name)

    def instrument_name(self, index):
        return self._inner.instrument_name(index)

    def __repr__(self) -> str:
        return repr(self._inner)


class Balance:
    """Python-friendly wrapper around the Rust-backed balance type."""

    __slots__ = ("_inner",)

    def __init__(self, total, free):
        self._inner = _balance_new(total, free)

    @classmethod
    def new(cls, total, free):
        return cls(total, free)

    @classmethod
    def _from_inner(cls, inner: _Balance):
        obj = object.__new__(cls)
        obj._inner = inner
        return obj

    @property
    def total(self):
        return self._inner.total

    @property
    def free(self):
        return self._inner.free

    def used(self):
        return self._inner.used()

    def to_dict(self):
        return self._inner.to_dict()

    def __getattr__(self, name: str):
        return getattr(self._inner, name)

    def __repr__(self) -> str:
        return repr(self._inner)

    def __str__(self) -> str:
        return str(self._inner)

    def __eq__(self, other: object) -> bool:
        other_inner = getattr(other, "_inner", other)
        return self._inner == other_inner

    def __hash__(self) -> int:
        return hash(self._inner)


class AssetBalance:
    """Python wrapper exposing constructors for the Rust asset balance type."""

    __slots__ = ("_inner",)

    def __init__(self, asset, balance, time_exchange):
        if isinstance(balance, Balance):
            balance_inner = balance._inner
        elif isinstance(balance, _Balance):
            balance_inner = balance
        else:
            raise TypeError("balance must be a Balance value")
        self._inner = _asset_balance_new(asset, balance_inner, time_exchange)

    @classmethod
    def new(cls, asset, balance, time_exchange):
        return cls(asset, balance, time_exchange)

    @classmethod
    def _from_inner(cls, inner: _AssetBalance):
        obj = object.__new__(cls)
        obj._inner = inner
        return obj

    @property
    def asset(self):
        return self._inner.asset

    @property
    def balance(self) -> Balance:
        return Balance._from_inner(self._inner.balance)

    @property
    def time_exchange(self):
        return self._inner.time_exchange

    def __getattr__(self, name: str):
        return getattr(self._inner, name)

    def __repr__(self) -> str:
        return repr(self._inner)

    def __str__(self) -> str:
        return str(self._inner)

    def __eq__(self, other: object) -> bool:
        other_inner = getattr(other, "_inner", other)
        return self._inner == other_inner

    def __hash__(self) -> int:
        return hash(self._inner)

AssetKey = TypeVar("AssetKey")
InstrumentKey = TypeVar("InstrumentKey")
ExchangeKey = TypeVar("ExchangeKey")


class OrderKind(Enum):
    """Order kind - Market or Limit."""

    MARKET = "market"
    LIMIT = "limit"

    def __str__(self) -> str:
        return self.value


class TimeInForce(Enum):
    """Time in force for orders."""

    GOOD_UNTIL_CANCELLED = "good_until_cancelled"
    GOOD_UNTIL_END_OF_DAY = "good_until_end_of_day"
    FILL_OR_KILL = "fill_or_kill"
    IMMEDIATE_OR_CANCEL = "immediate_or_cancel"

    def __str__(self) -> str:
        return self.value
@dataclass(frozen=True)
class AssetFees(Generic[AssetKey]):
    """Asset fees."""

    asset: AssetKey
    fees: Decimal

    @classmethod
    def quote_fees(cls, fees: Decimal):
        return AssetFees(QuoteAsset(), fees)

    def __str__(self) -> str:
        return f"AssetFees(asset={self.asset}, fees={self.fees})"

    def __repr__(self) -> str:
        return f"AssetFees(asset={self.asset!r}, fees={self.fees!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetFees):
            return NotImplemented
        return self.asset == other.asset and self.fees == other.fees

    def __hash__(self) -> int:
        return hash((self.asset, self.fees))


@dataclass(frozen=True)
class TradeId:
    """Trade identifier."""

    value: str

    @classmethod
    def new(cls, value: str) -> TradeId:
        return cls(value)

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"TradeId({self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TradeId):
            return NotImplemented
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)


@dataclass(frozen=True)
class Trade(Generic[AssetKey, InstrumentKey]):
    """Trade execution."""

    id: TradeId
    order_id: OrderId
    instrument: InstrumentKey
    strategy: StrategyId
    time_exchange: datetime
    side: Side
    price: Decimal
    quantity: Decimal
    fees: AssetFees[AssetKey]

    def value_quote(self) -> Decimal:
        """Calculate quote value of the trade."""
        return self.price * self.quantity

    def __str__(self) -> str:
        return (
            f"Trade("
            f"instrument={self.instrument}, "
            f"side={self.side}, "
            f"price={self.price}, "
            f"quantity={self.quantity}, "
            f"time={self.time_exchange}"
            f")"
        )

    def __repr__(self) -> str:
        return (
            f"Trade("
            f"id={self.id!r}, "
            f"order_id={self.order_id!r}, "
            f"instrument={self.instrument!r}, "
            f"strategy={self.strategy!r}, "
            f"time_exchange={self.time_exchange!r}, "
            f"side={self.side!r}, "
            f"price={self.price!r}, "
            f"quantity={self.quantity!r}, "
            f"fees={self.fees!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Trade):
            return NotImplemented
        return (
            self.id == other.id
            and self.order_id == other.order_id
            and self.instrument == other.instrument
            and self.strategy == other.strategy
            and self.time_exchange == other.time_exchange
            and self.side == other.side
            and self.price == other.price
            and self.quantity == other.quantity
            and self.fees == other.fees
        )

    def __hash__(self) -> int:
        return hash(
            (
                self.id,
                self.order_id,
                self.instrument,
                self.strategy,
                self.time_exchange,
                self.side,
                self.price,
                self.quantity,
                self.fees,
            )
        )


# Order states
@dataclass(frozen=True)
class OpenInFlight:
    """Order is being submitted to exchange."""

    def __str__(self) -> str:
        return "OpenInFlight"

    def __repr__(self) -> str:
        return "OpenInFlight()"


@dataclass(frozen=True)
class Open:
    """Order is open on exchange."""

    id: OrderId
    time_exchange: datetime
    filled_quantity: Decimal

    def quantity_remaining(self, initial_quantity: Decimal) -> Decimal:
        """Calculate remaining quantity to fill."""
        return initial_quantity - self.filled_quantity

    def __str__(self) -> str:
        return f"Open(id={self.id}, time={self.time_exchange}, filled={self.filled_quantity})"

    def __repr__(self) -> str:
        return (
            f"Open("
            f"id={self.id!r}, "
            f"time_exchange={self.time_exchange!r}, "
            f"filled_quantity={self.filled_quantity!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Open):
            return NotImplemented
        return (
            self.id == other.id
            and self.time_exchange == other.time_exchange
            and self.filled_quantity == other.filled_quantity
        )

    def __hash__(self) -> int:
        return hash((self.id, self.time_exchange, self.filled_quantity))


@dataclass(frozen=True)
class CancelInFlight:
    """Order cancellation is in flight."""

    order: Open | None

    @classmethod
    def new(cls, order: Open | None = None) -> CancelInFlight:
        return cls(order)

    def __str__(self) -> str:
        return f"CancelInFlight(order={self.order})"

    def __repr__(self) -> str:
        return f"CancelInFlight(order={self.order!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, CancelInFlight):
            return NotImplemented
        return self.order == other.order

    def __hash__(self) -> int:
        return hash(self.order)


ActiveOrderState = Union[OpenInFlight, Open, CancelInFlight]


@dataclass(frozen=True)
class Cancelled:
    """Order was cancelled."""

    id: OrderId
    time_exchange: datetime

    def __str__(self) -> str:
        return f"Cancelled(id={self.id}, time={self.time_exchange})"

    def __repr__(self) -> str:
        return f"Cancelled(id={self.id!r}, time_exchange={self.time_exchange!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Cancelled):
            return NotImplemented
        return self.id == other.id and self.time_exchange == other.time_exchange

    def __hash__(self) -> int:
        return hash((self.id, self.time_exchange))


class OrderError(Enum):
    """Order error types."""

    INSUFFICIENT_BALANCE = "insufficient_balance"
    INVALID_PRICE = "invalid_price"
    INVALID_QUANTITY = "invalid_quantity"
    UNKNOWN_INSTRUMENT = "unknown_instrument"
    EXCHANGE_ERROR = "exchange_error"

    def __str__(self) -> str:
        return self.value


class InactiveOrderState(Generic[AssetKey, InstrumentKey]):
    """Inactive order state."""

    def __init__(self, state: Cancelled | OrderError | str):
        self._state = state

    @classmethod
    def cancelled(
        cls, cancelled: Cancelled
    ) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls(cancelled)

    @classmethod
    def fully_filled(cls) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls("FullyFilled")

    @classmethod
    def expired(cls) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls("Expired")

    @classmethod
    def open_failed(
        cls, error: OrderError
    ) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls(error)

    @property
    def state(self) -> Cancelled | OrderError | str:
        return self._state

    def is_cancelled(self) -> bool:
        return isinstance(self._state, Cancelled)

    def is_fully_filled(self) -> bool:
        return self._state == "FullyFilled"

    def is_expired(self) -> bool:
        return self._state == "Expired"

    def is_open_failed(self) -> bool:
        return isinstance(self._state, OrderError)

    def __str__(self) -> str:
        return f"InactiveOrderState({self._state})"

    def __repr__(self) -> str:
        return f"InactiveOrderState({self._state!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InactiveOrderState):
            return NotImplemented
        return self._state == other._state

    def __hash__(self) -> int:
        return hash(self._state)


class OrderState(Generic[AssetKey, InstrumentKey]):
    """Order state enum."""

    def __init__(
        self, state: ActiveOrderState | InactiveOrderState[AssetKey, InstrumentKey]
    ):
        self._state = state

    @classmethod
    def active(cls, state: ActiveOrderState) -> OrderState[AssetKey, InstrumentKey]:
        return cls(state)

    @classmethod
    def inactive(
        cls, state: InactiveOrderState[AssetKey, InstrumentKey]
    ) -> OrderState[AssetKey, InstrumentKey]:
        return cls(state)

    @classmethod
    def fully_filled(cls) -> OrderState[AssetKey, InstrumentKey]:
        return cls(InactiveOrderState.fully_filled())

    @classmethod
    def expired(cls) -> OrderState[AssetKey, InstrumentKey]:
        return cls(InactiveOrderState.expired())

    @property
    def state(self) -> ActiveOrderState | InactiveOrderState[AssetKey, InstrumentKey]:
        return self._state

    def is_active(self) -> bool:
        return isinstance(self._state, (OpenInFlight, Open, CancelInFlight))

    def is_inactive(self) -> bool:
        return isinstance(self._state, InactiveOrderState)

    def time_exchange(self) -> datetime | None:
        """Get the exchange timestamp if available."""
        if isinstance(self._state, InactiveOrderState):
            if isinstance(self._state.state, Cancelled):
                return self._state.state.time_exchange
            return None
        elif isinstance(self._state, Open):
            return self._state.time_exchange
        elif isinstance(self._state, CancelInFlight) and self._state.order:
            return self._state.order.time_exchange
        return None

    def __str__(self) -> str:
        return f"OrderState({self._state})"

    def __repr__(self) -> str:
        return f"OrderState({self._state!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderState):
            return NotImplemented
        return self._state == other._state

    def __hash__(self) -> int:
        return hash(self._state)


@dataclass(frozen=True)
class Order(Generic[ExchangeKey, InstrumentKey, AssetKey]):
    """Order data structure."""

    key: OrderKey
    side: Side
    price: Decimal
    quantity: Decimal
    kind: OrderKind
    time_in_force: TimeInForce
    state: OrderState[AssetKey, InstrumentKey]

    def __str__(self) -> str:
        return (
            f"Order("
            f"key={self.key}, "
            f"side={self.side}, "
            f"price={self.price}, "
            f"quantity={self.quantity}, "
            f"kind={self.kind}, "
            f"time_in_force={self.time_in_force}, "
            f"state={self.state}"
            f")"
        )

    def __repr__(self) -> str:
        return (
            f"Order("
            f"key={self.key!r}, "
            f"side={self.side!r}, "
            f"price={self.price!r}, "
            f"quantity={self.quantity!r}, "
            f"kind={self.kind!r}, "
            f"time_in_force={self.time_in_force!r}, "
            f"state={self.state!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Order):
            return NotImplemented
        return (
            self.key == other.key
            and self.side == other.side
            and self.price == other.price
            and self.quantity == other.quantity
            and self.kind == other.kind
            and self.time_in_force == other.time_in_force
            and self.state == other.state
        )

    def __hash__(self) -> int:
        return hash(
            (
                self.key,
                self.side,
                self.price,
                self.quantity,
                self.kind,
                self.time_in_force,
                self.state,
            )
        )


# Order request types
@dataclass(frozen=True)
class RequestOpen:
    """Request to open an order."""

    side: Side
    price: Decimal
    quantity: Decimal
    kind: OrderKind
    time_in_force: TimeInForce

    def __str__(self) -> str:
        return (
            f"RequestOpen("
            f"side={self.side}, "
            f"price={self.price}, "
            f"quantity={self.quantity}, "
            f"kind={self.kind}, "
            f"time_in_force={self.time_in_force}"
            f")"
        )

    def __repr__(self) -> str:
        return (
            f"RequestOpen("
            f"side={self.side!r}, "
            f"price={self.price!r}, "
            f"quantity={self.quantity!r}, "
            f"kind={self.kind!r}, "
            f"time_in_force={self.time_in_force!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, RequestOpen):
            return NotImplemented
        return (
            self.side == other.side
            and self.price == other.price
            and self.quantity == other.quantity
            and self.kind == other.kind
            and self.time_in_force == other.time_in_force
        )

    def __hash__(self) -> int:
        return hash(
            (self.side, self.price, self.quantity, self.kind, self.time_in_force)
        )


@dataclass(frozen=True)
class OrderRequestOpen(Generic[ExchangeKey, InstrumentKey]):
    """Request to open an order."""

    key: OrderKey
    state: RequestOpen

    def __str__(self) -> str:
        return f"OrderRequestOpen(key={self.key}, state={self.state})"

    def __repr__(self) -> str:
        return f"OrderRequestOpen(key={self.key!r}, state={self.state!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderRequestOpen):
            return NotImplemented
        return self.key == other.key and self.state == other.state

    def __hash__(self) -> int:
        return hash((self.key, self.state))


@dataclass(frozen=True)
class OrderRequestCancel(Generic[ExchangeKey, InstrumentKey]):
    """Request to cancel an order."""

    key: OrderKey
    state: OrderId | None

    def __str__(self) -> str:
        return f"OrderRequestCancel(key={self.key}, state={self.state})"

    def __repr__(self) -> str:
        return f"OrderRequestCancel(key={self.key!r}, state={self.state!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderRequestCancel):
            return NotImplemented
        return self.key == other.key and self.state == other.state

    def __hash__(self) -> int:
        return hash((self.key, self.state))


# Account events
@dataclass(frozen=True)
class AccountEvent(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Account event from exchange."""

    exchange: ExchangeKey
    kind: AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]

    @classmethod
    def new(
        cls,
        exchange: ExchangeKey,
        kind: AccountEventKind[ExchangeKey, AssetKey, InstrumentKey],
    ) -> AccountEvent[ExchangeKey, AssetKey, InstrumentKey]:
        return cls(exchange, kind)

    def __str__(self) -> str:
        return f"AccountEvent(exchange={self.exchange}, kind={self.kind})"

    def __repr__(self) -> str:
        return f"AccountEvent(exchange={self.exchange!r}, kind={self.kind!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AccountEvent):
            return NotImplemented
        return self.exchange == other.exchange and self.kind == other.kind

    def __hash__(self) -> int:
        return hash((self.exchange, self.kind))


AccountEventKindType = Union[
    "AccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]",
    "AssetBalance",
    "Order[ExchangeKey, InstrumentKey, AssetKey]",
    "OrderResponseCancel[ExchangeKey, AssetKey, InstrumentKey]",
    Trade[AssetKey, InstrumentKey],
]


class AccountEventKind(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Account event kind."""

    def __init__(
        self,
        kind: str,
        data: AccountEventKindType[ExchangeKey, AssetKey, InstrumentKey],
    ):
        self._kind = kind
        self._data = data

    @classmethod
    def snapshot(
        cls, snapshot: AccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]
    ) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("snapshot", snapshot)

    @classmethod
    def balance_snapshot(
        cls, balance: AssetBalance[AssetKey]
    ) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("balance_snapshot", balance)

    @classmethod
    def order_snapshot(
        cls, order: Order[ExchangeKey, InstrumentKey, AssetKey]
    ) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("order_snapshot", order)

    @classmethod
    def order_cancelled(
        cls, response: OrderResponseCancel[ExchangeKey, AssetKey, InstrumentKey]
    ) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("order_cancelled", response)

    @classmethod
    def trade(
        cls, trade: Trade[AssetKey, InstrumentKey]
    ) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("trade", trade)

    @property
    def kind(self) -> str:
        return self._kind

    @property
    def data(self) -> AccountEventKindType[ExchangeKey, AssetKey, InstrumentKey]:
        return self._data

    def __str__(self) -> str:
        return f"AccountEventKind({self._kind}: {self._data})"

    def __repr__(self) -> str:
        return f"AccountEventKind({self._kind!r}, {self._data!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AccountEventKind):
            return NotImplemented
        return self._kind == other._kind and self._data == other._data

    def __hash__(self) -> int:
        return hash((self._kind, self._data))


# Type aliases for convenience
OrderSnapshot = Order


# Placeholder for request types (to be implemented later)
@dataclass(frozen=True)
class OrderResponseCancel(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Order cancellation response."""

    key: OrderKey
    state: Cancelled | Exception  # Simplified for now

    def __str__(self) -> str:
        return f"OrderResponseCancel(key={self.key}, state={self.state})"

    def __repr__(self) -> str:
        return f"OrderResponseCancel(key={self.key!r}, state={self.state!r})"
