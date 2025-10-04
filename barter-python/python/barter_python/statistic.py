"""Pure Python implementation of barter statistic module for financial metrics."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import timedelta
from decimal import Decimal
from typing import Protocol


class TimeInterval(Protocol):
    """Protocol for types that represent time intervals used in financial calculations."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Human-readable name of the time interval."""
        ...

    @property
    @abstractmethod
    def interval(self) -> timedelta:
        """The timedelta representing this interval."""
        ...


@dataclass(frozen=True)
class Annual365:
    """Annual time interval with 365 days (crypto markets, 24/7 trading)."""

    @property
    def name(self) -> str:
        return "Annual(365)"

    @property
    def interval(self) -> timedelta:
        return timedelta(days=365)


@dataclass(frozen=True)
class Annual252:
    """Annual time interval with 252 days (traditional markets, trading days)."""

    @property
    def name(self) -> str:
        return "Annual(252)"

    @property
    def interval(self) -> timedelta:
        return timedelta(days=252)


@dataclass(frozen=True)
class Daily:
    """Daily time interval."""

    @property
    def name(self) -> str:
        return "Daily"

    @property
    def interval(self) -> timedelta:
        return timedelta(days=1)


@dataclass(frozen=True)
class TimeDeltaInterval:
    """Custom time interval based on a timedelta."""

    delta: timedelta

    @property
    def name(self) -> str:
        return f"Duration {self.delta.total_seconds() / 60:.0f} (minutes)"

    @property
    def interval(self) -> timedelta:
        return self.delta