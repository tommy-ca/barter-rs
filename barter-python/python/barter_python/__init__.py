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

# Export execution classes
OrderId = execution.OrderId
StrategyId = execution.StrategyId
ClientOrderId = execution.ClientOrderId
OrderKey = execution.OrderKey
OrderKind = execution.OrderKind
TimeInForce = execution.TimeInForce
TradeId = execution.TradeId
Trade = execution.Trade
AssetFees = execution.AssetFees
Order = execution.Order
InstrumentAccountSnapshot = execution.InstrumentAccountSnapshot
AccountSnapshot = execution.AccountSnapshot
OrderEvent = execution.OrderEvent

# Export order state classes
OpenInFlight = execution.OpenInFlight
Open = execution.Open
CancelInFlight = execution.CancelInFlight
Cancelled = execution.Cancelled
OrderError = execution.OrderError
InactiveOrderState = execution.InactiveOrderState
OrderState = execution.OrderState

# Export request classes
RequestOpen = execution.RequestOpen
OrderResponseCancel = execution.OrderResponseCancel

# Export event classes
AccountEvent = execution.AccountEvent
AccountEventKind = execution.AccountEventKind

__all__.extend([
    "OrderId", "StrategyId", "ClientOrderId", "OrderKey", "OrderKind", "TimeInForce",
    "TradeId", "Trade", "AssetFees", "Order", "InstrumentAccountSnapshot", "AccountSnapshot", "OrderEvent",
    "OpenInFlight", "Open", "CancelInFlight", "Cancelled", "OrderError", "InactiveOrderState", "OrderState",
    "RequestOpen", "OrderResponseCancel",
    "AccountEvent", "AccountEventKind"
])

ENGINE_FEED_MODE_STREAM = "stream"
ENGINE_FEED_MODE_ITERATOR = "iterator"
ENGINE_FEED_MODES = (ENGINE_FEED_MODE_STREAM, ENGINE_FEED_MODE_ITERATOR)

__all__.extend(
    [
        "ENGINE_FEED_MODE_STREAM",
        "ENGINE_FEED_MODE_ITERATOR",
        "ENGINE_FEED_MODES",
    ]
)



def __getattr__(name: str):
    return getattr(_core, name)


def __dir__() -> list[str]:
    return sorted(__all__)
