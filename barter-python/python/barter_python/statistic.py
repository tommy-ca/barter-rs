"""Pure Python implementation of barter statistic module for financial metrics."""

from __future__ import annotations

import math
from abc import abstractmethod
from dataclasses import dataclass
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Generic, Protocol, Sequence, TypeVar


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


@dataclass(frozen=True)
class ProfitFactor:
    """ProfitFactor is a performance metric that divides the absolute value of gross profits
    by the absolute value of gross losses. A profit factor greater than 1 indicates a profitable
    strategy.

    Special cases:
    - Returns None when both profits and losses are zero (neutral performance)
    - Returns a very large positive value when there are profits but no losses (perfect performance)
    - Returns a very large negative value when there are losses but no profits (worst performance)
    """

    value: Decimal

    @classmethod
    def calculate(cls, profits_gross_abs: Decimal, losses_gross_abs: Decimal) -> ProfitFactor | None:
        """Calculate the ProfitFactor given the provided gross profits and losses."""
        if profits_gross_abs.is_zero() and losses_gross_abs.is_zero():
            return None

        if losses_gross_abs.is_zero():
            value = Decimal('1e1000')  # Very large positive (like Decimal::MAX)
        elif profits_gross_abs.is_zero():
            value = Decimal('-1e1000')  # Very large negative (like Decimal::MIN)
        else:
            value = abs(profits_gross_abs) / abs(losses_gross_abs)

        return cls(value=value)


@dataclass(frozen=True)
class WinRate:
    """Represents a win rate ratio between 0 and 1, calculated as wins/total.

    The win rate is calculated as the absolute ratio of winning trades to total trades.

    Returns None if there are no trades (total = 0).
    """

    value: Decimal

    @classmethod
    def calculate(cls, wins: Decimal, total: Decimal) -> WinRate | None:
        """Calculate the WinRate given the provided number of wins and total positions."""
        if total == Decimal('0'):
            return None
        else:
            value = abs(wins) / abs(total)
            return cls(value=value)


@dataclass(frozen=True)
class RateOfReturn(Generic[IntervalT]):
    """Rate of Return value over a specific time interval.

    Rate of Return measures the percentage change in value over a time period.
    Unlike risk-adjusted metrics, returns scale linearly with time.
    """

    value: Decimal
    interval: IntervalT

    @classmethod
    def calculate(cls, mean_return: Decimal, returns_period: IntervalT) -> RateOfReturn[IntervalT]:
        """Calculate the RateOfReturn over the provided time interval."""
        return cls(value=mean_return, interval=returns_period)

    def scale(self, target: TimeInterval) -> RateOfReturn[TimeInterval]:
        """Scale the RateOfReturn from current interval to target interval.

        Unlike risk metrics which use square root scaling, RateOfReturn scales linearly
        with time.

        For example, a 1% daily return scales to approximately 252% annual return (not √252%).

        This assumes simple interest rather than compound interest.
        """
        # Determine scale factor: linear scaling of Self Intervals in Target Intervals
        target_secs = Decimal(str(target.interval.total_seconds()))
        current_secs = Decimal(str(self.interval.interval.total_seconds()))

        scale = target_secs / current_secs

        new_value = self.value * scale

        return RateOfReturn(value=new_value, interval=target)


@dataclass(frozen=True)
class Drawdown:
    """Drawdown represents the peak-to-trough decline of a value during a specific period.

    Drawdown is a measure of downside volatility.

    Attributes:
        value: The drawdown value as a decimal (negative for losses).
        time_start: The start time of the drawdown period.
        time_end: The end time of the drawdown period.
    """

    value: Decimal
    time_start: datetime
    time_end: datetime

    @property
    def duration(self) -> timedelta:
        """Time period of the drawdown."""
        return self.time_end - self.time_start


@dataclass(frozen=True)
class MaxDrawdown:
    """Maximum drawdown is the largest peak-to-trough decline of PnL or asset balance.

    Max Drawdown is a measure of downside risk, with larger values indicating
    downside movements could be volatile.
    """

    drawdown: Drawdown


@dataclass(frozen=True)
class MeanDrawdown:
    """Mean drawdown is the average drawdown value and duration from a collection of drawdowns."""

    mean_drawdown: Decimal
    mean_drawdown_ms: Decimal


@dataclass
class DrawdownGenerator:
    """Generator for calculating drawdowns from a series of equity points.

    Tracks peak values and calculates drawdown periods when the value recovers
    above the previous peak.
    """

    peak: Decimal | None = None
    drawdown_max: Decimal = Decimal('0')
    time_peak: datetime | None = None
    time_now: datetime = datetime.min

    @classmethod
    def init(cls, point: tuple[datetime, Decimal]) -> DrawdownGenerator:
        """Initialize from an initial timed value."""
        time, value = point
        return cls(
            peak=value,
            drawdown_max=Decimal('0'),
            time_peak=time,
            time_now=time,
        )

    def update(self, point: tuple[datetime, Decimal]) -> Drawdown | None:
        """Update the generator with a new point and return a completed drawdown if any."""
        time, value = point
        self.time_now = time

        if self.peak is None:
            self.peak = value
            self.time_peak = time
            return None

        peak = self.peak
        if value > peak:
            # Only emit a drawdown if one actually occurred
            ended_drawdown = self.generate()

            # Reset parameters
            self.peak = value
            self.time_peak = time
            self.drawdown_max = Decimal('0')

            return ended_drawdown
        else:
            # Calculate current drawdown
            if peak != 0:
                drawdown_current = (peak - value) / peak
                if drawdown_current > self.drawdown_max:
                    self.drawdown_max = drawdown_current

            return None

    def generate(self) -> Drawdown | None:
        """Generate the current drawdown if it is non-zero."""
        if self.time_peak is None or self.drawdown_max == 0:
            return None

        return Drawdown(
            value=self.drawdown_max,
            time_start=self.time_peak,
            time_end=self.time_now,
        )


@dataclass
class MaxDrawdownGenerator:
    """Generator for tracking the maximum drawdown over time."""

    max_drawdown: MaxDrawdown | None = None

    @classmethod
    def init(cls, drawdown: Drawdown) -> MaxDrawdownGenerator:
        """Initialize from an initial drawdown."""
        return cls(max_drawdown=MaxDrawdown(drawdown))

    def update(self, drawdown: Drawdown) -> None:
        """Update with a new drawdown, keeping the maximum."""
        if self.max_drawdown is None:
            self.max_drawdown = MaxDrawdown(drawdown)
        elif abs(drawdown.value) > abs(self.max_drawdown.drawdown.value):
            self.max_drawdown = MaxDrawdown(drawdown)

    def generate(self) -> MaxDrawdown | None:
        """Generate the current maximum drawdown."""
        return self.max_drawdown


@dataclass
class MeanDrawdownGenerator:
    """Generator for calculating the mean drawdown from a series of drawdowns."""

    count: int = 0
    mean_drawdown: MeanDrawdown | None = None

    @classmethod
    def init(cls, drawdown: Drawdown) -> MeanDrawdownGenerator:
        """Initialize from an initial drawdown."""
        return cls(
            count=1,
            mean_drawdown=MeanDrawdown(
                mean_drawdown=drawdown.value,
                mean_drawdown_ms=Decimal(str(drawdown.duration.total_seconds() * 1000)),
            )
        )

    def update(self, drawdown: Drawdown) -> None:
        """Update with a new drawdown, recalculating the mean."""
        self.count += 1

        if self.mean_drawdown is None:
            self.mean_drawdown = MeanDrawdown(
                mean_drawdown=drawdown.value,
                mean_drawdown_ms=Decimal(str(drawdown.duration.total_seconds() * 1000)),
            )
            return

        # Welford's online algorithm for mean
        prev_mean = self.mean_drawdown.mean_drawdown
        next_value = drawdown.value
        count = Decimal(str(self.count))

        new_mean_drawdown = prev_mean + (next_value - prev_mean) / count

        prev_mean_ms = self.mean_drawdown.mean_drawdown_ms
        next_ms = Decimal(str(drawdown.duration.total_seconds() * 1000))

        new_mean_ms = prev_mean_ms + (next_ms - prev_mean_ms) / Decimal(str(self.count))

        self.mean_drawdown = MeanDrawdown(
            mean_drawdown=new_mean_drawdown,
            mean_drawdown_ms=new_mean_ms,
        )

    def generate(self) -> MeanDrawdown | None:
        """Generate the current mean drawdown."""
        return self.mean_drawdown


def build_drawdown_series(points: list[tuple[datetime, Decimal]]) -> list[Drawdown]:
    """Build a series of drawdowns from equity points."""
    if not points:
        return []

    generator = DrawdownGenerator()
    drawdowns = []

    for point in points:
        if generator.peak is None:
            generator = DrawdownGenerator.init(point)
        else:
            if completed := generator.update(point):
                drawdowns.append(completed)

    # Add any remaining drawdown
    if remaining := generator.generate():
        drawdowns.append(remaining)

    return drawdowns


def generate_drawdown_series(points: Sequence[tuple[datetime, float | Decimal]]) -> list[Drawdown]:
    """Generate a series of drawdowns from equity points.

    Args:
        points: List of (datetime, value) tuples representing equity over time.

    Returns:
        List of Drawdown objects representing completed drawdown periods.
    """
    # Convert to Decimal and filter out invalid points
    parsed_points = []
    for time, value in points:
        if isinstance(value, float):
            if math.isnan(value) or math.isinf(value):
                continue
            value = Decimal(str(value))
        parsed_points.append((time, value))

    return build_drawdown_series(parsed_points)


def calculate_max_drawdown(points: Sequence[tuple[datetime, float | Decimal]]) -> MaxDrawdown | None:
    """Calculate the maximum drawdown from equity points.

    Args:
        points: List of (datetime, value) tuples representing equity over time.

    Returns:
        The maximum drawdown, or None if no drawdowns occurred.
    """
    drawdowns = generate_drawdown_series(points)
    if not drawdowns:
        return None

    generator = MaxDrawdownGenerator.init(drawdowns[0])
    for drawdown in drawdowns[1:]:
        generator.update(drawdown)

    return generator.generate()


def calculate_mean_drawdown(points: Sequence[tuple[datetime, float | Decimal]]) -> MeanDrawdown | None:
    """Calculate the mean drawdown from equity points.

    Args:
        points: List of (datetime, value) tuples representing equity over time.

    Returns:
        The mean drawdown statistics, or None if no drawdowns occurred.
    """
    drawdowns = generate_drawdown_series(points)
    if not drawdowns:
        return None

    generator = MeanDrawdownGenerator.init(drawdowns[0])
    for drawdown in drawdowns[1:]:
        generator.update(drawdown)

    return generator.generate()
