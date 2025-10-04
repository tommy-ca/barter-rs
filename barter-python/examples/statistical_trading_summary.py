"""Example demonstrating statistical trading summary generation using synthetic market data."""

from __future__ import annotations

import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

import barter_python as bp


def create_synthetic_market_data() -> list[dict]:
    """Create synthetic market data with trades and price movements."""
    base_time = datetime(2024, 1, 1, tzinfo=timezone.utc)
    events = []

    # Initial market data
    events.append({
        "Item": {
            "Ok": {
                "time_exchange": base_time.isoformat().replace('+00:00', 'Z'),
                "time_received": base_time.isoformat().replace('+00:00', 'Z'),
                "exchange": "binance_spot",
                "instrument": 0,  # BTCUSDT
                "kind": {
                    "Trade": {
                        "id": "trade-1",
                        "price": 50000.0,
                        "amount": 0.1,
                        "side": "Buy"
                    }
                }
            }
        }
    })

    # Add some price movements
    for i in range(1, 10):
        time = base_time + timedelta(hours=i)
        price = 50000.0 + (i * 100.0)  # Gradual price increase
        events.append({
            "Item": {
                "Ok": {
                    "time_exchange": time.isoformat().replace('+00:00', 'Z'),
                    "time_received": time.isoformat().replace('+00:00', 'Z'),
                    "exchange": "binance_spot",
                    "instrument": 0,
                    "kind": {
                        "Trade": {
                            "id": f"trade-{i+1}",
                            "price": price,
                            "amount": 0.05,
                            "side": "Buy" if i % 2 == 0 else "Sell"
                        }
                    }
                }
            }
        })

    return events


def create_system_config() -> dict:
    """Create a basic system configuration for backtesting."""
    return {
        "risk_free_return": 0.05,
        "instruments": [
            {
                "exchange": "binance_spot",
                "name_exchange": "BTCUSDT",
                "underlying": {
                    "base": "btc",
                    "quote": "usdt"
                },
                "quote": "underlying_quote",
                "kind": "spot"
            }
        ],
        "executions": [
            {
                "mocked_exchange": "binance_spot",
                "latency_ms": 100,
                "fees_percent": 0.05,
                "initial_state": {
                    "exchange": "binance_spot",
                    "balances": [
                        {
                            "asset": "usdt",
                            "balance": {
                                "total": 10000.0,
                                "free": 10000.0
                            },
                            "time_exchange": "2024-01-01T00:00:00Z"
                        },
                        {
                            "asset": "btc",
                            "balance": {
                                "total": 0.0,
                                "free": 0.0
                            },
                            "time_exchange": "2024-01-01T00:00:00Z"
                        }
                    ],
                    "instruments": [
                        {
                            "instrument": "BTCUSDT",
                            "orders": []
                        }
                    ]
                }
            }
        ]
    }


def main():
    """Run statistical trading summary example."""
    print("Barter Python - Statistical Trading Summary Example")
    print("=" * 50)

    # Create synthetic market data
    market_data = create_synthetic_market_data()
    market_data_path = Path("synthetic_market_data.json")
    with open(market_data_path, 'w') as f:
        json.dump(market_data, f, indent=2)

    # Create system config
    config = create_system_config()
    config_path = Path("example_config.json")
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)

    try:
        # Load config and run backtest
        system_config = bp.SystemConfig.from_json(str(config_path))
        summary = bp.run_historic_backtest(
            system_config,
            str(market_data_path),
            risk_free_return=0.05,
            interval="annual-365"
        )

        # Print summary
        print(f"Trading Summary (Risk-free return: 5%, Interval: Annual-365)")
        print(f"Start: {summary.time_engine_start}")
        print(f"End: {summary.time_engine_end}")
        print()

        # Print instrument summaries
        instruments = summary.instruments
        if instruments:
            print("Instrument Performance:")
            for name, tear_sheet in instruments.items():
                print(f"  {name}:")
                print(f"    PnL: {tear_sheet.pnl}")
                print(f"    Sharpe Ratio: {tear_sheet.sharpe_ratio.value} ({tear_sheet.sharpe_ratio.interval})")
                print(f"    Sortino Ratio: {tear_sheet.sortino_ratio.value} ({tear_sheet.sortino_ratio.interval})")
                print(f"    Calmar Ratio: {tear_sheet.calmar_ratio.value} ({tear_sheet.calmar_ratio.interval})")
                if tear_sheet.win_rate is not None:
                    print(f"    Win Rate: {tear_sheet.win_rate}")
                if tear_sheet.profit_factor is not None:
                    print(f"    Profit Factor: {tear_sheet.profit_factor}")
                print()

        # Print asset summaries
        assets = summary.assets
        if assets:
            print("Asset Performance:")
            for key, tear_sheet in assets.items():
                print(f"  {key}:")
                if tear_sheet.balance_end is not None:
                    balance = tear_sheet.balance_end
                    print(f"    End Balance - Total: {balance.total}, Free: {balance.free}")
                print()

        # Save detailed summary
        summary_dict = summary.to_dict()
        output_path = Path("trading_summary.json")
        with open(output_path, 'w') as f:
            json.dump(summary_dict, f, indent=2, default=str)

        print(f"Detailed summary saved to: {output_path}")

    finally:
        # Clean up temporary files
        market_data_path.unlink(missing_ok=True)
        config_path.unlink(missing_ok=True)


if __name__ == "__main__":
    main()