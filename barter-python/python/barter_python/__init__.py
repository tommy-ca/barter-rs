"""Python package wrapper exposing the Rust extension module and pure Python modules."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType

_core: ModuleType = import_module(".barter_python", __name__)

# Import pure Python modules
from . import (
    backtest,  # noqa: E402,F401
    data,  # noqa: E402,F401
    engine,  # noqa: E402,F401
    execution,  # noqa: E402,F401
    instrument,  # noqa: E402,F401
    integration,  # noqa: E402,F401
    risk,  # noqa: E402,F401
    statistic,  # noqa: E402,F401
    strategy,  # noqa: E402,F401
)

__all__ = [name for name in dir(_core) if not name.startswith("_")]

# Add pure Python modules to __all__
__all__.extend(["instrument", "execution", "data", "integration", "strategy", "statistic", "backtest", "risk", "engine"])


def __getattr__(name: str):
    return getattr(_core, name)


def __dir__() -> list[str]:
    return sorted(__all__)
