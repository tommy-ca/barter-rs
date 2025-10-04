"""Python package wrapper exposing the Rust extension module."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType

_core: ModuleType = import_module(".barter_python", __name__)

__all__ = [name for name in dir(_core) if not name.startswith("_")]


def __getattr__(name: str):
    return getattr(_core, name)


def __dir__() -> list[str]:
    return sorted(__all__)
