"""Pure Python implementation of barter engine module for trading logic."""

from __future__ import annotations

from abc import abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from decimal import Decimal
from typing import Any, Generic, Protocol, TypeVar

from .data import Candle, MarketEvent, OrderBookL1, as_candle, as_public_trade
from .execution import (
    AccountEvent,
    AccountSnapshot,
    AssetBalance,
    CancelInFlight,
    Open,
    OpenInFlight,
    Order,
    OrderKey,
    OrderRequestCancel,
    OrderRequestOpen,
    OrderResponseCancel,
    OrderState,
    Trade,
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



@dataclass(frozen=True)
class DefaultGlobalData:
    """Default implementation of global data with no additional state."""



class InstrumentMarketData(Protocol):
    """Protocol for instrument-specific market data."""



@dataclass(frozen=True)
class DefaultInstrumentMarketData:
    """Default implementation of instrument market data."""

    last_price: Decimal | None = None
    last_update_time: datetime | None = None
    order_book_l1: OrderBookL1 | None = None
    recent_candle: Candle | None = None


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
    position: Position | None = None
    market_data: DefaultInstrumentMarketData = field(
        default_factory=DefaultInstrumentMarketData
    )
    orders: dict[OrderKey, Order] = field(default_factory=dict)

    @property
    def has_position(self) -> bool:
        """Check if the instrument has an open position."""
        return self.position is not None

    @property
    def position_quantity(self) -> Decimal:
        """Get the position quantity (positive for long, negative for short)."""
        if self.position is None:
            return Decimal("0")
        return (
            self.position.quantity_abs
            if self.position.side == "buy"
            else -self.position.quantity_abs
        )


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
    trading_state: TradingState = field(
        default_factory=lambda: TradingState(enabled=True)
    )
    balances: dict[str, AssetBalance] = field(default_factory=dict)

    def get_instrument_state(
        self, instrument: InstrumentIndex
    ) -> InstrumentState | None:
        """Get the state for a specific instrument."""
        return self.instruments.get(instrument)

    def update_instrument_state(
        self, instrument: InstrumentIndex, state: InstrumentState
    ) -> None:
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

    def execute(
        self, engine_state: EngineState
    ) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate algorithmic orders."""
        cancels, opens = self.strategy.generate_algo_orders(self.state)
        return (list(cancels), list(opens))


@dataclass
class ClosePositions(Generic[State]):
    """Action to close open positions."""

    strategy: ClosePositionsStrategy
    state: State
    instrument_filter: InstrumentFilter | None = None

    def execute(
        self, engine_state: EngineState
    ) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate orders to close positions."""
        cancels, opens = self.strategy.close_positions_requests(
            self.state, self.instrument_filter
        )
        return (list(cancels), list(opens))


@dataclass
class SendRequests:
    """Action to send order requests."""

    open_requests: list[OrderRequestOpen]
    cancel_requests: list[OrderRequestCancel]

    def execute(self, engine_state: EngineState) -> None:
        """Apply open and cancel requests to the engine state."""
        for cancel in self.cancel_requests:
            self._apply_cancel(engine_state, cancel)

        for open_request in self.open_requests:
            self._apply_open(engine_state, open_request)

    def _apply_open(
        self,
        engine_state: EngineState,
        open_request: OrderRequestOpen,
    ) -> None:
        instrument_id = open_request.key.instrument
        exchange_id = open_request.key.exchange

        instrument_state = engine_state.get_instrument_state(instrument_id)
        if instrument_state is None:
            instrument_state = InstrumentState(
                instrument=instrument_id,  # type: ignore[arg-type]
                exchange=exchange_id,  # type: ignore[arg-type]
            )
            engine_state.update_instrument_state(instrument_id, instrument_state)

        instrument_state.orders[open_request.key] = Order(
            open_request.key,
            open_request.state.side,
            open_request.state.price,
            open_request.state.quantity,
            open_request.state.kind,
            open_request.state.time_in_force,
            OrderState.active(OpenInFlight()),
        )

    def _apply_cancel(
        self,
        engine_state: EngineState,
        cancel_request: OrderRequestCancel,
    ) -> None:
        instrument_id = cancel_request.key.instrument
        instrument_state = engine_state.get_instrument_state(instrument_id)
        if instrument_state is None:
            return

        order = instrument_state.orders.get(cancel_request.key)
        if order is None:
            return

        current_state = order.state.state
        open_state = current_state if isinstance(current_state, Open) else None
        next_state = OrderState.active(CancelInFlight.new(open_state))

        instrument_state.orders[cancel_request.key] = Order(
            order.key,
            order.side,
            order.price,
            order.quantity,
            order.kind,
            order.time_in_force,
            next_state,
        )


@dataclass
class CancelOrders:
    """Action to cancel orders."""

    instrument_filter: InstrumentFilter | None = None

    def execute(self, engine_state: EngineState) -> list[OrderRequestCancel]:
        """Generate cancel requests for orders matching the filter."""
        cancel_requests = []
        for inst_state in engine_state.instruments.values():
            if self.instrument_filter is None or self.instrument_filter.matches(
                inst_state.exchange, inst_state.instrument
            ):
                for order_key, order in inst_state.orders.items():
                    # Get order ID if available
                    order_id = getattr(order.state.state, "id", None)
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

    def process_account_event(
        self,
        event: AccountEvent[ExchangeKey, AssetKey, InstrumentKey],
    ) -> None:
        """Process an account event and update engine state."""

        kind = event.kind.kind
        data = event.kind.data

        if kind == "snapshot" and isinstance(data, AccountSnapshot):
            self._apply_account_snapshot(event.exchange, data)
        elif kind == "balance_snapshot" and isinstance(data, AssetBalance):
            self._apply_balance_snapshot(data)
        elif kind == "order_snapshot" and isinstance(data, Order):
            self._apply_order_snapshot(event.exchange, data)
        elif kind == "order_cancelled" and isinstance(data, OrderResponseCancel):
            self._apply_order_cancelled(data)
        elif kind == "trade" and isinstance(data, Trade):
            self._apply_trade(event.exchange, data)
        else:
            raise ValueError(f"Unsupported account event kind: {kind}")

    def _apply_account_snapshot(
        self,
        exchange: ExchangeKey,
        snapshot: AccountSnapshot[ExchangeKey, AssetKey, InstrumentKey],
    ) -> None:
        """Refresh balances and orders from a full account snapshot."""

        balances_seq = snapshot.balances
        if callable(balances_seq):
            balances_seq = balances_seq()

        self.state.balances = {
            str(balance.asset): balance for balance in balances_seq
        }

        instruments_seq = snapshot.instruments
        if callable(instruments_seq):
            instruments_seq = instruments_seq()

        for instrument_snapshot in instruments_seq:
            instrument_id = instrument_snapshot.instrument
            existing_state = self.state.get_instrument_state(instrument_id)

            if existing_state is None:
                existing_state = InstrumentState(
                    instrument=instrument_id,
                    exchange=exchange,
                )

            orders_seq = instrument_snapshot.orders
            if callable(orders_seq):
                orders_seq = orders_seq()

            existing_state.orders = {
                order.key: order for order in orders_seq
            }

            self.state.update_instrument_state(instrument_id, existing_state)

    def _apply_balance_snapshot(self, balance: AssetBalance[AssetKey]) -> None:
        """Update a single balance snapshot."""

        self.state.balances[str(balance.asset)] = balance

    def _apply_order_snapshot(
        self,
        exchange: ExchangeKey,
        order: Order[ExchangeKey, InstrumentKey, AssetKey],
    ) -> None:
        """Upsert a single order snapshot into engine state."""

        instrument_id = order.key.instrument
        inst_state = self.state.get_instrument_state(instrument_id)

        if inst_state is None:
            inst_state = InstrumentState(
                instrument=instrument_id,
                exchange=exchange,
            )

        inst_state.orders[order.key] = order
        self.state.update_instrument_state(instrument_id, inst_state)

    def _apply_order_cancelled(
        self,
        response: OrderResponseCancel[ExchangeKey, AssetKey, InstrumentKey],
    ) -> None:
        """Remove cancelled order from engine state."""

        instrument_id = response.key.instrument
        inst_state = self.state.get_instrument_state(instrument_id)

        if inst_state is None:
            return

        inst_state.orders.pop(response.key, None)
        self.state.update_instrument_state(instrument_id, inst_state)

    def _apply_trade(
        self,
        exchange: ExchangeKey,
        trade: Trade[AssetKey, InstrumentKey],
    ) -> None:
        """Update instrument position from a trade event."""

        instrument_id = trade.instrument
        inst_state = self.state.get_instrument_state(instrument_id)

        if inst_state is None:
            inst_state = InstrumentState(
                instrument=instrument_id,
                exchange=exchange,
            )

        current_position = inst_state.position
        trade_signed_qty = (
            trade.quantity if trade.side.value == "buy" else -trade.quantity
        )

        if current_position is None:
            inst_state.position = Position(
                instrument=inst_state.instrument,
                side=trade.side.value,
                quantity_abs=abs(trade_signed_qty),
                entry_price=trade.price,
            )
        else:
            current_signed_qty = (
                current_position.quantity_abs
                if current_position.side == "buy"
                else -current_position.quantity_abs
            )
            new_signed_qty = current_signed_qty + trade_signed_qty

            if new_signed_qty == 0:
                inst_state.position = None
            else:
                if (current_signed_qty >= 0 and trade_signed_qty >= 0) or (
                    current_signed_qty <= 0 and trade_signed_qty <= 0
                ):
                    total_quantity = abs(current_signed_qty) + abs(trade_signed_qty)
                    weighted_price = (
                        (abs(current_signed_qty) * current_position.entry_price)
                        + (abs(trade_signed_qty) * trade.price)
                    ) / total_quantity
                    inst_state.position = Position(
                        instrument=inst_state.instrument,
                        side="buy" if new_signed_qty > 0 else "sell",
                        quantity_abs=abs(new_signed_qty),
                        entry_price=weighted_price,
                    )
                else:
                    if abs(trade_signed_qty) >= abs(current_signed_qty):
                        inst_state.position = (
                            None
                            if new_signed_qty == 0
                            else Position(
                                instrument=inst_state.instrument,
                                side="buy" if new_signed_qty > 0 else "sell",
                                quantity_abs=abs(new_signed_qty),
                                entry_price=trade.price,
                            )
                        )
                    else:
                        inst_state.position = Position(
                            instrument=inst_state.instrument,
                            side="buy" if current_signed_qty > 0 else "sell",
                            quantity_abs=abs(new_signed_qty),
                            entry_price=current_position.entry_price,
                        )

        self.state.update_instrument_state(instrument_id, inst_state)

    def generate_algo_orders(
        self,
    ) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate algorithmic orders using the strategy."""
        action = GenerateAlgoOrders(self.strategy, self.state)
        return action.execute(self.state)

    def close_positions(
        self, instrument_filter: InstrumentFilter | None = None
    ) -> tuple[list[OrderRequestCancel], list[OrderRequestOpen]]:
        """Generate orders to close positions."""
        if hasattr(self.strategy, "close_positions_requests"):
            action = ClosePositions(self.strategy, self.state, instrument_filter)
            return action.execute(self.state)
        return ([], [])

    def send_requests(
        self,
        open_requests: list[OrderRequestOpen],
        cancel_requests: list[OrderRequestCancel],
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
        action = SendRequests(approved_open_requests, approved_cancel_requests)  # type: ignore[arg-type]
        action.execute(self.state)

    def cancel_orders(self, instrument_filter: InstrumentFilter | None = None) -> None:
        """Cancel orders matching the filter."""
        action = CancelOrders(instrument_filter)
        cancel_requests = action.execute(self.state)
        self.send_requests([], cancel_requests)

    def set_trading_enabled(self, enabled: bool) -> None:
        """Enable or disable trading."""
        self.state.trading_state = TradingState(enabled=enabled)
