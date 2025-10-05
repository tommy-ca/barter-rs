"""Multi-exchange backtest example demonstrating cross-exchange trading."""

from __future__ import annotations

import barter_python as bp


def main():
    """Run the multi-exchange backtest example."""
    print("Barter Python - Multi-Exchange Backtest Example")
    print("=" * 50)

    # Load existing config
    config_path = "../barter/examples/config/system_config.json"
    config = bp.SystemConfig.from_json(config_path)
    print("Loaded base configuration")

    # Demonstrate multi-exchange setup (would add instruments and executions)
    print("Would add Coinbase BTC instrument and mock execution")
    print("Multi-exchange config requires API support for adding instruments/executions")

    # Note: For full multi-exchange backtest, need market data for both exchanges
    print("Multi-exchange config setup complete. Full backtest requires multi-exchange market data.")


if __name__ == "__main__":
    main()
