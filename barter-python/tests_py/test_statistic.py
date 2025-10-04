"""Unit tests for pure Python statistic module."""

from datetime import timedelta

from barter_python.statistic import (
    Annual252,
    Annual365,
    Daily,
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