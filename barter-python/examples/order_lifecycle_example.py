"""Order lifecycle example covering submission, fills, cancellations, and error handling."""

from __future__ import annotations

from decimal import Decimal
from datetime import datetime, timezone

import barter_python as bp


def main():
    """Run the order lifecycle example."""
    print("Barter Python - Order Lifecycle Example")
    print("=" * 50)

    # Create an order key
    order_key = bp.OrderKey(1, 2, "strategy-alpha", "cid-123")
    print(f"Created order key: {order_key}")

    # Create an open order request
    open_request = bp.OrderRequestOpen(
        order_key,
        "buy",
        price=Decimal("105.25"),
        quantity=Decimal("0.75"),
        kind="limit",
        time_in_force="good_until_cancelled",
        post_only=True,
    )
    print(f"Created open request: {open_request.side} {open_request.quantity} @ {open_request.price}")

    # Create an order snapshot (simulating fill)
    time_exchange = datetime(2025, 9, 10, 11, 12, 13, tzinfo=timezone.utc)
    snapshot = bp.OrderSnapshot.from_open_request(
        open_request,
        order_id="order-789",
        time_exchange=time_exchange,
        filled_quantity=Decimal("0.5"),
    )
    print("Created order snapshot")

    # Create account order snapshot event
    order_event = bp.EngineEvent.account_order_snapshot(exchange=1, snapshot=snapshot)
    print("Created order event")

    # Create a cancel request
    cancel_request = bp.OrderRequestCancel(order_key, "order-789")
    print("Created cancel request")

    # Create cancellation event
    cancel_time = datetime(2025, 9, 10, 11, 13, 0, tzinfo=timezone.utc)
    cancel_event = bp.EngineEvent.account_order_cancelled(
        exchange=1,
        request=cancel_request,
        order_id="order-789",
        time_exchange=cancel_time,
    )
    print("Created cancel event")

    # Demonstrate with mock execution client
    mock_config = bp.MockExecutionConfig()
    instrument_map = bp.ExecutionInstrumentMap.from_definitions(
        bp.ExchangeId.MOCK,
        [
            {
                "exchange": "mock",
                "name_exchange": "BTCUSDT",
                "underlying": {"base": "btc", "quote": "usdt"},
                "quote": "underlying_quote",
                "kind": "spot",
            }
        ],
    )

    with bp.MockExecutionClient(mock_config, instrument_map) as client:
        # Open an order
        order = client.open_limit_order(
            "BTCUSDT",
            "sell",
            Decimal("45000"),
            Decimal("0.05"),
            time_in_force="good_until_cancelled",
            post_only=True,
        )
        print(f"Opened order via mock client: {order['kind']} {order['side']}")

        # Note: Mock client may not support cancel, demonstrating open only

    print("Order lifecycle example complete")


if __name__ == "__main__":
    main()