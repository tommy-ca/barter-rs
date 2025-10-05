"""Pure Python implementation of barter backtest module for historical trading simulations."""

from __future__ import annotations

import asyncio
import time
from abc import abstractmethod
from collections.abc import AsyncIterable, Iterable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from decimal import Decimal
from pathlib import Path
from typing import Any, Generic, Protocol, TypeVar

from .barter_python import (
    BacktestArgsConstant as _BacktestArgsConstant,
    BacktestArgsDynamic as _BacktestArgsDynamic,
    ExecutionConfig as _ExecutionConfig,
    MarketDataInMemory as _RustMarketDataInMemory,
    MockExecutionConfig as _MockExecutionConfig,
)

from .data import (
    Candle,
    DataKind,
    Liquidation,
    MarketEvent,
    PublicTrade,
    as_candle,
    as_public_trade,
)
from .engine import Engine
from .engine import EngineState as EngineEngineState
from .execution import OrderRequestCancel, OrderRequestOpen
from .instrument import (
    IndexedInstruments,
    InstrumentIndex,
    Side,
)
from .risk import RiskManager
from .statistic import (
    CalmarRatio,
    ProfitFactor,
    RateOfReturn,
    SharpeRatio,
    SortinoRatio,
    TimeInterval,
    WinRate,
)

ExecutionConfig = _ExecutionConfig
MockExecutionConfig = _MockExecutionConfig
BacktestArgsConstant = _BacktestArgsConstant
BacktestArgsDynamic = _BacktestArgsDynamic

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
    pnl_return: RateOfReturn[SummaryInterval]
    sharpe_ratio: SharpeRatio[SummaryInterval]
    sortino_ratio: SortinoRatio[SummaryInterval]
    calmar_ratio: CalmarRatio[SummaryInterval]
    pnl_drawdown: Drawdown | None
    pnl_drawdown_mean: MeanDrawdown | None
    pnl_drawdown_max: MaxDrawdown | None
    win_rate: WinRate | None
    profit_factor: ProfitFactor | None


@dataclass(frozen=True)
class TearSheetAsset:
    """Tear sheet summarizing asset changes."""

    balance_end: AssetBalance | None
    drawdown: Drawdown | None
    drawdown_mean: MeanDrawdown | None
    drawdown_max: MaxDrawdown | None


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


class BacktestMarketData(Protocol):
    """Protocol for market data sources used in backtests."""

    @abstractmethod
    def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
        """Return an async iterable of market events for the backtest."""
        ...

    @abstractmethod
    async def time_first_event(self) -> datetime:
        """Return the timestamp of the first market event."""
        ...


class MarketDataInMemory:
    """In-memory market data source for backtests."""

    def __init__(
        self,
        _time_first_event: datetime,
        events: list[MarketEvent[int, DataKind]],
        *,
        _inner: _RustMarketDataInMemory | None = None,
    ) -> None:
        self._time_first_event = _time_first_event
        self.events = events
        self._inner = _inner

    @classmethod
    def from_json_file(cls, path: Path) -> MarketDataInMemory:
        """Load market data from a JSON file using the Rust binding."""
        inner = _RustMarketDataInMemory.from_json_file(str(path))
        events = inner.events()
        return cls(
            _time_first_event=inner.time_first_event,
            events=events,
            _inner=inner,
        )

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
    def new(
        cls, duration: float, summaries: list[BacktestSummary[SummaryInterval]]
    ) -> MultiBacktestSummary[SummaryInterval]:
        """Create a new MultiBacktestSummary."""
        return cls(total_duration=duration, summaries=summaries)


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
        backtest(args_constant, args_dynamic) for args_dynamic in args_dynamic_iter
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
    simulator = BacktestEngineSimulator(
        args_constant.engine_state, args_dynamic.strategy, args_dynamic.risk
    )
    simulator.start_time = time_start

    # Process market events
    async for market_event in args_constant.market_data.stream():
        # Update engine state with market data
        await simulator.process_market_event(market_event)

        # TODO: Generate orders using strategy
        # TODO: Process orders and account events

    # Generate trading summary
    trading_summary = simulator.get_trading_summary(
        args_dynamic.risk_free_return, args_constant.summary_interval
    )

    return BacktestSummary(
        id=args_dynamic.id,
        risk_free_return=args_dynamic.risk_free_return,
        trading_summary=trading_summary,
    )


class BacktestEngineSimulator:
    """Simple engine simulator for backtesting."""

    def __init__(
        self,
        initial_engine_state: EngineEngineState,
        strategy: Any,
        risk_manager: RiskManager,
    ):
        self.engine: Engine = Engine(initial_engine_state, strategy, risk_manager)
        self.current_time: datetime | None = None
        self.start_time: datetime | None = None
        self.positions: dict[int, Any] = {}  # Track positions by instrument
        self.trades: list[Any] = []  # Track executed trades
        self.pnl_by_instrument: dict[int, Any] = {}  # Track PnL by instrument
        self.returns_series: list[Any] = []  # Track return series for calculations

    async def process_market_event(self, event: MarketEvent[int, DataKind]) -> None:
        """Process a market event and update engine state."""
        if self.start_time is None:
            self.start_time = event.time_exchange
        self.current_time = event.time_exchange

        # Update engine with market event
        self.engine.process_market_event(event)

        # Update instrument prices based on market data
        if event.kind.kind == "trade":
            trade_event = as_public_trade(event)
            if trade_event:
                # Update price for the instrument
                inst_state = self.engine.state.get_instrument_state(
                    InstrumentIndex(event.instrument)
                )
                if inst_state:
                    # Note: InstrumentState doesn't have price, so we store in a dict or something
                    pass  # TODO: Add price tracking to InstrumentState
        elif event.kind.kind == "candle":
            candle_event = as_candle(event)
            if candle_event:
                # Update price with candle close
                inst_state = self.engine.state.get_instrument_state(
                    InstrumentIndex(event.instrument)
                )
                if inst_state:
                    pass  # TODO: Add price tracking

        # Generate orders using strategy
        if self.engine.state.is_trading_enabled():
            cancel_requests, open_requests = self.engine.generate_algo_orders()
            self.simulate_order_execution(open_requests, cancel_requests)

    def record_trade(
        self,
        instrument: int,
        side: Side,
        quantity: Decimal,
        price: Decimal,
        pnl: Decimal = Decimal("0"),
    ):
        """Record a trade for tracking."""
        self.trades.append(
            {
                "instrument": instrument,
                "side": side,
                "quantity": quantity,
                "price": price,
                "pnl": pnl,
                "time": self.current_time,
            }
        )

        # Update PnL tracking
        if instrument not in self.pnl_by_instrument:
            self.pnl_by_instrument[instrument] = Decimal("0")
        self.pnl_by_instrument[instrument] += pnl

    def simulate_order_execution(
        self,
        open_requests: list[OrderRequestOpen],
        cancel_requests: list[OrderRequestCancel],
    ) -> None:
        """Simulate order execution for backtesting."""
        # For simplicity, assume all market orders fill immediately at current price
        for request in open_requests:
            if request.state.kind.value == "market":  # Assuming kind has value
                # Get current price (simplified - would need to get from market data)
                current_price = Decimal("100")  # Placeholder
                # Create a fill
                fill_quantity = request.state.quantity
                fill_price = current_price
                pnl = Decimal("0")  # Calculate based on position
                self.record_trade(
                    request.key.instrument,
                    request.state.side,
                    fill_quantity,
                    fill_price,
                    pnl,
                )
                # Update position
                # TODO: Update engine state positions

        # Cancel requests - just mark as cancelled
        for _cancel in cancel_requests:
            # TODO: Update order state to cancelled
            pass

    def get_trading_summary(
        self, risk_free_return: Decimal, summary_interval: TimeInterval
    ) -> TradingSummary:
        """Generate a trading summary from current state."""
        start_time = self.start_time or datetime.now()
        end_time = self.current_time or datetime.now()

        # Generate instrument tear sheets
        instruments = {}
        for inst_index, _inst_state in self.engine.state.instruments.items():
            instrument_name = f"instrument_{inst_index}"
            pnl = self.pnl_by_instrument.get(inst_index.index, Decimal("0"))

            # Create basic tear sheet with placeholder values
            tear_sheet = TearSheet(
                pnl=pnl,
                pnl_return=RateOfReturn.calculate(pnl, summary_interval),
                sharpe_ratio=SharpeRatio.calculate(
                    risk_free_return, pnl, Decimal("0.1"), summary_interval
                ),
                sortino_ratio=SortinoRatio.calculate(
                    risk_free_return, pnl, Decimal("0.05"), summary_interval
                ),
                calmar_ratio=CalmarRatio.calculate(
                    risk_free_return, pnl, Decimal("0.1"), summary_interval
                ),
                pnl_drawdown=None,  # Would need drawdown calculation
                pnl_drawdown_mean=None,
                pnl_drawdown_max=None,
                win_rate=WinRate.calculate(Decimal("5"), Decimal("10"))
                if self.trades
                else None,  # 50% win rate if trades exist
                profit_factor=ProfitFactor.calculate(Decimal("100"), Decimal("50"))
                if pnl > 0
                else None,
            )
            instruments[instrument_name] = tear_sheet

        # Generate asset tear sheets (simplified)
        assets: dict[str, Any] = {}
        # For now, just create empty asset tear sheets
        # In a full implementation, this would track asset balances

        return TradingSummary(
            time_engine_start=start_time,
            time_engine_end=end_time,
            instruments=instruments,
            assets=assets,
        )
