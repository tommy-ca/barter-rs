"""Risk management integration example with position limits and dynamic thresholds."""

from __future__ import annotations

from decimal import Decimal

import barter_python as bp


def main():
    """Run the risk management example."""
    print("Barter Python - Risk Management Example")
    print("=" * 50)

    # Load system configuration
    config_path = "../barter/examples/config/system_config.json"
    config = bp.SystemConfig.from_json(config_path)
    print("Loaded system configuration")

    # Inspect existing risk limits
    global_limits = config.risk_limits()["global"]
    print(f"Existing global limits: {global_limits}")

    # Set global risk limits
    config.set_global_risk_limits({
        "max_leverage": Decimal("1.5"),
        "max_position_notional": Decimal("10000"),
    })
    print("Set global risk limits: max_leverage=1.5, max_position_notional=10000")

    # Set per-instrument risk limits
    config.set_instrument_risk_limits(
        0,  # Assuming instrument 0
        {
            "max_exposure_percent": Decimal("0.3"),
            "max_position_quantity": Decimal("0.2"),
        },
    )
    print("Set per-instrument limits for instrument 0")

    # Fetch and display the limits
    instrument_limits = config.get_instrument_risk_limits(0)
    print(f"Instrument 0 limits: {instrument_limits}")

    # Persist the updated configuration
    config.to_json_file("config_with_risk.json")
    print("Saved updated config to config_with_risk.json")

    # Run a quick backtest to see risk in action
    market_data_path = "../barter/examples/data/binance_spot_market_data_with_disconnect_events.json"
    summary = bp.run_historic_backtest(
        config=config,
        market_data_path=market_data_path,
        risk_free_return=Decimal("0.02"),
        interval="annual_365",
    )

    print("\nBacktest completed with risk management:")
    if summary.instruments:
        for name, tear_sheet in summary.instruments.items():
            print(f"  {name}: PnL={tear_sheet.pnl}")

    print("Risk management integration example complete")


if __name__ == "__main__":
    main()
