"""Helper utilities and CLI entry points shipped with barter-python."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType
from typing import Any

__all__ = ["backtest"]


def __getattr__(name: str) -> ModuleType:
    if name == "backtest":
        return import_module(".backtest", __name__)
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")


def __dir__() -> list[str]:
    return sorted(__all__ + [item for item in globals() if not item.startswith("__")])
