"""Pure Python implementation of barter backtest module for historical trading simulations."""

from __future__ import annotations

import asyncio
import json
import time
from abc import abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from decimal import Decimal
from pathlib import Path
from typing import AsyncIterable, Generic, Iterable, Optional, Protocol, TypeVar

from .data import MarketEvent, DataKind, PublicTrade, OrderBookL1, Candle, Liquidation, as_public_trade, as_candle
from .execution import AccountEvent
from .instrument import (
    Asset, AssetIndex, AssetNameInternal, ExchangeAsset, ExchangeId, ExchangeIndex,
    Instrument, InstrumentIndex, InstrumentNameInternal, Keyed, Underlying
)
from .statistic import TimeInterval, SharpeRatio, SortinoRatio, CalmarRatio, WinRate, ProfitFactor, RateOfReturn
from .strategy import AlgoStrategy, ClosePositionsStrategy, OnDisconnectStrategy, OnTradingDisabledStrategy, EngineState, InstrumentState, Position

# Type variables for generic backtest interfaces
MarketEventKind = TypeVar("MarketEventKind")
GlobalData = TypeVar("GlobalData")
InstrumentData = TypeVar("InstrumentData")
Strategy = TypeVar("Strategy")
Risk = TypeVar("Risk")
SummaryInterval = TypeVar("SummaryInterval", bound=TimeInterval)


# Supporting structures for TradingSummary

@dataclass(frozen=True)
class Balance:
    """Asset balance with total and free amounts."""
    total: Decimal
    free: Decimal

    @property
    def used(self) -> Decimal:
        return self.total - self.free


@dataclass(frozen=True)
class AssetBalance:
    """Asset balance with metadata."""
    asset: str  # Simplified to string for now
    balance: Balance
    time_exchange: datetime


@dataclass(frozen=True)
class Drawdown:
    """Drawdown measurement."""
    value: Decimal
    time_start: datetime
    time_end: datetime

    @property
    def duration(self) -> timedelta:
        return self.time_end - self.time_start


@dataclass(frozen=True)
class MeanDrawdown:
    """Mean drawdown measurement."""
    mean_drawdown: Decimal
    mean_drawdown_ms: int


@dataclass(frozen=True)
class MaxDrawdown:
    """Maximum drawdown wrapper."""
    drawdown: Drawdown


@dataclass(frozen=True)
class Range:
    """Value range."""
    min: Decimal
    max: Decimal


@dataclass(frozen=True)
class Dispersion:
    """Statistical dispersion measures."""
    range: Range
    recurrence_relation_m: Decimal
    variance: Decimal
    std_dev: Decimal


@dataclass(frozen=True)
class DataSetSummary:
    """Statistical summary of a dataset."""
    count: Decimal
    sum: Decimal
    mean: Decimal
    dispersion: Dispersion


@dataclass(frozen=True)
class PnLReturns:
    """PnL returns with statistical summaries."""
    pnl_raw: Decimal
    total: DataSetSummary
    losses: DataSetSummary


@dataclass(frozen=True)
class TearSheet(Generic[SummaryInterval]):
    """Tear sheet summarizing trading performance for an instrument."""
    pnl: Decimal
    pnl_return: "RateOfReturn[SummaryInterval]"
    sharpe_ratio: "SharpeRatio[SummaryInterval]"
    sortino_ratio: "SortinoRatio[SummaryInterval]"
    calmar_ratio: "CalmarRatio[SummaryInterval]"
    pnl_drawdown: Optional[Drawdown]
    pnl_drawdown_mean: Optional[MeanDrawdown]
    pnl_drawdown_max: Optional[MaxDrawdown]
    win_rate: Optional["WinRate"]
    profit_factor: Optional["ProfitFactor"]


@dataclass(frozen=True)
class TearSheetAsset:
    """Tear sheet summarizing asset changes."""
    balance_end: Optional[AssetBalance]
    drawdown: Optional[Drawdown]
    drawdown_mean: Optional[MeanDrawdown]
    drawdown_max: Optional[MaxDrawdown]


@dataclass(frozen=True)
class TradingSummary(Generic[SummaryInterval]):
    """Complete trading summary with instrument and asset tear sheets."""

    time_engine_start: datetime
    time_engine_end: datetime
    instruments: dict[str, TearSheet[SummaryInterval]] = field(default_factory=dict)
    assets: dict[str, TearSheetAsset] = field(default_factory=dict)

    @property
    def trading_duration(self) -> timedelta:
        return self.time_engine_end - self.time_engine_start


class BacktestMarketData(Protocol[MarketEventKind]):
    """Protocol for market data sources used in backtests."""

    @abstractmethod
    def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
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

    def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
        """Return an async iterable of market events."""
        async def _gen():
            for event in self.events:
                yield event
        return _gen()

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

    return MultiBacktestSummary(total_duration, summaries)


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
    # Track start time
    time_start = await args_constant.market_data.time_first_event()

    # Initialize simulator
    simulator = BacktestEngineSimulator(args_constant.engine_state)
    simulator.start_time = time_start

    # Process market events
    async for market_event in args_constant.market_data.stream():
        # Update engine state with market data
        await simulator.process_market_event(market_event)

        # TODO: Generate orders using strategy
        # TODO: Process orders and account events

    # Generate trading summary
    trading_summary = simulator.get_trading_summary(args_dynamic.risk_free_return, args_constant.summary_interval)

    return BacktestSummary(
        id=args_dynamic.id,
        risk_free_return=args_dynamic.risk_free_return,
        trading_summary=trading_summary,
    )


# Simplified IndexedInstruments for backtest use
class IndexedInstruments:
    """Indexed collection of instruments for backtest."""

    def __init__(self, instruments: list[Instrument[Asset]]):
        self._instruments = instruments

    @classmethod
    def new(cls, instruments: list[Instrument[Asset]]) -> IndexedInstruments:
        """Create IndexedInstruments from a list of instruments."""
        return cls(instruments)

    def instruments(self) -> list[Instrument[Asset]]:
        """Return the instruments."""
        return self._instruments


class MockExecutionConfig:
    """Configuration for mock execution."""

    def __init__(self, initial_balances=None):
        self.initial_balances = initial_balances or {}

class ExecutionConfig:
    """Configuration for execution links."""

    def __init__(self, mock_config: MockExecutionConfig):
        self.mock_config = mock_config

    @classmethod
    def mock(cls, mock_config: MockExecutionConfig) -> ExecutionConfig:
        return cls(mock_config)


class BacktestEngineSimulator:
    """Simple engine simulator for backtesting."""

    def __init__(self, initial_state: EngineState):
        self.state = initial_state
        self.current_time = None
        self.start_time = None
        self.positions = {}  # Track positions by instrument
        self.trades = []  # Track executed trades

    async def process_market_event(self, event: MarketEvent[int, DataKind]) -> None:
        """Process a market event and update engine state."""
        if self.start_time is None:
            self.start_time = event.time_exchange
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
        # Placeholder implementation - in a full implementation this would
        # calculate actual P&L, Sharpe ratios, etc. from trades and positions
        # For now, return empty tear sheets
        return TradingSummary(
            time_engine_start=self.start_time or datetime.now(),
            time_engine_end=self.current_time or datetime.now(),
            instruments={},
            assets={},
        )
