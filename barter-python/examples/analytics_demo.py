"""Comprehensive analytics demonstration using barter-python.

This example demonstrates all available financial metrics and analytics functions
with realistic trading performance data patterns.
"""

from __future__ import annotations

import datetime as dt
from decimal import Decimal
from math import sqrt

import barter_python as bp


def create_sample_equity_curve() -> list[tuple[dt.datetime, Decimal]]:
    """Create a realistic equity curve with various market conditions."""
    base_time = dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc)
    points = []

    # Starting capital
    equity = Decimal("10000")

    # Bull market phase (steady growth)
    for i in range(30):
        time = base_time + dt.timedelta(days=i)
        # Add some volatility but overall upward trend
        daily_return = Decimal(str(0.002 + 0.005 * (i / 30)))  # 0.2% to 0.7%
        # Add random noise
        noise = Decimal(str(0.001 * (i % 3 - 1)))  # -0.1% to +0.1%
        equity *= (Decimal("1") + daily_return + noise)
        points.append((time, equity))

    # Volatile sideways market
    for i in range(30, 60):
        time = base_time + dt.timedelta(days=i)
        # Oscillating returns around 0%
        oscillation = Decimal(str(0.01 * (i % 4 - 2) / 100))  # -0.02% to +0.02%
        equity *= (Decimal("1") + oscillation)
        points.append((time, equity))

    # Bear market phase (drawdown)
    for i in range(60, 90):
        time = base_time + dt.timedelta(days=i)
        # Negative trend with increasing volatility
        trend = Decimal(str(-0.005 - 0.002 * ((i - 60) / 30)))
        volatility = Decimal(str(0.02 * ((i - 60) / 30)))
        # Simulate random walk with downward bias
        random_component = Decimal(str(volatility * (i % 3 - 1)))
        equity *= (Decimal("1") + trend + random_component)
        points.append((time, equity))

    # Recovery phase
    for i in range(90, 120):
        time = base_time + dt.timedelta(days=i)
        # Strong recovery with high volatility
        recovery = Decimal(str(0.015 - 0.01 * ((i - 90) / 30)))
        equity *= (Decimal("1") + recovery)
        points.append((time, equity))

    return points


def demonstrate_risk_adjusted_metrics():
    """Demonstrate Sharpe, Sortino, and Calmar ratios."""
    print("=" * 60)
    print("RISK-ADJUSTED RETURN METRICS")
    print("=" * 60)

    # Sample portfolio statistics
    risk_free_rate = 0.03  # 3% annual risk-free rate
    mean_return = 0.12     # 12% annual return
    total_volatility = 0.18  # 18% total volatility
    downside_volatility = 0.12  # 12% downside volatility
    max_drawdown = 0.22   # 22% maximum drawdown

    print(".1f")
    print(".1f")
    print(".1f")
    print(".1f")
    print()

    # Sharpe Ratio
    print("Sharpe Ratio (Annual 252):")
    sharpe = bp.calculate_sharpe_ratio(
        risk_free_return=risk_free_rate,
        mean_return=mean_return,
        std_dev_returns=total_volatility,
        interval="annual_252"
    )
    print(".4f")
    print()

    # Sortino Ratio
    print("Sortino Ratio (Annual 252):")
    sortino = bp.calculate_sortino_ratio(
        risk_free_return=risk_free_rate,
        mean_return=mean_return,
        std_dev_loss_returns=downside_volatility,
        interval="annual_252"
    )
    print(".4f")
    print()

    # Calmar Ratio
    print("Calmar Ratio (Annual 252):")
    calmar = bp.calculate_calmar_ratio(
        risk_free_return=risk_free_rate,
        mean_return=mean_return,
        max_drawdown=max_drawdown,
        interval="annual_252"
    )
    print(".4f")
    print()

    # Demonstrate scaling
    print("Scaling Example - Sharpe Ratio from Daily to Annual:")
    daily_sharpe = bp.calculate_sharpe_ratio(
        risk_free_return=risk_free_rate / 252,  # Daily risk-free rate
        mean_return=mean_return / 252,          # Daily return
        std_dev_returns=total_volatility / sqrt(252),  # Daily volatility
        interval="daily"
    )
    print(".4f")

    scaled_sharpe = daily_sharpe  # This would be scaled if we had scaling method
    print(".4f")
    print()


def demonstrate_performance_metrics():
    """Demonstrate profit factor, win rate, and rate of return."""
    print("=" * 60)
    print("PERFORMANCE METRICS")
    print("=" * 60)

    # Trading performance data
    gross_profits = 45000.0
    gross_losses = 25000.0
    winning_trades = 68
    total_trades = 120

    print("Trading Performance Summary:")
    print(".0f")
    print(".0f")
    print(f"Winning Trades: {winning_trades}")
    print(f"Total Trades: {total_trades}")
    print()

    # Profit Factor
    print("Profit Factor:")
    profit_factor = bp.calculate_profit_factor(gross_profits, gross_losses)
    if profit_factor is not None:
        print(".2f")
        if profit_factor > 1:
            print("  → Profitable strategy (Profit Factor > 1)")
        else:
            print("  → Unprofitable strategy (Profit Factor < 1)")
    else:
        print("  → No trading activity")
    print()

    # Win Rate
    print("Win Rate:")
    win_rate = bp.calculate_win_rate(winning_trades, total_trades)
    if win_rate is not None:
        print(".1%")
    else:
        print("  → No trades executed")
    print()

    # Rate of Return
    print("Rate of Return Scaling:")
    daily_return = bp.calculate_rate_of_return(0.005, "daily")  # 0.5% daily
    print(".3f")

    annual_return = bp.calculate_rate_of_return(
        0.005, "daily", "annual_252"
    )
    print(".3f")
    print()


def demonstrate_drawdown_analysis():
    """Demonstrate drawdown analysis with equity curve."""
    print("=" * 60)
    print("DRAWDOWN ANALYSIS")
    print("=" * 60)

    # Create sample equity curve
    equity_curve = create_sample_equity_curve()

    print(f"Equity curve: {len(equity_curve)} data points")
    print(".2f")
    print(".2f")
    print()

    # Generate drawdown series
    drawdowns = bp.generate_drawdown_series(equity_curve)
    print(f"Drawdown periods identified: {len(drawdowns)}")

    if drawdowns:
        print("\nDrawdown Details:")
        for i, dd in enumerate(drawdowns[:5], 1):  # Show first 5
            duration = dd.duration()
            print(f"  {i}. Value: {dd.value:.1%}, "
                  f"Start: {dd.time_start.date()}, "
                  f"End: {dd.time_end.date()}, "
                  f"Duration: {duration.days} days")

        if len(drawdowns) > 5:
            print(f"  ... and {len(drawdowns) - 5} more periods")
    print()

    # Maximum drawdown
    max_dd = bp.calculate_max_drawdown(equity_curve)
    if max_dd:
        print("Maximum Drawdown:")
        print(".1%")
        duration = max_dd.duration()
        print(f"  Start: {max_dd.time_start.date()}")
        print(f"  End: {max_dd.time_end.date()}")
        print(f"  Duration: {duration.days} days")
    else:
        print("No drawdowns detected")
    print()

    # Mean drawdown
    mean_dd = bp.calculate_mean_drawdown(equity_curve)
    if mean_dd:
        print("Mean Drawdown Statistics:")
        print(".1%")
        print(f"  Mean Duration: {mean_dd.mean_duration}")
    else:
        print("No drawdowns detected")
    print()


def demonstrate_welford_algorithms():
    """Demonstrate Welford's online algorithms for streaming statistics."""
    print("=" * 60)
    print("WELFORD ONLINE ALGORITHMS")
    print("=" * 60)

    print("Streaming calculation of statistical moments:")
    print()

    # Simulate streaming data
    returns = [0.01, 0.005, -0.002, 0.008, -0.003, 0.012, -0.005, 0.006]

    # Initialize
    count = 1
    mean = returns[0]
    m = 0.0  # Recurrence relation M

    print("Step-by-step calculation:")
    print("Step | Value | Running Mean | M | Sample Variance")
    print("-" * 55)

    for i, value in enumerate(returns):
        if i == 0:
            print("2d")
            continue

        count += 1
        prev_mean = mean
        prev_m = m

        # Update mean
        mean = bp.welford_calculate_mean(prev_mean, value, count)

        # Update M
        m = bp.welford_calculate_recurrence_relation_m(
            prev_m, prev_mean, value, float(mean)
        )

        # Calculate variance
        variance = bp.welford_calculate_sample_variance(m, count)

        print("2d")

    print()
    print("Final Statistics:")
    print(".4f")
    print(".4f")
    print(".4f")
    print()


def demonstrate_time_intervals():
    """Demonstrate different time interval options."""
    print("=" * 60)
    print("TIME INTERVAL SUPPORT")
    print("=" * 60)

    # Common intervals
    intervals = ["daily", "annual_252", "annual_365"]

    print("Sharpe Ratio across different intervals:")
    print("(Risk-free: 2%, Return: 10%, Volatility: 15%)")
    print()

    for interval in intervals:
        ratio = bp.calculate_sharpe_ratio(
            risk_free_return=0.02,
            mean_return=0.10,
            std_dev_returns=0.15,
            interval=interval
        )
        print("12s")

    # Custom interval
    custom_interval = dt.timedelta(hours=4)
    ratio = bp.calculate_sharpe_ratio(
        risk_free_return=0.02,
        mean_return=0.10,
        std_dev_returns=0.15,
        interval=custom_interval
    )
    print("12s")
    print()


def main():
    """Run the comprehensive analytics demonstration."""
    print("Barter Python - Comprehensive Analytics Demonstration")
    print("=" * 60)
    print()

    demonstrate_risk_adjusted_metrics()
    demonstrate_performance_metrics()
    demonstrate_drawdown_analysis()
    demonstrate_welford_algorithms()
    demonstrate_time_intervals()

    print("=" * 60)
    print("Analytics demonstration completed!")
    print("=" * 60)


if __name__ == "__main__":
    main()