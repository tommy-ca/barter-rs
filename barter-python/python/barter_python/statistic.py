"""Pure Python implementation of barter statistic module for financial metrics."""

from __future__ import annotations

import math
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import timedelta
from decimal import Decimal
from typing import Generic, Protocol, TypeVar


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


IntervalT = TypeVar("IntervalT", bound=TimeInterval)


@dataclass(frozen=True)
class SharpeRatio(Generic[IntervalT]):
    """Sharpe Ratio value over a specific time interval.

    Sharpe Ratio measures the risk-adjusted return of an investment by comparing
    its excess returns (over risk-free rate) to its standard deviation.

    See docs: https://www.investopedia.com/articles/07/sharpe_ratio.asp
    """

    value: Decimal
    interval: IntervalT

    @classmethod
    def calculate(
        cls,
        risk_free_return: Decimal,
        mean_return: Decimal,
        std_dev_returns: Decimal,
        returns_period: IntervalT,
    ) -> SharpeRatio[IntervalT]:
        """Calculate the SharpeRatio over the provided time interval."""
        if std_dev_returns.is_zero():
            # Use a very large Decimal to represent MAX (similar to Decimal::MAX in Rust)
            return cls(value=Decimal('1e1000'), interval=returns_period)
        else:
            excess_returns = mean_return - risk_free_return
            ratio = excess_returns / std_dev_returns
            return cls(value=ratio, interval=returns_period)

    def scale(self, target: TimeInterval) -> SharpeRatio[TimeInterval]:
        """Scale the SharpeRatio from current interval to target interval.

        This scaling assumes returns are independently and identically distributed (IID).
        """
        # Determine scale factor: square root of number of Self Intervals in Target Intervals
        target_secs = Decimal(str(target.interval.total_seconds()))
        current_secs = Decimal(str(self.interval.interval.total_seconds()))

        scale_ratio = target_secs / current_secs
        scale = Decimal(str(math.sqrt(float(scale_ratio))))

        new_value = self.value * scale

        return SharpeRatio(value=new_value, interval=target)


@dataclass(frozen=True)
class SortinoRatio(Generic[IntervalT]):
    """Sortino Ratio value over a specific time interval.

    Similar to the Sharpe Ratio, but only considers downside volatility (standard deviation of
    negative returns) rather than total volatility. This makes it a better metric for portfolios
    with non-normal return distributions.
    """

    value: Decimal
    interval: IntervalT

    @classmethod
    def calculate(
        cls,
        risk_free_return: Decimal,
        mean_return: Decimal,
        std_dev_loss_returns: Decimal,
        returns_period: IntervalT,
    ) -> SortinoRatio[IntervalT]:
        """Calculate the SortinoRatio over the provided time interval."""
        if std_dev_loss_returns.is_zero():
            excess_returns = mean_return - risk_free_return
            if excess_returns > 0:
                value = Decimal('1e1000')  # Very large positive (like Decimal::MAX)
            elif excess_returns < 0:
                value = Decimal('-1e1000')  # Very large negative (like Decimal::MIN)
            else:
                value = Decimal('0')
            return cls(value=value, interval=returns_period)
        else:
            excess_returns = mean_return - risk_free_return
            ratio = excess_returns / std_dev_loss_returns
            return cls(value=ratio, interval=returns_period)

    def scale(self, target: TimeInterval) -> SortinoRatio[TimeInterval]:
        """Scale the SortinoRatio from current interval to target interval.

        This scaling assumes returns are independently and identically distributed (IID).
        However, this assumption may be less appropriate for downside deviation.
        """
        # Determine scale factor: square root of number of Self Intervals in Target Intervals
        target_secs = Decimal(str(target.interval.total_seconds()))
        current_secs = Decimal(str(self.interval.interval.total_seconds()))

        scale_ratio = target_secs / current_secs
        scale = Decimal(str(math.sqrt(float(scale_ratio))))

        new_value = self.value * scale

        return SortinoRatio(value=new_value, interval=target)


@dataclass(frozen=True)
class CalmarRatio(Generic[IntervalT]):
    """Calmar Ratio value over a specific time interval.

    The Calmar Ratio is a risk-adjusted return measure that divides the excess return
    (over risk-free rate) by the Maximum Drawdown risk. It's similar to the Sharpe and Sortino
    ratios, but uses Maximum Drawdown as the risk measure instead of standard deviation.

    See docs: https://corporatefinanceinstitute.com/resources/career-map/sell-side/capital-markets/calmar-ratio/
    """

    value: Decimal
    interval: IntervalT

    @classmethod
    def calculate(
        cls,
        risk_free_return: Decimal,
        mean_return: Decimal,
        max_drawdown: Decimal,
        returns_period: IntervalT,
    ) -> CalmarRatio[IntervalT]:
        """Calculate the CalmarRatio over the provided time interval."""
        if max_drawdown.is_zero():
            excess_returns = mean_return - risk_free_return
            if excess_returns > 0:
                value = Decimal('1e1000')  # Very large positive (like Decimal::MAX)
            elif excess_returns < 0:
                value = Decimal('-1e1000')  # Very large negative (like Decimal::MIN)
            else:
                value = Decimal('0')
            return cls(value=value, interval=returns_period)
        else:
            excess_returns = mean_return - risk_free_return
            ratio = excess_returns / abs(max_drawdown)
            return cls(value=ratio, interval=returns_period)

    def scale(self, target: TimeInterval) -> CalmarRatio[TimeInterval]:
        """Scale the CalmarRatio from current interval to target interval.

        This scaling assumes returns are independently and identically distributed (IID).
        However, this assumption is debatable since maximum drawdown may not scale with the square
        root of time like, for example, volatility does.
        """
        # Determine scale factor: square root of number of Self Intervals in Target Intervals
        target_secs = Decimal(str(target.interval.total_seconds()))
        current_secs = Decimal(str(self.interval.interval.total_seconds()))

        scale_ratio = target_secs / current_secs
        scale = Decimal(str(math.sqrt(float(scale_ratio))))

        new_value = self.value * scale

        return CalmarRatio(value=new_value, interval=target)