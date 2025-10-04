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


def test_calculate_calmar_ratio_daily_interval() -> None:
    metric = bp.calculate_calmar_ratio(
        risk_free_return=0.0015,
        mean_return=0.0025,
        max_drawdown=0.02,
        interval="Daily",
    )

    assert metric.value == Decimal("0.05")
    assert metric.interval == "Daily"


def test_calculate_calmar_ratio_timedelta_interval() -> None:
    metric = bp.calculate_calmar_ratio(
        risk_free_return=0.0015,
        mean_return=0.0025,
        max_drawdown=0.02,
        interval=dt.timedelta(hours=4),
    )

    assert metric.value == Decimal("0.05")
    assert metric.interval.startswith("Duration 240")


def test_calculate_calmar_ratio_zero_drawdown_positive_excess() -> None:
    metric = bp.calculate_calmar_ratio(
        risk_free_return=0.001,
        mean_return=0.002,
        max_drawdown=0.0,
        interval="Daily",
    )

    assert metric.value == Decimal("79228162514264337593543950335")
    assert metric.interval == "Daily"


def test_calculate_calmar_ratio_zero_drawdown_negative_excess() -> None:
    metric = bp.calculate_calmar_ratio(
        risk_free_return=0.002,
        mean_return=0.001,
        max_drawdown=0.0,
        interval="Daily",
    )

    assert metric.value == Decimal("-79228162514264337593543950335")
    assert metric.interval == "Daily"


def test_calculate_calmar_ratio_invalid_interval_raises() -> None:
    with pytest.raises(ValueError):
        bp.calculate_calmar_ratio(
            risk_free_return=0.001,
            mean_return=0.002,
            max_drawdown=0.02,
            interval="Weekly",
        )


def test_calculate_profit_factor_typical_case() -> None:
    factor = bp.calculate_profit_factor(
        profits_gross_abs=10.0,
        losses_gross_abs=5.0,
    )

    assert factor == Decimal("2")


def test_calculate_profit_factor_returns_none_when_no_activity() -> None:
    assert (
        bp.calculate_profit_factor(
            profits_gross_abs=0.0,
            losses_gross_abs=0.0,
        )
        is None
    )


def test_calculate_profit_factor_handles_perfect_performance() -> None:
    factor = bp.calculate_profit_factor(
        profits_gross_abs=5.0,
        losses_gross_abs=0.0,
    )

    assert factor == Decimal("79228162514264337593543950335")


def test_calculate_profit_factor_rejects_non_finite_input() -> None:
    with pytest.raises(ValueError):
        bp.calculate_profit_factor(
            profits_gross_abs=float("nan"),
            losses_gross_abs=1.0,
        )


def test_calculate_win_rate_typical_case() -> None:
    rate = bp.calculate_win_rate(wins=6.0, total=10.0)

    assert rate == Decimal("0.6")


def test_calculate_win_rate_zero_total_returns_none() -> None:
    assert bp.calculate_win_rate(wins=0.0, total=0.0) is None


def test_calculate_win_rate_rejects_non_finite_input() -> None:
    with pytest.raises(ValueError):
        bp.calculate_win_rate(wins=1.0, total=float("inf"))


def test_calculate_rate_of_return_daily_interval() -> None:
    metric = bp.calculate_rate_of_return(
        mean_return=0.01,
        interval="daily",
    )

    assert metric.value == Decimal("0.01")
    assert metric.interval == "Daily"


def test_calculate_rate_of_return_scale_to_annual() -> None:
    metric = bp.calculate_rate_of_return(
        mean_return=0.01,
        interval="daily",
        target_interval="annual_252",
    )

    assert metric.value == Decimal("2.52")
    assert metric.interval == "Annual(252)"


def test_generate_drawdown_series_produces_expected_periods() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)
    points = [
        (base, 100.0),
        (base + dt.timedelta(days=1), 110.0),
        (base + dt.timedelta(days=2), 90.0),
        (base + dt.timedelta(days=3), 115.0),
        (base + dt.timedelta(days=4), 105.0),
        (base + dt.timedelta(days=5), 95.0),
        (base + dt.timedelta(days=6), 120.0),
        (base + dt.timedelta(days=7), 118.0),
    ]

    drawdowns = bp.generate_drawdown_series(points)

    expected_values = [
        Decimal(20) / Decimal(110),
        Decimal(20) / Decimal(115),
        Decimal(2) / Decimal(120),
    ]
    quant = Decimal("1E-28")

    assert len(drawdowns) == 3
    assert [value.quantize(quant) for value in (item.value for item in drawdowns)] == [
        value.quantize(quant) for value in expected_values
    ]
    assert drawdowns[0].time_start == base + dt.timedelta(days=1)
    assert drawdowns[0].time_end == base + dt.timedelta(days=3)
    assert drawdowns[-1].time_start == base + dt.timedelta(days=6)
    assert drawdowns[-1].time_end == base + dt.timedelta(days=7)


def test_calculate_max_drawdown_returns_largest_period() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)
    points = [
        (base, 100.0),
        (base + dt.timedelta(days=1), 110.0),
        (base + dt.timedelta(days=2), 90.0),
        (base + dt.timedelta(days=3), 115.0),
        (base + dt.timedelta(days=4), 105.0),
        (base + dt.timedelta(days=5), 95.0),
        (base + dt.timedelta(days=6), 120.0),
        (base + dt.timedelta(days=7), 118.0),
    ]

    drawdown = bp.calculate_max_drawdown(points)

    assert drawdown is not None
    expected_value = (Decimal(20) / Decimal(110)).quantize(Decimal("1E-28"))
    assert drawdown.value.quantize(Decimal("1E-28")) == expected_value
    assert drawdown.time_start == base + dt.timedelta(days=1)
    assert drawdown.time_end == base + dt.timedelta(days=3)


def test_calculate_mean_drawdown_returns_average_value_and_duration() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)
    points = [
        (base, 100.0),
        (base + dt.timedelta(days=1), 110.0),
        (base + dt.timedelta(days=2), 90.0),
        (base + dt.timedelta(days=3), 115.0),
        (base + dt.timedelta(days=4), 105.0),
        (base + dt.timedelta(days=5), 95.0),
        (base + dt.timedelta(days=6), 120.0),
        (base + dt.timedelta(days=7), 118.0),
    ]

    mean = bp.calculate_mean_drawdown(points)

    assert mean is not None

    expected_values = [
        Decimal(20) / Decimal(110),
        Decimal(20) / Decimal(115),
        Decimal(2) / Decimal(120),
    ]
    quant = Decimal("1E-28")
    expected_mean = sum(expected_values) / Decimal(len(expected_values))

    assert mean.mean_drawdown.quantize(quant) == expected_mean.quantize(quant)
    assert mean.mean_duration == dt.timedelta(days=2)


def test_drawdown_helpers_return_empty_results_for_short_series() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)

    assert bp.generate_drawdown_series([]) == []
    assert bp.calculate_max_drawdown([]) is None
    assert bp.calculate_mean_drawdown([]) is None
    assert bp.generate_drawdown_series([(base, 100.0)]) == []
    assert bp.calculate_max_drawdown([(base, 100.0)]) is None
    assert bp.calculate_mean_drawdown([(base, 100.0)]) is None


def test_drawdown_helpers_validate_input_shape() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)

    with pytest.raises(ValueError):
        bp.generate_drawdown_series([(base,)])


def test_drawdown_helpers_reject_non_finite_values() -> None:
    base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)

    with pytest.raises(ValueError):
        bp.generate_drawdown_series([(base, float("nan"))])


def test_calculate_rate_of_return_scale_custom_interval() -> None:
    metric = bp.calculate_rate_of_return(
        mean_return=0.01,
        interval=dt.timedelta(hours=2),
        target_interval=dt.timedelta(hours=8),
    )

    assert metric.value == Decimal("0.04")
    assert metric.interval.startswith("Duration 480")


def test_calculate_rate_of_return_invalid_interval_raises() -> None:
    with pytest.raises(ValueError):
        bp.calculate_rate_of_return(
            mean_return=0.01,
            interval="weekly",
        )
