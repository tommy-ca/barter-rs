"""Python package wrapper exposing the Rust extension module and pure Python modules."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType

# Import pure Python modules
from . import (
    backtest,
    data,
    engine,
    execution,
    instrument,
    integration,
    risk,
    statistic,
    strategy,
)

_core: ModuleType = import_module(".barter_python", __name__)

__all__ = [name for name in dir(_core) if not name.startswith("_")]

# Add pure Python modules to __all__
__all__.extend(
    [
        "instrument",
        "execution",
        "data",
        "integration",
        "strategy",
        "statistic",
        "backtest",
        "risk",
        "engine",
    ]
)

ExecutionInstrumentMap = execution.ExecutionInstrumentMap
__all__.append("ExecutionInstrumentMap")

MockExecutionClient = execution.MockExecutionClient
__all__.append("MockExecutionClient")

Balance = execution.Balance
AssetBalance = execution.AssetBalance



def __getattr__(name: str):
    return getattr(_core, name)


def __dir__() -> list[str]:
    return sorted(__all__)
