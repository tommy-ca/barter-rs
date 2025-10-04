"""Python package wrapper exposing the Rust extension module and pure Python modules."""

from __future__ import annotations

from importlib import import_module
from types import ModuleType

_core: ModuleType = import_module(".barter_python", __name__)

# Import pure Python modules
from . import instrument
from . import execution

__all__ = [name for name in dir(_core) if not name.startswith("_")]

# Add pure Python modules to __all__
__all__.extend(["instrument", "execution"])


def __getattr__(name: str):
    return getattr(_core, name)


def __dir__() -> list[str]:
    return sorted(__all__)
