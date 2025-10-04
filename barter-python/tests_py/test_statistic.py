"""Unit tests for pure Python statistic module."""

from datetime import timedelta
from decimal import Decimal

from barter_python.statistic import (
    Annual252,
    Annual365,
    Daily,
    SharpeRatio,
    TimeDeltaInterval,
)


class TestAnnual365:
    def test_name(self):
        interval = Annual365()
        assert interval.name == "Annual(365)"

    def test_interval(self):
        interval = Annual365()
        assert interval.interval == timedelta(days=365)


class TestAnnual252:
    def test_name(self):
        interval = Annual252()
        assert interval.name == "Annual(252)"

    def test_interval(self):
        interval = Annual252()
        assert interval.interval == timedelta(days=252)


class TestDaily:
    def test_name(self):
        interval = Daily()
        assert interval.name == "Daily"

    def test_interval(self):
        interval = Daily()
        assert interval.interval == timedelta(days=1)


class TestTimeDeltaInterval:
    def test_name_minutes(self):
        delta = timedelta(hours=2)
        interval = TimeDeltaInterval(delta)
        assert interval.name == "Duration 120 (minutes)"

    def test_interval(self):
        delta = timedelta(hours=2)
        interval = TimeDeltaInterval(delta)
        assert interval.interval == delta

    def test_name_fractional_minutes(self):
        delta = timedelta(minutes=90)
        interval = TimeDeltaInterval(delta)
        assert interval.name == "Duration 90 (minutes)"


class TestSharpeRatio:
    def test_calculate_with_zero_std_dev(self):
        risk_free_return = Decimal('0.001')
        mean_return = Decimal('0.002')
        std_dev_returns = Decimal('0.0')
        time_period = TimeDeltaInterval(timedelta(hours=2))

        result = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )
        assert result.value == Decimal('1e1000')
        assert result.interval == time_period

    def test_calculate_with_custom_interval(self):
        # Define custom interval returns statistics
        risk_free_return = Decimal('0.0015')  # 0.15%
        mean_return = Decimal('0.0025')  # 0.25%
        std_dev_returns = Decimal('0.02')  # 2%
        time_period = TimeDeltaInterval(timedelta(hours=2))

        actual = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )

        expected_value = Decimal('0.05')
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_with_daily_interval(self):
        # Define daily returns statistics
        risk_free_return = Decimal('0.0015')  # 0.15%
        mean_return = Decimal('0.0025')  # 0.25%
        std_dev_returns = Decimal('0.02')  # 2%
        time_period = Daily()

        actual = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )

        expected_value = Decimal('0.05')
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_scale_from_daily_to_annual_252(self):
        input_ratio = SharpeRatio(
            value=Decimal('0.05'),
            interval=Daily(),
        )

        actual = input_ratio.scale(Annual252())

        # Expected value calculated with Python's precision
        expected_value = Decimal('0.79372539331937720')
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)