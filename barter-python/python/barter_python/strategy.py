"""Pure Python implementation of barter-strategy algorithms."""

from __future__ import annotations

import importlib
from collections.abc import Iterable
from decimal import Decimal
from typing import TYPE_CHECKING, Callable, Protocol, TypeVar

_core = importlib.import_module("barter_python.barter_python")
_build_ioc_market_order_to_close_position = _core.build_ioc_market_order_to_close_position
_InstrumentFilterBinding = _core.InstrumentFilter
_execution_bindings = _core.execution
from .execution import (
    OrderKey,
    OrderKind,
    OrderRequestCancel,
    OrderRequestOpen,
    RequestOpen,
    TimeInForce,
)
from .instrument import Side

# Type aliases for common key types
ExchangeIndex = int
InstrumentIndex = int

# Type variables for generic strategy interfaces
ExchangeKey = TypeVar("ExchangeKey")
AssetKey = TypeVar("AssetKey")
InstrumentKey = TypeVar("InstrumentKey")
State = TypeVar("State")


class AlgoStrategy(Protocol[ExchangeKey, InstrumentKey]):
    """Strategy interface for generating algorithmic open and cancel order requests based on the current EngineState.

    This allows full customisation of algorithmic trading logic.

    Different strategies may:
    - Implement momentum-based trading
    - Use statistical arbitrage
    - Apply machine learning models
    - etc.
    """

    def generate_algo_orders(
        self,
        state: State,
    ) -> tuple[
        Iterable[OrderRequestCancel[ExchangeKey, InstrumentKey]],
        Iterable[OrderRequestOpen[ExchangeKey, InstrumentKey]],
    ]:
        """Generate algorithmic orders based on current system State.

        Args:
            state: Current system state

        Returns:
            Tuple of (cancel_requests, open_requests)
        """
        ...


class ClosePositionsStrategy(Protocol[ExchangeKey, AssetKey, InstrumentKey]):
    """Strategy interface for generating open and cancel order requests that close open positions.

    This allows full customisation of how a strategy will close a position.

    Different strategies may:
    - Use different order types (Market, Limit, etc.).
    - Prioritise certain exchanges.
    - Increase the position of an inversely correlated instrument in order to neutralise exposure.
    - etc.
    """

    def close_positions_requests(
        self,
        state: State,
        filter: "InstrumentFilter[ExchangeKey, AssetKey, InstrumentKey]" | None = None,
    ) -> tuple[
        Iterable[OrderRequestCancel[ExchangeKey, InstrumentKey]],
        Iterable[OrderRequestOpen[ExchangeKey, InstrumentKey]],
    ]:
        """Generate orders based on current system State.

        Args:
            state: Current system state
            filter: Instrument filter to apply

        Returns:
            Tuple of (cancel_requests, open_requests)
        """
        ...


class _InstrumentFilterProtocol(Protocol[ExchangeKey, AssetKey, InstrumentKey]):
    """Filter for instruments in the engine state."""

    def matches(self, exchange: ExchangeKey, instrument: InstrumentKey) -> bool:
        """Check if the instrument matches the filter."""
        ...


_ClientOrderIdBinding = _execution_bindings.ClientOrderId
_StrategyIdBinding = _execution_bindings.StrategyId


if TYPE_CHECKING:
    InstrumentFilter = _InstrumentFilterProtocol
else:  # pragma: no cover - runtime binding alias
    InstrumentFilter = _InstrumentFilterBinding


ClientOrderId = _ClientOrderIdBinding
StrategyId = _StrategyIdBinding


class Position:
    """Represents a trading position."""

    def __init__(
        self,
        instrument: InstrumentIndex,
        side: Side,
        quantity_abs: float,
        entry_price: float,
    ) -> None:
        self.instrument = instrument
        self.side = side
        self.quantity_abs = quantity_abs
        self.entry_price = entry_price

    def __repr__(self) -> str:
        return (
            f"Position("
            f"instrument={self.instrument!r}, "
            f"side={self.side!r}, "
            f"quantity_abs={self.quantity_abs!r}, "
            f"entry_price={self.entry_price!r}"
            f")"
        )


class InstrumentState:
    """State of a single instrument in the engine."""

    def __init__(
        self,
        instrument: InstrumentIndex,
        exchange: ExchangeIndex,
        position: Position | None,
        price: float | None,
    ) -> None:
        self.instrument = instrument
        self.exchange = exchange
        self.position = position
        self.price = price

    def __repr__(self) -> str:
        return (
            f"InstrumentState("
            f"instrument={self.instrument!r}, "
            f"exchange={self.exchange!r}, "
            f"position={self.position!r}, "
            f"price={self.price!r}"
            f")"
        )


class EngineState:
    """Simplified engine state for strategy operations."""

    def __init__(self, instruments: list[InstrumentState]) -> None:
        self.instruments = instruments

    def instruments_iter(
        self, filter: InstrumentFilter | None = None
    ) -> Iterable[InstrumentState]:
        """Iterate over instruments, optionally filtered."""
        for instrument in self.instruments:
            if filter is None or filter.matches(
                instrument.exchange, instrument.instrument
            ):
                yield instrument


_SIDE_BY_VALUE = {
    "buy": Side.BUY,
    "sell": Side.SELL,
}

_ORDER_KIND_BY_VALUE = {
    "market": OrderKind.MARKET,
    "limit": OrderKind.LIMIT,
}

_TIME_IN_FORCE_BY_VALUE = {
    "good_until_cancelled": TimeInForce.GOOD_UNTIL_CANCELLED,
    "good_until_end_of_day": TimeInForce.GOOD_UNTIL_END_OF_DAY,
    "fill_or_kill": TimeInForce.FILL_OR_KILL,
    "immediate_or_cancel": TimeInForce.IMMEDIATE_OR_CANCEL,
}


def _coerce_client_order_id(value: ClientOrderId | str) -> ClientOrderId:
    if isinstance(value, ClientOrderId):
        return value
    return ClientOrderId.new(str(value))


def _convert_binding_request(
    exchange: ExchangeIndex,
    instrument: InstrumentIndex,
    strategy_id: StrategyId,
    client_order_id: ClientOrderId,
    price: float,
    quantity: float,
    binding_request,
) -> OrderRequestOpen:
    side = _SIDE_BY_VALUE[binding_request.side]
    order_kind = _ORDER_KIND_BY_VALUE[binding_request.kind]
    time_in_force = _TIME_IN_FORCE_BY_VALUE[binding_request.time_in_force]

    key = OrderKey(
        exchange,
        instrument,
        strategy=strategy_id,
        cid=client_order_id,
    )

    state = RequestOpen(
        side=side,
        price=Decimal(str(price)),
        quantity=Decimal(str(quantity)),
        kind=order_kind,
        time_in_force=time_in_force,
    )

    return OrderRequestOpen(key, state)


def close_open_positions_with_market_orders(
    strategy_id: StrategyId,
    state: EngineState,
    filter: InstrumentFilter | None = None,
    gen_cid: Callable[[InstrumentState], ClientOrderId] | None = None,
) -> tuple[
    Iterable[OrderRequestCancel],
    Iterable[OrderRequestOpen],
]:
    """Naive strategy logic for closing open positions with market orders only.

    This function finds all open positions and generates equal but opposite Side market orders
    that will neutralise the position.

    Args:
        strategy_id: Strategy identifier for generated orders
        state: Current engine state
        filter: Optional instrument filter
        gen_cid: Function to generate client order IDs, defaults to using instrument index

    Returns:
        Tuple of (cancel_requests, open_requests)
    """
    cancel_requests: list[OrderRequestCancel] = []
    open_requests: list[OrderRequestOpen] = []

    for inst_state in state.instruments_iter(filter):
        position = inst_state.position
        if position is None:
            continue

        if position.quantity_abs <= 0:
            raise ValueError("position quantity must be positive")

        price = inst_state.price
        if price is None:
            raise ValueError("instrument price is required to close positions")

        cid_input: ClientOrderId | str | None
        if gen_cid is None:
            cid_input = None
        else:
            cid_input = gen_cid(inst_state)

        order = build_ioc_market_order_to_close_position(
            inst_state.exchange,
            position,
            strategy_id,
            price,
            gen_cid=cid_input,
        )
        open_requests.append(order)

    return cancel_requests, open_requests


class DefaultStrategy:
    """Naive implementation of all strategy interfaces.

    *THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION*.

    This strategy:
    - Generates no algorithmic orders (AlgoStrategy).
    - Closes positions via the naive close_open_positions_with_market_orders logic (ClosePositionsStrategy).
    - Does nothing when an exchange disconnects (OnDisconnectStrategy).
    - Does nothing when trading state is set to disabled (OnTradingDisabledStrategy).
    """

    def __init__(self, strategy_id: str = "default") -> None:
        self.id = StrategyId.new(strategy_id)

    @classmethod
    def default(cls) -> DefaultStrategy:
        return cls()

    def generate_algo_orders(self, state: State) -> tuple[list, list]:
        """Generate no algorithmic orders."""
        return ([], [])

    def close_positions_requests(
        self,
        state: EngineState,
        filter: InstrumentFilter | None = None,
    ) -> tuple[Iterable[OrderRequestCancel], Iterable[OrderRequestOpen]]:
        """Close positions using market orders."""
        return close_open_positions_with_market_orders(self.id, state, filter)

    def on_disconnect(self, exchange_id: str) -> None:
        """Do nothing when an exchange disconnects."""

    def on_trading_disabled(self) -> None:
        """Do nothing when trading is disabled."""


class OnDisconnectStrategy(Protocol):
    """Strategy interface for handling exchange disconnections."""

    def on_disconnect(self, exchange_id: str) -> None:
        """Handle actions when an exchange disconnects.

        Args:
            exchange_id: The exchange that disconnected
        """
        ...


class OnTradingDisabledStrategy(Protocol):
    """Strategy interface for handling trading state changes to disabled."""

    def on_trading_disabled(self) -> None:
        """Handle actions when trading is disabled."""
        ...


def cancel_all_orders_on_disconnect(exchange_id: str) -> list[OrderRequestCancel]:
    """Simple strategy: cancel all orders when an exchange disconnects.

    Args:
        exchange_id: The exchange that disconnected

    Returns:
        List of cancel requests for all orders on the exchange
    """
    # This is a simplified implementation - in practice, this would need
    # access to the current order state to generate specific cancel requests
    # For now, return empty list as a placeholder
    return []


def close_all_positions_on_trading_disabled() -> tuple[
    list[OrderRequestCancel], list[OrderRequestOpen]
]:
    """Simple strategy: close all positions when trading is disabled.

    Returns:
        Tuple of (cancel_requests, open_requests) to close all positions
    """
    # This is a simplified implementation - in practice, this would need
    # access to the current position state
    # For now, return empty lists as placeholders
    return ([], [])


def build_ioc_market_order_to_close_position(
    exchange: ExchangeIndex,
    position: Position,
    strategy_id: StrategyId,
    price: float,
    gen_cid: Callable[[], ClientOrderId] | ClientOrderId | str | None = None,
) -> OrderRequestOpen[ExchangeIndex, InstrumentIndex]:
    """Build an equal but opposite Side ImmediateOrCancel Market order that neutralises the provided Position.

    For example, if Position is LONG by 100, build a market order request to sell 100.

    Args:
        exchange: Exchange index
        position: Position to close
        strategy_id: Strategy identifier
        price: Current market price
        gen_cid: Function to generate client order ID

    Returns:
        Order request to close the position
    """
    if callable(gen_cid):
        cid_value = gen_cid()
    elif gen_cid is None:
        cid_value = ClientOrderId.new(f"close-{position.instrument}")
    else:
        cid_value = gen_cid

    client_order_id = _coerce_client_order_id(cid_value)

    binding_request = _build_ioc_market_order_to_close_position(
        exchange,
        position.instrument,
        position.side.value,
        position.quantity_abs,
        strategy_id,
        price,
        client_order_id,
    )

    return _convert_binding_request(
        exchange,
        position.instrument,
        strategy_id,
        client_order_id,
        price,
        position.quantity_abs,
        binding_request,
    )
