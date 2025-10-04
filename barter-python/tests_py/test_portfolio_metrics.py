from __future__ import annotations

import datetime as dt
from decimal import Decimal

import pytest

import barter_python as bp


def test_calculate_sharpe_ratio_daily_interval() -> None:
    metric = bp.calculate_sharpe_ratio(
        risk_free_return=0.0015,
        mean_return=0.0025,
        std_dev_returns=0.02,
        interval="Daily",
    )

    assert metric.value == Decimal("0.05")
    assert metric.interval == "Daily"


def test_calculate_sharpe_ratio_timedelta_interval() -> None:
    metric = bp.calculate_sharpe_ratio(
        risk_free_return=0.0015,
        mean_return=0.0025,
        std_dev_returns=0.02,
        interval=dt.timedelta(hours=4),
    )

    assert metric.value == Decimal("0.05")
    assert metric.interval.startswith("Duration 240")


def test_calculate_sortino_ratio_zero_downside_positive_excess() -> None:
    metric = bp.calculate_sortino_ratio(
        risk_free_return=0.001,
        mean_return=0.002,
        std_dev_loss_returns=0.0,
        interval="Daily",
    )

    assert metric.value == Decimal("79228162514264337593543950335")
    assert metric.interval == "Daily"


def test_calculate_sortino_ratio_zero_downside_negative_excess() -> None:
    metric = bp.calculate_sortino_ratio(
        risk_free_return=0.002,
        mean_return=0.001,
        std_dev_loss_returns=0.0,
        interval="Daily",
    )

    assert metric.value == Decimal("-79228162514264337593543950335")
    assert metric.interval == "Daily"


def test_calculate_sortino_ratio_invalid_interval_raises() -> None:
    with pytest.raises(ValueError):
        bp.calculate_sortino_ratio(
            risk_free_return=0.001,
            mean_return=0.002,
            std_dev_loss_returns=0.015,
            interval="Weekly",
        )


def test_calculate_sharpe_ratio_rejects_non_finite_input() -> None:
    with pytest.raises(ValueError):
        bp.calculate_sharpe_ratio(
            risk_free_return=float("nan"),
            mean_return=0.0025,
            std_dev_returns=0.02,
            interval="Daily",
        )
