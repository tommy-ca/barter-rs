"""Live system simulation example with mock execution, order management, and audit streaming."""

from __future__ import annotations

from decimal import Decimal
from time import sleep
from datetime import datetime, timezone
import json

import barter_python as bp


def main():
    """Run the live system simulation example."""
    print("Barter Python - Live System Simulation Example")
    print("=" * 50)

    # Load system configuration
    config_path = "../barter/examples/config/system_config.json"
    config = bp.SystemConfig.from_json(config_path)
    print("Loaded system configuration")

    # Configure mock execution
    mock_execution = bp.MockExecutionConfig(
        mocked_exchange=bp.ExchangeId.BINANCE_SPOT,
        latency_ms=50,
        fees_percent=0.1,
    )
    execution_config = bp.ExecutionConfig.mock(mock_execution)
    config.clear_executions()
    config.add_execution(execution_config)
    print("Configured mock execution")

    # Configure risk management
    config.set_global_risk_limits({
        "max_leverage": Decimal("2.0"),
        "max_position_notional": Decimal("5000"),
    })

    # Start the system with audit enabled
    handle = bp.start_system(config, trading_enabled=False, audit=True)
    print("Started system with audit streaming")

    # Take audit handle
    audit = handle.take_audit()
    print("Audit streaming enabled")

    # Enable trading
    handle.set_trading_enabled(True)
    print("Trading enabled")

    # Send some engine events (simulate market data or commands)
    # For simulation, send a balance snapshot
    balance_event = bp.EngineEvent.account_balance_snapshot(
        exchange=0,
        asset=0,
        total=Decimal("10000"),
        free=Decimal("9000"),
        time_exchange=datetime.now(timezone.utc),
    )
    handle.send_event(balance_event)
    print("Sent balance snapshot event")

    # Wait a bit
    sleep(1)

    # Check audit updates
    try:
        update = audit.updates.recv(timeout=1.0)
        print(f"Received audit update: {update.event.kind}")
    except:
        print("No audit update received")

    # Send trading state change
    trading_event = bp.EngineEvent.trading_state(True)
    handle.send_event(trading_event)
    print("Sent trading state enabled event")

    # Wait
    sleep(2)

    # Shutdown and get summary
    summary = handle.shutdown_with_summary(interval="annual_365")
    print("System shutdown complete")

    # Print summary
    print("\nShutdown Summary:")
    print(f"Start: {summary.time_engine_start}")
    print(f"End: {summary.time_engine_end}")

    if summary.instruments:
        for name, tear_sheet in summary.instruments.items():
            print(f"  {name}: PnL={tear_sheet.pnl}")

    # Save summary
    summary_dict = summary.to_dict()
    with open("live_simulation_summary.json", "w") as f:
        json.dump(summary_dict, f, indent=2, default=str)
    print("Summary saved to live_simulation_summary.json")


if __name__ == "__main__":
    main()