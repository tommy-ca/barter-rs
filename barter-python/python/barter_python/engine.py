"""Pure Python implementation of barter engine module for trading logic."""

from __future__ import annotations

from abc import abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from decimal import Decimal
from typing import Any, Generic, Optional, Protocol, TypeVar

from .data import MarketEvent, OrderBookL1, Candle, as_public_trade, as_candle
from .execution import (
    AssetBalance,
    Order,
    OrderKey,
    OrderRequestCancel,
    OrderRequestOpen,
)
from .instrument import ExchangeId, InstrumentIndex
from .risk import RiskManager
from .strategy import AlgoStrategy, ClosePositionsStrategy, InstrumentFilter


class AllInstrumentsFilter:
    """Filter that matches all instruments."""

    def matches(self, exchange, instrument) -> bool:
        return True


class ExchangeFilter:
    """Filter that matches instruments on a specific exchange."""

    def __init__(self, exchange: int):
        self.exchange = exchange

    def matches(self, exchange, instrument) -> bool:
        return exchange == self.exchange

# Type variables for generic engine interfaces
ExchangeKey = TypeVar("ExchangeKey")
AssetKey = TypeVar("AssetKey")
InstrumentKey = TypeVar("InstrumentKey")
State = TypeVar("State")


class GlobalData(Protocol):
    """Protocol for global engine data."""

    pass


@dataclass(frozen=True)
class DefaultGlobalData:
    """Default implementation of global data with no additional state."""

    pass


class InstrumentMarketData(Protocol):
    """Protocol for instrument-specific market data."""

    pass


@dataclass(frozen=True)
class DefaultInstrumentMarketData:
    """Default implementation of instrument market data."""

    last_price: Optional[Decimal] = None
    last_update_time: Optional[datetime] = None
    order_book_l1: Optional[OrderBookL1] = None
    recent_candle: Optional[Candle] = None


@dataclass(frozen=True)
class Position:
    """Represents a trading position."""

    instrument: InstrumentIndex
    side: str  # "buy" or "sell"
    quantity_abs: Decimal
    entry_price: Decimal

    @property
    def value(self) -> Decimal:
        """Calculate the current value of the position."""
        return self.quantity_abs * self.entry_price


@dataclass
class InstrumentState:
    """State of a single instrument in the engine."""

    instrument: InstrumentIndex
    exchange: ExchangeId
    position: Optional[Position] = None
    market_data: DefaultInstrumentMarketData = field(default_factory=DefaultInstrumentMarketData)
    orders: dict[OrderKey, Order] = field(default_factory=dict)

    @property
    def has_position(self) -> bool:
        """Check if the instrument has an open position."""
        return self.position is not None

    @property
    def position_quantity(self) -> Decimal:
        """Get the position quantity (positive for long, negative for short)."""
        if self.position is None:
            return Decimal('0')
        return self.position.quantity_abs if self.position.side == "buy" else -self.position.quantity_abs


@dataclass
class TradingState:
    """Overall trading state of the engine."""

    enabled: bool = True

    @classmethod
    def trading_enabled(cls) -> TradingState:
        return cls(enabled=True)

    @classmethod
    def trading_disabled(cls) -> TradingState:
        return cls(enabled=False)


@dataclass
class EngineState:
    """Complete engine state combining all state types."""

    global_data: DefaultGlobalData = field(default_factory=DefaultGlobalData)
    instruments: dict[InstrumentIndex, InstrumentState] = field(default_factory=dict)
    trading_state: TradingState = field(default_factory=lambda: TradingState(enabled=True))
    balances: dict[str, AssetBalance] = field(default_factory=dict)

    def get_instrument_state(self, instrument: InstrumentIndex) -> Optional[InstrumentState]:
        """Get the state for a specific instrument."""
        return self.instruments.get(instrument)

    def update_instrument_state(self, instrument: InstrumentIndex, state: InstrumentState) -> None:
        """Update the state for a specific instrument."""
        self.instruments[instrument] = state

    def is_trading_enabled(self) -> bool:
        """Check if trading is currently enabled."""
        return self.trading_state.enabled


class EngineAction(Protocol):
    """Protocol for engine actions that can be executed."""

    @abstractmethod
    def execute(self, state: EngineState) -> None:
        """Execute the action on the engine state."""
        ...


@dataclass
class GenerateAlgoOrders(Generic[State]):
    """Action to generate algorithmic orders using a strategy."""

    strategy: AlgoStrategy
    state: State

    def execute(self, engine_state: EngineState) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate algorithmic orders."""
        cancels, opens = self.strategy.generate_algo_orders(self.state)
        return (list(cancels), list(opens))


@dataclass
class ClosePositions(Generic[State]):
    """Action to close open positions."""

    strategy: ClosePositionsStrategy
    state: State
    instrument_filter: Optional[InstrumentFilter] = None

    def execute(self, engine_state: EngineState) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate orders to close positions."""
        cancels, opens = self.strategy.close_positions_requests(self.state, self.instrument_filter)
        return (list(cancels), list(opens))


@dataclass
class SendRequests:
    """Action to send order requests."""

    open_requests: list[OrderRequestOpen]
    cancel_requests: list[OrderRequestCancel]

    def execute(self, engine_state: EngineState) -> None:
        """Send the requests (placeholder for actual execution)."""
        # TODO: Integrate with execution layer
        pass


@dataclass
class CancelOrders:
    """Action to cancel orders."""

    instrument_filter: Optional[InstrumentFilter] = None  # TODO: Define proper filter

    def execute(self, engine_state: EngineState) -> list[OrderRequestCancel]:
        """Generate cancel requests for orders matching the filter."""
        cancel_requests = []
        for inst_state in engine_state.instruments.values():
            if self.instrument_filter is None or self.instrument_filter.matches(inst_state.exchange, inst_state.instrument):
                for order_key, order in inst_state.orders.items():
                    # Get order ID if available
                    order_id = None
                    if order.state.is_active() and hasattr(order.state.state, 'id'):
                        order_id = order.state.state.id
                    # Create cancel request for this order
                    cancel_request = OrderRequestCancel(key=order_key, state=order_id)
                    cancel_requests.append(cancel_request)
        return cancel_requests


class Engine(Generic[State]):
    """Main trading engine coordinating state and actions."""

    def __init__(
        self,
        initial_state: EngineState,
        strategy: Any,  # Strategy that may implement AlgoStrategy and/or ClosePositionsStrategy
        risk_manager: RiskManager,
    ):
        self.state = initial_state
        self.strategy = strategy
        self.risk_manager = risk_manager

    def process_market_event(self, event: MarketEvent) -> None:
        """Process a market event and update engine state."""
        # Update instrument market data
        if event.instrument in self.state.instruments:
            inst_state = self.state.instruments[event.instrument]
            market_data = inst_state.market_data

            # Update market data based on event kind
            if event.kind.kind == "trade":
                trade_event = as_public_trade(event)
                if trade_event:
                    market_data = DefaultInstrumentMarketData(
                        last_price=Decimal(str(trade_event.kind.price)),
                        last_update_time=event.time_exchange,
                        order_book_l1=market_data.order_book_l1,
                        recent_candle=market_data.recent_candle,
                    )
            elif event.kind.kind == "candle":
                candle_event = as_candle(event)
                if candle_event:
                    market_data = DefaultInstrumentMarketData(
                        last_price=Decimal(str(candle_event.kind.close)),
                        last_update_time=event.time_exchange,
                        order_book_l1=market_data.order_book_l1,
                        recent_candle=candle_event.kind,
                    )
            elif event.kind.kind == "order_book_l1":
                # Note: as_order_book_l1 is not defined, but we can access directly
                if event.kind.data and isinstance(event.kind.data, OrderBookL1):
                    market_data = DefaultInstrumentMarketData(
                        last_price=market_data.last_price,
                        last_update_time=event.time_exchange,
                        order_book_l1=event.kind.data,
                        recent_candle=market_data.recent_candle,
                    )

            # Update the instrument state with new market data
            updated_inst_state = InstrumentState(
                instrument=inst_state.instrument,
                exchange=inst_state.exchange,
                position=inst_state.position,
                market_data=market_data,
                orders=inst_state.orders,
            )
            self.state.update_instrument_state(event.instrument, updated_inst_state)

    def process_account_event(self, event: object) -> None:  # TODO: Define AccountEvent
        """Process an account event and update engine state."""
        # TODO: Update balances, orders, positions from account event
        pass

    def generate_algo_orders(self) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate algorithmic orders using the strategy."""
        action = GenerateAlgoOrders(self.strategy, self.state)
        return action.execute(self.state)

    def close_positions(self, instrument_filter: Optional[InstrumentFilter] = None) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate orders to close positions."""
        if hasattr(self.strategy, 'close_positions_requests'):
            action = ClosePositions(self.strategy, self.state, instrument_filter)
            return action.execute(self.state)
        return ([], [])

    def send_requests(
        self,
        open_requests: list[OrderRequestOpen],
        cancel_requests: list[OrderRequestCancel]
    ) -> None:
        """Send order requests after risk checking."""
        # Apply risk management
        approved_cancels, approved_opens, _, _ = self.risk_manager.check(
            self.state, cancel_requests, open_requests
        )

        # Extract the approved items
        approved_cancel_requests = [approved.item for approved in approved_cancels]
        approved_open_requests = [approved.item for approved in approved_opens]

        # Send approved requests
        action = SendRequests(approved_open_requests, approved_cancel_requests)  # type: ignore
        action.execute(self.state)

    def cancel_orders(self, instrument_filter: Optional[InstrumentFilter] = None) -> None:
        """Cancel orders matching the filter."""
        action = CancelOrders(instrument_filter)
        cancel_requests = action.execute(self.state)
        self.send_requests([], cancel_requests)

    def set_trading_enabled(self, enabled: bool) -> None:
        """Enable or disable trading."""
        self.state.trading_state = TradingState(enabled=enabled)
