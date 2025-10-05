"""Thin Python façade for the Rust backtest bindings."""

from __future__ import annotations

from collections.abc import AsyncIterable, Iterable
from datetime import datetime
from pathlib import Path
from typing import List

from .barter_python import (
    BacktestArgsConstant as _BacktestArgsConstant,
    BacktestArgsDynamic as _BacktestArgsDynamic,
    BacktestSummary as _BacktestSummary,
    ExecutionConfig as _ExecutionConfig,
    IndexedInstruments as _IndexedInstruments,
    MarketDataInMemory as _RustMarketDataInMemory,
    MultiBacktestSummary as _MultiBacktestSummary,
    MockExecutionConfig as _MockExecutionConfig,
    backtest as _backtest,
    run_backtests as _run_backtests,
)
from .data import DataKind, MarketEvent

BacktestArgsConstant = _BacktestArgsConstant
BacktestArgsDynamic = _BacktestArgsDynamic
BacktestSummary = _BacktestSummary
ExecutionConfig = _ExecutionConfig
IndexedInstruments = _IndexedInstruments
MultiBacktestSummary = _MultiBacktestSummary
MockExecutionConfig = _MockExecutionConfig


class MarketDataInMemory:
    """Python-friendly wrapper that retains Rust-backed market data."""

    def __init__(
        self,
        _time_first_event: datetime,
        events: List[MarketEvent[int, DataKind]],
        *,
        _inner: _RustMarketDataInMemory | None = None,
    ) -> None:
        self._time_first_event = _time_first_event
        self.events = events
        self._inner = _inner

    @classmethod
    def from_json_file(cls, path: str | Path) -> "MarketDataInMemory":
        """Load market data from a JSON file into memory."""
        inner = _RustMarketDataInMemory.from_json_file(str(path))
        events = inner.events()
        return cls(
            _time_first_event=inner.time_first_event,
            events=events,
            _inner=inner,
        )

    def stream(self) -> AsyncIterable[MarketEvent[int, DataKind]]:
        """Provide an async iterator over the buffered market events."""

        async def _generator():
            for event in self.events:
                yield event

        return _generator()

    async def time_first_event(self) -> datetime:
        """Return the timestamp of the first buffered market event."""
        return self._time_first_event

    def __len__(self) -> int:  # pragma: no cover - trivial wrapper
        return len(self.events)

    def __repr__(self) -> str:  # pragma: no cover - debugging helper
        return (
            "MarketDataInMemory(events={events}, time_first_event={time})".format(
                events=len(self.events),
                time=self._time_first_event.isoformat(),
            )
        )


def backtest(
    args_constant: BacktestArgsConstant,
    args_dynamic: BacktestArgsDynamic,
) -> BacktestSummary:
    """Run a single backtest synchronously while the GIL is released."""

    return _backtest(args_constant, args_dynamic)


def run_backtests(
    args_constant: BacktestArgsConstant,
    args_dynamics: Iterable[BacktestArgsDynamic],
) -> MultiBacktestSummary:
    """Run multiple backtests concurrently and aggregate the summaries."""

    return _run_backtests(args_constant, list(args_dynamics))


__all__ = [
    "BacktestArgsConstant",
    "BacktestArgsDynamic",
    "BacktestSummary",
    "ExecutionConfig",
    "IndexedInstruments",
    "MarketDataInMemory",
    "MultiBacktestSummary",
    "MockExecutionConfig",
    "backtest",
    "run_backtests",
]
