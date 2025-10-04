"""Re-export risk bindings backed by the Rust extension module."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Protocol, TypeVar, Union

from .barter_python import (
    DefaultRiskManager,
    RiskApproved,
    RiskRefused,
    calculate_abs_percent_difference,
    calculate_delta,
    calculate_quote_notional,
)
from .execution import OrderRequestCancel, OrderRequestOpen

ExchangeKey = TypeVar("ExchangeKey")
InstrumentKey = TypeVar("InstrumentKey")
State = TypeVar("State")
OrderRequestType = TypeVar(
    "OrderRequestType", bound=Union[OrderRequestCancel, OrderRequestOpen]
)


class RiskManager(Protocol[ExchangeKey, InstrumentKey]):
    """Risk manager interface for typing purposes."""

    def check(
        self,
        state: State,
        cancels: Iterable[OrderRequestCancel[ExchangeKey, InstrumentKey]],
        opens: Iterable[OrderRequestOpen[ExchangeKey, InstrumentKey]],
    ) -> tuple[
        Iterable[RiskApproved[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskApproved[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
    ]:
        ...


__all__ = [
    "RiskApproved",
    "RiskRefused",
    "RiskManager",
    "DefaultRiskManager",
    "calculate_quote_notional",
    "calculate_abs_percent_difference",
    "calculate_delta",
]
