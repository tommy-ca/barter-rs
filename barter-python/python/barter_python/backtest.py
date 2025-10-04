"""Pure Python implementation of barter backtest module for historical trading simulations."""

from __future__ import annotations

import asyncio
import json
import time
from abc import abstractmethod
from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from pathlib import Path
from typing import AsyncIterable, Generic, Iterable, Protocol, TypeVar

from .data import MarketEvent, DataKind, PublicTrade, OrderBookL1, Candle, Liquidation, as_public_trade, as_candle
from .execution import AccountEvent
from .statistic import TimeInterval
from .strategy import AlgoStrategy, ClosePositionsStrategy, OnDisconnectStrategy, OnTradingDisabledStrategy, EngineState, InstrumentState, Position
from . import TradingSummary

# Type variables for generic backtest interfaces
MarketEventKind = TypeVar("MarketEventKind")
GlobalData = TypeVar("GlobalData")
InstrumentData = TypeVar("InstrumentData")
Strategy = TypeVar("Strategy")
Risk = TypeVar("Risk")
SummaryInterval = TypeVar("SummaryInterval", bound=TimeInterval)


class BacktestMarketData(Protocol[MarketEventKind]):
    """Protocol for market data sources used in backtests."""

    @abstractmethod
    async def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
        """Return an async iterable of market events for the backtest."""
        ...

    @abstractmethod
    async def time_first_event(self) -> datetime:
        """Return the timestamp of the first market event."""
        ...


@dataclass(frozen=True)
class MarketDataInMemory:
    """In-memory market data source for backtests."""

    _time_first_event: datetime
    events: list[MarketEvent[int, DataKind]]

    @classmethod
    def from_json_file(cls, path: Path) -> MarketDataInMemory:
        """Load market data from a JSON file."""
        with open(path, 'r') as f:
            data = json.load(f)

        events = []
        first_time = None

        for event_data in data:
            # Parse the market event from JSON - handle the Item/Ok wrapper
            if 'Item' in event_data and 'Ok' in event_data['Item']:
                inner_data = event_data['Item']['Ok']
            else:
                # Skip non-Item/Ok events (like Reconnecting)
                continue

            time_exchange = datetime.fromisoformat(inner_data['time_exchange'])
            time_received = datetime.fromisoformat(inner_data['time_received'])
            exchange = inner_data['exchange']
            instrument = inner_data['instrument']
            kind_data = inner_data['kind']

            # Parse the data kind
            if 'Trade' in kind_data:
                trade_data = kind_data['Trade']
                trade = PublicTrade(
                    id=trade_data['id'],
                    price=trade_data['price'],
                    amount=trade_data['amount'],
                    side=trade_data['side']
                )
                kind = DataKind.trade(trade)
            elif 'OrderBookL1' in kind_data:
                # Handle OrderBookL1 parsing
                l1_data = kind_data['OrderBookL1']
                # This would need more implementation
                continue  # Skip for now
            elif 'Candle' in kind_data:
                candle_data = kind_data['Candle']
                candle = Candle(
                    close_time=datetime.fromisoformat(candle_data['close_time']),
                    open=candle_data['open'],
                    high=candle_data['high'],
                    low=candle_data['low'],
                    close=candle_data['close'],
                    volume=candle_data['volume'],
                    trade_count=candle_data['trade_count']
                )
                kind = DataKind.candle(candle)
            elif 'Liquidation' in kind_data:
                liq_data = kind_data['Liquidation']
                liquidation = Liquidation(
                    side=liq_data['side'],
                    price=liq_data['price'],
                    quantity=liq_data['quantity'],
                    time=datetime.fromisoformat(liq_data['time'])
                )
                kind = DataKind.liquidation(liquidation)
            else:
                continue  # Skip unknown kinds

            event = MarketEvent(
                time_exchange=time_exchange,
                time_received=time_received,
                exchange=exchange,
                instrument=instrument,
                kind=kind
            )
            events.append(event)

            if first_time is None:
                first_time = time_exchange

        if not events:
            raise ValueError("No valid market events found in JSON file")

        return cls(_time_first_event=first_time, events=events)  # type: ignore

    async def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
        """Return an async iterable of market events."""
        for event in self.events:
            yield event

    async def time_first_event(self) -> datetime:
        """Return the timestamp of the first market event."""
        return self._time_first_event


@dataclass(frozen=True)
class BacktestSummary(Generic[SummaryInterval]):
    """Summary of a single backtest run."""

    id: str
    risk_free_return: Decimal
    trading_summary: TradingSummary


@dataclass(frozen=True)
class MultiBacktestSummary(Generic[SummaryInterval]):
    """Summary aggregating results from multiple concurrent backtests."""

    total_duration: float  # seconds
    summaries: list[BacktestSummary[SummaryInterval]]

    @classmethod
    def new(cls, duration: float, summaries: list[BacktestSummary[SummaryInterval]]) -> MultiBacktestSummary[SummaryInterval]:
        """Create a new MultiBacktestSummary."""
        return cls(total_duration=duration, summaries=summaries)


@dataclass(frozen=True)
class BacktestArgsConstant(Generic[MarketEventKind, SummaryInterval]):
    """Configuration for constants used across all backtests in a batch."""

    instruments: IndexedInstruments  # TODO: Define this
    executions: list[ExecutionConfig]  # TODO: Define this
    market_data: BacktestMarketData[MarketEventKind]
    summary_interval: SummaryInterval
    engine_state: EngineState


@dataclass(frozen=True)
class BacktestArgsDynamic(Generic[Strategy, Risk]):
    """Configuration for variables that can change between individual backtests."""

    id: str
    risk_free_return: Decimal
    strategy: Strategy
    risk: Risk


async def run_backtests(
    args_constant: BacktestArgsConstant,
    args_dynamic_iter: Iterable[BacktestArgsDynamic],
) -> MultiBacktestSummary:
    """Run multiple backtests concurrently, each with different strategy parameters.

    Args:
        args_constant: Shared constants for all backtests
        args_dynamic_iter: Iterator of different strategy configurations

    Returns:
        Aggregated results from all backtests
    """
    start_time = time.time()

    # Run all backtests concurrently
    tasks = [
        backtest(args_constant, args_dynamic)
        for args_dynamic in args_dynamic_iter
    ]

    summaries = await asyncio.gather(*tasks)

    total_duration = time.time() - start_time

    return MultiBacktestSummary.new(total_duration, summaries)


async def backtest(
    args_constant: BacktestArgsConstant,
    args_dynamic: BacktestArgsDynamic,
) -> BacktestSummary:
    """Run a single backtest with the given parameters.

    Args:
        args_constant: Shared constants for the backtest
        args_dynamic: Dynamic parameters for this specific backtest

    Returns:
        Summary of the backtest results
    """
    # Initialize engine simulator
    simulator = BacktestEngineSimulator(args_constant.engine_state)

    # Track start time
    time_start = await args_constant.market_data.time_first_event()

    # Process market events
    async for market_event in args_constant.market_data.stream():
        # Update engine state with market data
        await simulator.process_market_event(market_event)

        # Generate orders using strategy
        # cancel_requests, open_requests = args_dynamic.strategy.generate_algo_orders(simulator.state)

        # Process orders and account events
        # This would update positions, balances, etc.

    # Generate trading summary
    trading_summary = simulator.get_trading_summary(args_dynamic.risk_free_return, args_constant.summary_interval)

    return BacktestSummary(
        id=args_dynamic.id,
        risk_free_return=args_dynamic.risk_free_return,
        trading_summary=trading_summary,
    )


# TODO: Define placeholder types that need to be implemented
class IndexedInstruments:
    """Placeholder for indexed instruments."""
    pass


class ExecutionConfig:
    """Placeholder for execution configuration."""
    pass


class BacktestEngineSimulator:
    """Simple engine simulator for backtesting."""

    def __init__(self, initial_state: EngineState):
        self.state = initial_state
        self.current_time = None

    async def process_market_event(self, event: MarketEvent[int, DataKind]) -> None:
        """Process a market event and update engine state."""
        self.current_time = event.time_exchange

        # Update instrument prices based on market data
        if event.kind.kind == "trade":
            trade_event = as_public_trade(event)
            if trade_event:
                trade = trade_event.kind
                # Update price for the instrument
                for inst_state in self.state.instruments:
                    if inst_state.instrument == trade_event.instrument:
                        inst_state.price = trade.price
                        break
        elif event.kind.kind == "candle":
            candle_event = as_candle(event)
            if candle_event:
                candle = candle_event.kind
                # Update price with candle close
                for inst_state in self.state.instruments:
                    if inst_state.instrument == candle_event.instrument:
                        inst_state.price = candle.close
                        break

    def get_trading_summary(self, risk_free_return: Decimal, summary_interval: TimeInterval) -> TradingSummary:
        """Generate a trading summary from current state."""
        # This is a placeholder implementation
        # In a real implementation, this would calculate actual metrics
        return TradingSummary(
            time_engine_start=self.current_time or datetime.now(),
            time_engine_end=self.current_time or datetime.now(),
            instrument_tear_sheets=[],
            asset_tear_sheets=[],
            portfolio_tear_sheet=None,
        )
