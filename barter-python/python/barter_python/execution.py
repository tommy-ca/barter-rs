"""Pure Python implementation of barter-execution data structures."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Generic, Optional, TypeVar, Union

from .instrument import QuoteAsset, Side

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
class ClientOrderId:
    """Client order identifier."""

    value: str

    @classmethod
    def new(cls, value: str) -> ClientOrderId:
        return cls(value)

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"ClientOrderId({self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ClientOrderId):
            return NotImplemented
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)


@dataclass(frozen=True)
class OrderId:
    """Exchange order identifier."""

    value: str

    @classmethod
    def new(cls, value: str) -> OrderId:
        return cls(value)

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"OrderId({self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderId):
            return NotImplemented
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)


@dataclass(frozen=True)
class StrategyId:
    """Strategy identifier."""

    value: str

    @classmethod
    def new(cls, value: str) -> StrategyId:
        return cls(value)

    @classmethod
    def unknown(cls) -> StrategyId:
        return cls("unknown")

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return f"StrategyId({self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, StrategyId):
            return NotImplemented
        return self.value == other.value

    def __hash__(self) -> int:
        return hash(self.value)


@dataclass(frozen=True)
class OrderKey(Generic[ExchangeKey, InstrumentKey]):
    """Key identifying an order."""

    exchange: ExchangeKey
    instrument: InstrumentKey
    strategy: StrategyId
    cid: ClientOrderId

    def __str__(self) -> str:
        return f"{self.exchange}:{self.instrument}:{self.strategy}:{self.cid}"

    def __repr__(self) -> str:
        return (
            f"OrderKey("
            f"exchange={self.exchange!r}, "
            f"instrument={self.instrument!r}, "
            f"strategy={self.strategy!r}, "
            f"cid={self.cid!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, OrderKey):
            return NotImplemented
        return (
            self.exchange == other.exchange
            and self.instrument == other.instrument
            and self.strategy == other.strategy
            and self.cid == other.cid
        )

    def __hash__(self) -> int:
        return hash((self.exchange, self.instrument, self.strategy, self.cid))


@dataclass(frozen=True)
class Balance:
    """Asset balance with total and free amounts."""

    total: Decimal
    free: Decimal

    @classmethod
    def new(cls, total: Decimal, free: Decimal) -> Balance:
        return cls(total, free)

    def used(self) -> Decimal:
        """Calculate used balance."""
        return self.total - self.free

    def __str__(self) -> str:
        return f"Balance(total={self.total}, free={self.free})"

    def __repr__(self) -> str:
        return f"Balance(total={self.total!r}, free={self.free!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Balance):
            return NotImplemented
        return self.total == other.total and self.free == other.free

    def __hash__(self) -> int:
        return hash((self.total, self.free))


@dataclass(frozen=True)
class AssetBalance(Generic[AssetKey]):
    """Asset balance with timestamp."""

    asset: AssetKey
    balance: Balance
    time_exchange: datetime

    @classmethod
    def new(cls, asset: AssetKey, balance: Balance, time_exchange: datetime) -> AssetBalance[AssetKey]:
        return cls(asset, balance, time_exchange)

    def __str__(self) -> str:
        return f"AssetBalance(asset={self.asset}, balance={self.balance}, time={self.time_exchange})"

    def __repr__(self) -> str:
        return (
            f"AssetBalance("
            f"asset={self.asset!r}, "
            f"balance={self.balance!r}, "
            f"time_exchange={self.time_exchange!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AssetBalance):
            return NotImplemented
        return (
            self.asset == other.asset
            and self.balance == other.balance
            and self.time_exchange == other.time_exchange
        )

    def __hash__(self) -> int:
        return hash((self.asset, self.balance, self.time_exchange))


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
        return hash((
            self.id,
            self.order_id,
            self.instrument,
            self.strategy,
            self.time_exchange,
            self.side,
            self.price,
            self.quantity,
            self.fees,
        ))


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

    order: Optional[Open]

    @classmethod
    def new(cls, order: Optional[Open] = None) -> CancelInFlight:
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

    def __init__(self, state: Union[Cancelled, OrderError, str]):
        self._state = state

    @classmethod
    def cancelled(cls, cancelled: Cancelled) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls(cancelled)

    @classmethod
    def fully_filled(cls) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls("FullyFilled")

    @classmethod
    def expired(cls) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls("Expired")

    @classmethod
    def open_failed(cls, error: OrderError) -> InactiveOrderState[AssetKey, InstrumentKey]:
        return cls(error)

    @property
    def state(self) -> Union[Cancelled, OrderError, str]:
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

    def __init__(self, state: Union[ActiveOrderState, InactiveOrderState[AssetKey, InstrumentKey]]):
        self._state = state

    @classmethod
    def active(cls, state: ActiveOrderState) -> OrderState[AssetKey, InstrumentKey]:
        return cls(state)

    @classmethod
    def inactive(cls, state: InactiveOrderState[AssetKey, InstrumentKey]) -> OrderState[AssetKey, InstrumentKey]:
        return cls(state)

    @classmethod
    def fully_filled(cls) -> OrderState[AssetKey, InstrumentKey]:
        return cls(InactiveOrderState.fully_filled())

    @classmethod
    def expired(cls) -> OrderState[AssetKey, InstrumentKey]:
        return cls(InactiveOrderState.expired())

    @property
    def state(self) -> Union[ActiveOrderState, InactiveOrderState[AssetKey, InstrumentKey]]:
        return self._state

    def is_active(self) -> bool:
        return isinstance(self._state, (OpenInFlight, Open, CancelInFlight))

    def is_inactive(self) -> bool:
        return isinstance(self._state, InactiveOrderState)

    def time_exchange(self) -> Optional[datetime]:
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

    key: OrderKey[ExchangeKey, InstrumentKey]
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
        return hash((
            self.key,
            self.side,
            self.price,
            self.quantity,
            self.kind,
            self.time_in_force,
            self.state,
        ))


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
        return hash((self.side, self.price, self.quantity, self.kind, self.time_in_force))


@dataclass(frozen=True)
class OrderRequestOpen(Generic[ExchangeKey, InstrumentKey]):
    """Request to open an order."""

    key: OrderKey[ExchangeKey, InstrumentKey]
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

    key: OrderKey[ExchangeKey, InstrumentKey]
    state: Optional[OrderId]

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
    def new(cls, exchange: ExchangeKey, kind: AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]) -> AccountEvent[ExchangeKey, AssetKey, InstrumentKey]:
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
    AssetBalance[AssetKey],
    "Order[ExchangeKey, InstrumentKey, AssetKey]",
    "OrderResponseCancel[ExchangeKey, AssetKey, InstrumentKey]",
    Trade[AssetKey, InstrumentKey],
]


class AccountEventKind(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Account event kind."""

    def __init__(self, kind: str, data: AccountEventKindType[ExchangeKey, AssetKey, InstrumentKey]):
        self._kind = kind
        self._data = data

    @classmethod
    def snapshot(cls, snapshot: "AccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]") -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("snapshot", snapshot)

    @classmethod
    def balance_snapshot(cls, balance: AssetBalance[AssetKey]) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("balance_snapshot", balance)

    @classmethod
    def order_snapshot(cls, order: "Order[ExchangeKey, InstrumentKey, AssetKey]") -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("order_snapshot", order)

    @classmethod
    def order_cancelled(cls, response: "OrderResponseCancel[ExchangeKey, AssetKey, InstrumentKey]") -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
        return cls("order_cancelled", response)

    @classmethod
    def trade(cls, trade: Trade[AssetKey, InstrumentKey]) -> AccountEventKind[ExchangeKey, AssetKey, InstrumentKey]:
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


@dataclass(frozen=True)
class InstrumentAccountSnapshot(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Account snapshot for a specific instrument."""

    instrument: InstrumentKey
    orders: list["OrderSnapshot[ExchangeKey, AssetKey, InstrumentKey]"]

    @classmethod
    def new(cls, instrument: InstrumentKey, orders: Optional[list["OrderSnapshot[ExchangeKey, AssetKey, InstrumentKey]"]] = None) -> InstrumentAccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]:
        return cls(instrument, orders or [])

    def __str__(self) -> str:
        return f"InstrumentAccountSnapshot(instrument={self.instrument}, orders={len(self.orders)})"

    def __repr__(self) -> str:
        return f"InstrumentAccountSnapshot(instrument={self.instrument!r}, orders={self.orders!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, InstrumentAccountSnapshot):
            return NotImplemented
        return self.instrument == other.instrument and self.orders == other.orders

    def __hash__(self) -> int:
        return hash((self.instrument, tuple(self.orders)))


@dataclass(frozen=True)
class AccountSnapshot(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Full account snapshot."""

    exchange: ExchangeKey
    balances: list[AssetBalance[AssetKey]]
    instruments: list[InstrumentAccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]]

    @classmethod
    def new(
        cls,
        exchange: ExchangeKey,
        balances: list[AssetBalance[AssetKey]],
        instruments: list[InstrumentAccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]]
    ) -> AccountSnapshot[ExchangeKey, AssetKey, InstrumentKey]:
        return cls(exchange, balances, instruments)

    def time_most_recent(self) -> Optional[datetime]:
        """Get the most recent timestamp from balances or orders."""
        times = []
        for balance in self.balances:
            times.append(balance.time_exchange)
        for instrument in self.instruments:
            for order in instrument.orders:
                if order.state.time_exchange():
                    times.append(order.state.time_exchange())
        return max(times) if times else None

    def assets(self):
        """Iterate over unique assets."""
        seen = set()
        for balance in self.balances:
            if balance.asset not in seen:
                seen.add(balance.asset)
                yield balance.asset

    def instruments_iter(self):
        """Iterate over unique instruments."""
        seen = set()
        for instrument in self.instruments:
            if instrument.instrument not in seen:
                seen.add(instrument.instrument)
                yield instrument.instrument

    def __str__(self) -> str:
        return f"AccountSnapshot(exchange={self.exchange}, balances={len(self.balances)}, instruments={len(self.instruments)})"

    def __repr__(self) -> str:
        return (
            f"AccountSnapshot("
            f"exchange={self.exchange!r}, "
            f"balances={self.balances!r}, "
            f"instruments={self.instruments!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, AccountSnapshot):
            return NotImplemented
        return (
            self.exchange == other.exchange
            and self.balances == other.balances
            and self.instruments == other.instruments
        )

    def __hash__(self) -> int:
        return hash((self.exchange, tuple(self.balances), tuple(self.instruments)))


# Type aliases for convenience
OrderSnapshot = Order

# Placeholder for request types (to be implemented later)
@dataclass(frozen=True)
class OrderResponseCancel(Generic[ExchangeKey, AssetKey, InstrumentKey]):
    """Order cancellation response."""

    key: OrderKey[ExchangeKey, InstrumentKey]
    state: Union[Cancelled, Exception]  # Simplified for now

    def __str__(self) -> str:
        return f"OrderResponseCancel(key={self.key}, state={self.state})"

    def __repr__(self) -> str:
        return f"OrderResponseCancel(key={self.key!r}, state={self.state!r})"