"""Unit tests for pure Python statistic module."""

from datetime import datetime, timedelta, timezone
from decimal import Decimal

from barter_python.statistic import (
    Annual252,
    Annual365,
    CalmarRatio,
    Daily,
    Drawdown,
    MaxDrawdown,
    MeanDrawdown,
    ProfitFactor,
    RateOfReturn,
    SharpeRatio,
    SortinoRatio,
    TimeDeltaInterval,
    WinRate,
    build_drawdown_series,
    calculate_max_drawdown,
    calculate_mean_drawdown,
    generate_drawdown_series,
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
        risk_free_return = Decimal("0.001")
        mean_return = Decimal("0.002")
        std_dev_returns = Decimal("0.0")
        time_period = TimeDeltaInterval(timedelta(hours=2))

        result = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )
        assert result.value == Decimal("1e1000")
        assert result.interval == time_period

    def test_calculate_with_custom_interval(self):
        # Define custom interval returns statistics
        risk_free_return = Decimal("0.0015")  # 0.15%
        mean_return = Decimal("0.0025")  # 0.25%
        std_dev_returns = Decimal("0.02")  # 2%
        time_period = TimeDeltaInterval(timedelta(hours=2))

        actual = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )

        expected_value = Decimal("0.05")
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_with_daily_interval(self):
        # Define daily returns statistics
        risk_free_return = Decimal("0.0015")  # 0.15%
        mean_return = Decimal("0.0025")  # 0.25%
        std_dev_returns = Decimal("0.02")  # 2%
        time_period = Daily()

        actual = SharpeRatio.calculate(
            risk_free_return, mean_return, std_dev_returns, time_period
        )

        expected_value = Decimal("0.05")
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_scale_from_daily_to_annual_252(self):
        input_ratio = SharpeRatio(
            value=Decimal("0.05"),
            interval=Daily(),
        )

        actual = input_ratio.scale(Annual252())

        # Expected value calculated with Python's precision
        expected_value = Decimal("0.79372539331937720")
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)


class TestSortinoRatio:
    def test_calculate_normal_case(self):
        # Define test case with reasonable values
        risk_free_return = Decimal("0.0015")  # 0.15%
        mean_return = Decimal("0.0025")  # 0.25%
        std_dev_loss_returns = Decimal("0.02")  # 2%
        time_period = Daily()

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        expected_value = Decimal("0.05")  # (0.0025 - 0.0015) / 0.02
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_zero_downside_dev_positive_excess(self):
        # Test case: positive excess returns with no downside risk
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("0.002")  # 0.2%
        std_dev_loss_returns = Decimal("0.0")
        time_period = Daily()

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        assert actual.value == Decimal("1e1000")
        assert actual.interval == time_period

    def test_calculate_zero_downside_dev_negative_excess(self):
        # Test case: negative excess returns with no downside risk
        risk_free_return = Decimal("0.002")  # 0.2%
        mean_return = Decimal("0.001")  # 0.1%
        std_dev_loss_returns = Decimal("0.0")
        time_period = Daily()

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        assert actual.value == Decimal("-1e1000")
        assert actual.interval == time_period

    def test_calculate_zero_downside_dev_no_excess(self):
        # Test case: no excess returns with no downside risk
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("0.001")  # 0.1%
        std_dev_loss_returns = Decimal("0.0")
        time_period = Daily()

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        assert actual.value == Decimal("0.0")
        assert actual.interval == time_period

    def test_calculate_negative_returns(self):
        # Test case: negative mean returns
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("-0.002")  # -0.2%
        std_dev_loss_returns = Decimal("0.015")  # 1.5%
        time_period = Daily()

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        expected_value = Decimal("-0.2")  # (-0.002 - 0.001) / 0.015
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_custom_interval(self):
        # Test case with custom time interval
        risk_free_return = Decimal("0.0015")  # 0.15%
        mean_return = Decimal("0.0025")  # 0.25%
        std_dev_loss_returns = Decimal("0.02")  # 2%
        time_period = TimeDeltaInterval(timedelta(hours=4))

        actual = SortinoRatio.calculate(
            risk_free_return,
            mean_return,
            std_dev_loss_returns,
            time_period,
        )

        expected_value = Decimal("0.05")
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_scale_daily_to_annual(self):
        # Test scaling from daily to annual
        daily = SortinoRatio(
            value=Decimal("0.05"),
            interval=Daily(),
        )

        actual = daily.scale(Annual252())

        # 0.05 * √252 ≈ 0.7937
        expected_value = Decimal("0.79372539331937720")
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)


class TestCalmarRatio:
    def test_calculate_normal_case(self):
        risk_free_return = Decimal("0.0015")  # 0.15%
        mean_return = Decimal("0.0025")  # 0.25%
        max_drawdown = Decimal("0.02")  # 2%
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return, mean_return, max_drawdown, time_period
        )

        expected_value = Decimal("0.05")  # (0.0025 - 0.0015) / 0.02
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_zero_drawdown_positive_excess(self):
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("0.002")  # 0.2%
        max_drawdown = Decimal("0.0")  # 0%
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return, mean_return, max_drawdown, time_period
        )

        assert actual.value == Decimal("1e1000")
        assert actual.interval == time_period

    def test_calculate_zero_drawdown_negative_excess(self):
        risk_free_return = Decimal("0.002")  # 0.2%
        mean_return = Decimal("0.001")  # 0.1%
        max_drawdown = Decimal("0.0")  # 0%
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return, mean_return, max_drawdown, time_period
        )

        assert actual.value == Decimal("-1e1000")
        assert actual.interval == time_period

    def test_calculate_zero_drawdown_no_excess(self):
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("0.001")  # 0.1%
        max_drawdown = Decimal("0.0")  # 0%
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return, mean_return, max_drawdown, time_period
        )

        assert actual.value == Decimal("0.0")
        assert actual.interval == time_period

    def test_calculate_negative_returns(self):
        risk_free_return = Decimal("0.001")  # 0.1%
        mean_return = Decimal("-0.002")  # -0.2%
        max_drawdown = Decimal("0.015")  # 1.5%
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return, mean_return, max_drawdown, time_period
        )

        expected_value = Decimal("-0.2")  # (-0.002 - 0.001) / 0.015
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_calculate_absolute_drawdown(self):
        # Test that negative drawdown values are handled correctly (absolute value is used)
        risk_free_return = Decimal("0.001")
        mean_return = Decimal("0.002")
        negative_drawdown = Decimal(
            "-0.015"
        )  # Should be treated same as positive 0.015
        time_period = Daily()

        actual = CalmarRatio.calculate(
            risk_free_return,
            mean_return,
            negative_drawdown,
            time_period,
        )

        expected_value = Decimal(
            "0.06666666666666666666666666667"
        )  # (0.002 - 0.001) / 0.015
        assert actual.value == expected_value
        assert actual.interval == time_period

    def test_scale_daily_to_annual(self):
        daily = CalmarRatio(
            value=Decimal("0.05"),
            interval=Daily(),
        )

        actual = daily.scale(Annual252())

        # 0.05 * sqrt(252) ≈ 0.7937
        expected_value = Decimal("0.79372539331937720")
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)


class TestProfitFactor:
    def test_calculate_both_zero(self):
        result = ProfitFactor.calculate(Decimal("0.0"), Decimal("0.0"))
        assert result is None

    def test_calculate_profits_zero(self):
        result = ProfitFactor.calculate(Decimal("0.0"), Decimal("1.0"))
        assert result is not None
        assert result.value == Decimal("-1e1000")

    def test_calculate_losses_zero(self):
        result = ProfitFactor.calculate(Decimal("1.0"), Decimal("0.0"))
        assert result is not None
        assert result.value == Decimal("1e1000")

    def test_calculate_normal_case(self):
        result = ProfitFactor.calculate(Decimal("10.0"), Decimal("5.0"))
        assert result is not None
        assert result.value == Decimal("2.0")

    def test_calculate_with_negative_inputs(self):
        result = ProfitFactor.calculate(Decimal("10.0"), Decimal("-5.0"))
        assert result is not None
        assert result.value == Decimal("2.0")


class TestWinRate:
    def test_calculate_no_trades(self):
        result = WinRate.calculate(Decimal("0"), Decimal("0"))
        assert result is None

    def test_calculate_all_wins(self):
        result = WinRate.calculate(Decimal("10"), Decimal("10"))
        assert result is not None
        assert result.value == Decimal("1")

    def test_calculate_no_wins(self):
        result = WinRate.calculate(Decimal("0"), Decimal("10"))
        assert result is not None
        assert result.value == Decimal("0")

    def test_calculate_mixed(self):
        result = WinRate.calculate(Decimal("6"), Decimal("10"))
        assert result is not None
        assert result.value == Decimal("0.6")


class TestRateOfReturn:
    def test_calculate_normal_case(self):
        mean_return = Decimal("0.0025")  # 0.25%
        time_period = Daily()

        actual = RateOfReturn.calculate(mean_return, time_period)

        assert actual.value == Decimal("0.0025")
        assert actual.interval == time_period

    def test_calculate_zero(self):
        mean_return = Decimal("0.0")
        time_period = Daily()

        actual = RateOfReturn.calculate(mean_return, time_period)

        assert actual.value == Decimal("0.0")
        assert actual.interval == time_period

    def test_calculate_negative(self):
        mean_return = Decimal("-0.0025")  # -0.25%
        time_period = Daily()

        actual = RateOfReturn.calculate(mean_return, time_period)

        assert actual.value == Decimal("-0.0025")
        assert actual.interval == time_period

    def test_scale_daily_to_annual(self):
        # For returns, we use linear scaling (multiply by 252) not square root scaling
        daily = RateOfReturn(
            value=Decimal("0.01"),  # 1% daily return
            interval=Daily(),
        )

        actual = daily.scale(Annual252())

        expected_value = Decimal("2.52")  # Should be 252% annual return
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)

    def test_scale_zero(self):
        # Zero returns should remain zero when scaled
        daily = RateOfReturn(
            value=Decimal("0.0"),
            interval=Daily(),
        )

        actual = daily.scale(Annual252())

        assert actual.value == Decimal("0.0")
        assert isinstance(actual.interval, Annual252)

    def test_scale_negative(self):
        # Negative returns should scale linearly while maintaining sign
        daily = RateOfReturn(
            value=Decimal("-0.01"),  # -1% daily return
            interval=Daily(),
        )

        actual = daily.scale(Annual252())

        expected_value = Decimal("-2.52")  # Should be -252% annual return
        assert actual.value == expected_value
        assert isinstance(actual.interval, Annual252)


class TestDrawdown:
    def test_creation(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        end = datetime(2025, 1, 2, tzinfo=timezone.utc)
        drawdown = Drawdown(
            value=Decimal("-0.1"),
            time_start=start,
            time_end=end,
        )
        assert drawdown.value == Decimal("-0.1")
        assert drawdown.time_start == start
        assert drawdown.time_end == end
        assert drawdown.duration == timedelta(days=1)

    def test_equality(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        end = datetime(2025, 1, 2, tzinfo=timezone.utc)
        d1 = Drawdown(Decimal("-0.1"), start, end)
        d2 = Drawdown(Decimal("-0.1"), start, end)
        d3 = Drawdown(Decimal("-0.2"), start, end)
        assert d1 == d2
        assert d1 != d3


class TestMaxDrawdown:
    def test_creation(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        end = datetime(2025, 1, 2, tzinfo=timezone.utc)
        drawdown = Drawdown(Decimal("-0.1"), start, end)
        max_dd = MaxDrawdown(drawdown)
        assert max_dd.drawdown == drawdown


class TestMeanDrawdown:
    def test_creation(self):
        mean_dd = MeanDrawdown(
            mean_drawdown=Decimal("-0.05"),
            mean_drawdown_ms=Decimal("86400000"),  # 1 day in ms
        )
        assert mean_dd.mean_drawdown == Decimal("-0.05")
        assert mean_dd.mean_drawdown_ms == Decimal("86400000")


class TestBuildDrawdownSeries:
    def test_empty_points(self):
        result = build_drawdown_series([])
        assert result == []

    def test_single_point(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [(start, Decimal("100"))]
        result = build_drawdown_series(points)
        assert result == []

    def test_no_drawdown(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, Decimal("100")),
            (start + timedelta(days=1), Decimal("110")),
            (start + timedelta(days=2), Decimal("120")),
        ]
        result = build_drawdown_series(points)
        assert result == []

    def test_single_drawdown(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, Decimal("100")),
            (start + timedelta(days=1), Decimal("110")),
            (start + timedelta(days=2), Decimal("90")),
            (start + timedelta(days=3), Decimal("120")),
        ]
        result = build_drawdown_series(points)
        assert len(result) == 1
        assert result[0].value == Decimal(
            "0.1818181818181818181818181818"
        )  # (110-90)/110
        assert result[0].time_start == start + timedelta(days=1)
        assert result[0].time_end == start + timedelta(days=3)

    def test_multiple_drawdowns(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, Decimal("100")),
            (start + timedelta(days=1), Decimal("110")),
            (start + timedelta(days=2), Decimal("90")),
            (start + timedelta(days=3), Decimal("120")),
            (start + timedelta(days=4), Decimal("105")),
            (start + timedelta(days=5), Decimal("95")),
            (start + timedelta(days=6), Decimal("130")),
        ]
        result = build_drawdown_series(points)
        assert len(result) == 2
        # First drawdown: from 110 to 90, recovered at 120
        assert result[0].value == Decimal("0.1818181818181818181818181818")
        # Second drawdown: from 120 to 95, recovered at 130
        assert result[1].value == Decimal(
            "0.2083333333333333333333333333"
        )  # (120-95)/120


class TestGenerateDrawdownSeries:
    def test_with_floats(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 90.0),
            (start + timedelta(days=3), 120.0),
        ]
        result = generate_drawdown_series(points)
        assert len(result) == 1
        assert result[0].value == Decimal("0.1818181818181818181818181818")

    def test_filters_invalid_values(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), float("nan")),
            (start + timedelta(days=2), 110.0),
        ]
        result = generate_drawdown_series(points)
        assert result == []


class TestCalculateMaxDrawdown:
    def test_no_drawdowns(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 120.0),
        ]
        result = calculate_max_drawdown(points)
        assert result is None

    def test_single_drawdown(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 90.0),
            (start + timedelta(days=3), 120.0),
        ]
        result = calculate_max_drawdown(points)
        assert result is not None
        assert result.drawdown.value == Decimal("0.1818181818181818181818181818")

    def test_multiple_drawdowns(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 90.0),  # drawdown of ~0.1818
            (start + timedelta(days=3), 120.0),
            (start + timedelta(days=4), 105.0),
            (start + timedelta(days=5), 95.0),  # drawdown of ~0.2083
            (start + timedelta(days=6), 130.0),
        ]
        result = calculate_max_drawdown(points)
        assert result is not None
        # Should return the larger drawdown (second one)
        assert result.drawdown.value == Decimal("0.2083333333333333333333333333")


class TestCalculateMeanDrawdown:
    def test_no_drawdowns(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 120.0),
        ]
        result = calculate_mean_drawdown(points)
        assert result is None

    def test_single_drawdown(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 90.0),
            (start + timedelta(days=3), 120.0),
        ]
        result = calculate_mean_drawdown(points)
        assert result is not None
        assert result.mean_drawdown == Decimal("0.1818181818181818181818181818")
        assert result.mean_drawdown_ms == Decimal("172800000.0")  # 2 days in ms

    def test_multiple_drawdowns(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        points = [
            (start, 100.0),
            (start + timedelta(days=1), 110.0),
            (start + timedelta(days=2), 90.0),  # drawdown of ~0.1818, 1 day
            (start + timedelta(days=3), 120.0),
            (start + timedelta(days=4), 105.0),
            (start + timedelta(days=5), 95.0),  # drawdown of ~0.2083, 1 day
            (start + timedelta(days=6), 130.0),
        ]
        result = calculate_mean_drawdown(points)
        assert result is not None
        # Mean of 0.1818 and 0.2083
        expected_mean = (
            Decimal("0.1818181818181818181818181818")
            + Decimal("0.2083333333333333333333333333")
        ) / 2
        assert result.mean_drawdown == expected_mean
        assert result.mean_drawdown_ms == Decimal("216000000.0")  # 2.5 days in ms
