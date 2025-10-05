"""Comprehensive backtest example with risk management and performance analysis."""

from __future__ import annotations

from decimal import Decimal

import barter_python as bp


def main():
    """Run the comprehensive backtest example."""
    print("Barter Python - Comprehensive Backtest Example")
    print("=" * 50)

    # Load system configuration
    config_path = "../barter/examples/config/system_config.json"
    config = bp.SystemConfig.from_json(config_path)
    print("Loaded system configuration")

    # Configure risk management
    config.set_global_risk_limits({
        "max_leverage": Decimal("1.0"),
        "max_position_notional": Decimal("1000"),
    })

    # Set per-instrument risk limits (assuming instrument 0 is BTCUSDT)
    config.set_instrument_risk_limits(
        0,
        {
            "max_exposure_percent": Decimal("0.5"),
            "max_position_quantity": Decimal("0.1"),
        },
    )

    # Load market data
    market_data_path = "../barter/examples/data/binance_spot_market_data_with_disconnect_events.json"
    print(f"Loading market data from {market_data_path}")

    # Run backtests with different intervals for comparison
    intervals = ["daily", "annual_252", "annual_365"]
    summaries = {}

    for interval in intervals:
        print(f"\nRunning backtest with interval: {interval}")
        summary = bp.run_historic_backtest(
            config=config,
            market_data_path=market_data_path,
            risk_free_return=Decimal("0.02"),
            interval=interval,
        )
        summaries[interval] = summary

    # Analyze and compare results
    print("\n" + "=" * 50)
    print("Backtest Comparison Results:")
    print("=" * 50)

    for interval, summary in summaries.items():
        print(f"\nInterval: {interval}")
        print(f"Start: {summary.time_engine_start}")
        print(f"End: {summary.time_engine_end}")
        print(f"Duration: {(summary.time_engine_end - summary.time_engine_start).days} days")

        if summary.instruments:
            for name, tear_sheet in summary.instruments.items():
                print(f"  {name} - PnL: {tear_sheet.pnl}, Sharpe: {tear_sheet.sharpe_ratio.value}")

    # Save detailed results
    import json
    for interval, summary in summaries.items():
        summary_dict = summary.to_dict()
        with open(f"backtest_results_{interval}.json", "w") as f:
            json.dump(summary_dict, f, indent=2, default=str)
    print("\nDetailed results saved to backtest_results_*.json")


if __name__ == "__main__":
    main()