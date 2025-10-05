#!/usr/bin/env python3
"""
Performance benchmarks for Barter Python bindings.

This script measures the performance of key operations in the Python bindings
to identify bottlenecks and track improvements.
"""

import datetime as dt
import statistics
import time

import barter_python as bp


def benchmark_event_creation(n: int = 10000) -> float:
    """Benchmark EngineEvent creation throughput."""
    print(f"Benchmarking EngineEvent creation ({n} events)...")

    start_time = time.perf_counter()

    for i in range(n):
        # Create various types of events
        if i % 4 == 0:
            event = bp.EngineEvent.trading_state(i % 2 == 0)
        elif i % 4 == 1:
            event = bp.EngineEvent.account_balance_snapshot(
                exchange=i % 10,
                asset=i % 20,
                total=float(i + 100),
                free=float(i + 50),
                time_exchange=dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc),
            )
        elif i % 4 == 2:
            event = bp.EngineEvent.market_trade(
                "binance_spot",
                i % 5,
                float(50000 + i),
                float(0.1 + i * 0.01),
                "buy" if i % 2 == 0 else "sell",
                dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc),
            )
        else:
            event = bp.EngineEvent.account_order_snapshot(
                exchange=i % 10,
                snapshot=bp.OrderSnapshot.from_open_request(
                    bp.OrderRequestOpen(
                        bp.OrderKey(i % 10, i % 5, f"strategy-{i % 3}", f"cid-{i}"),
                        "buy",
                        float(50000 + i),
                        0.1,
                        kind="limit",
                        time_in_force="good_until_cancelled",
                        post_only=False,
                    ),
                    order_id=f"order-{i}",
                    time_exchange=dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc),
                    filled_quantity=0.0,
                ),
            )

    end_time = time.perf_counter()
    total_time = end_time - start_time
    throughput = n / total_time

    print(".2f")
    return throughput


def benchmark_analytics_calculations(n: int = 10000) -> float:
    """Benchmark analytics function calls."""
    print(f"Benchmarking analytics calculations ({n} calculations)...")

    # Prepare test data
    returns = [0.01 + i * 0.001 for i in range(100)]
    prices = [100.0 + i * 0.1 for i in range(100)]
    downside_returns = [r for r in returns if r < 0] or [0.001, 0.002]  # Ensure at least 2 points

    start_time = time.perf_counter()

    for i in range(n):
        # Mix of different analytics functions
        if i % 5 == 0:
            result = bp.calculate_sharpe_ratio(
                risk_free_return=0.02,
                mean_return=statistics.mean(returns),
                std_dev_returns=statistics.stdev(returns),
                interval="annual_252"
            )
        elif i % 5 == 1:
            result = bp.calculate_sortino_ratio(
                risk_free_return=0.02,
                mean_return=statistics.mean(returns),
                std_dev_loss_returns=statistics.stdev(downside_returns),
                interval="annual_252"
            )
        elif i % 5 == 2:
            result = bp.calculate_calmar_ratio(
                risk_free_return=0.02,
                mean_return=statistics.mean(returns),
                max_drawdown=0.1,
                interval="daily"
            )
        elif i % 5 == 3:
            equity_points = [(dt.datetime(2024, 1, 1) + dt.timedelta(days=j), prices[j]) for j in range(len(prices))]
            result = bp.calculate_max_drawdown(equity_points)
        else:
            result = bp.welford_calculate_mean(
                statistics.mean(returns),
                returns[i % len(returns)],
                len(returns) + 1
            )

    end_time = time.perf_counter()
    total_time = end_time - start_time
    throughput = n / total_time

    print(".2f")
    return throughput


def benchmark_order_book_operations(n: int = 5000) -> float:
    """Benchmark order book creation and analysis."""
    print(f"Benchmarking order book operations ({n} operations)...")

    start_time = time.perf_counter()

    for i in range(n):
        # Create order book with varying sizes
        bids = [(100.0 - j * 0.1, 1.0 + j * 0.1) for j in range(10)]
        asks = [(100.5 + j * 0.1, 1.0 + j * 0.1) for j in range(10)]

        book = bp.OrderBook(sequence=i, bids=bids, asks=asks)

        # Perform calculations
        mid_price = book.mid_price()
        vw_mid = book.volume_weighted_mid_price()

        # Access levels
        bid_levels = book.bids()
        ask_levels = book.asks()

    end_time = time.perf_counter()
    total_time = end_time - start_time
    throughput = n / total_time

    print(".2f")
    return throughput


def benchmark_json_serialization(n: int = 5000) -> float:
    """Benchmark JSON serialization/deserialization."""
    print(f"Benchmarking JSON serialization ({n} operations)...")

    # Create a complex event
    event = bp.EngineEvent.market_trade(
        "binance_spot",
        0,
        50000.0,
        0.1,
        "buy",
        dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc),
    )

    start_time = time.perf_counter()

    for _ in range(n):
        # Serialize to JSON
        json_str = event.to_json()

        # Deserialize from JSON
        restored = bp.EngineEvent.from_json(json_str)

    end_time = time.perf_counter()
    total_time = end_time - start_time
    throughput = n / total_time

    print(".2f")
    return throughput


def benchmark_system_config_operations(n: int = 1000) -> float:
    """Benchmark system configuration operations."""
    print(f"Benchmarking system config operations ({n} operations)...")

    # Load config once
    config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")

    start_time = time.perf_counter()

    for _ in range(n):
        # Various config operations
        executions = config.executions()
        json_str = config.to_json()
        dict_repr = config.to_dict()

        # Risk operations
        risk_limits = config.risk_limits()

    end_time = time.perf_counter()
    total_time = end_time - start_time
    throughput = n / total_time

    print(".2f")
    return throughput


def run_benchmarks() -> None:
    """Run all benchmarks and report results."""
    print("=" * 60)
    print("Barter Python Bindings Performance Benchmarks")
    print("=" * 60)
    print()

    benchmarks = [
        ("Event Creation", benchmark_event_creation),
        ("Analytics Calculations", benchmark_analytics_calculations),
        ("Order Book Operations", benchmark_order_book_operations),
        ("JSON Serialization", benchmark_json_serialization),
        ("System Config Operations", benchmark_system_config_operations),
    ]

    results = []

    for name, benchmark_func in benchmarks:
        try:
            throughput = benchmark_func()
            results.append((name, throughput))
            print()
        except Exception as e:
            print(f"ERROR in {name}: {e}")
            print()
            results.append((name, 0.0))

    print("=" * 60)
    print("Summary")
    print("=" * 60)

    for name, throughput in results:
        if throughput > 0:
            print("25s")
        else:
            print("25s")

    print()
    print("Note: Higher throughput values indicate better performance.")


if __name__ == "__main__":
    run_benchmarks()
