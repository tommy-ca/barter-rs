from __future__ import annotations

from decimal import Decimal
from pathlib import Path

import barter_python as bp


def test_start_system_with_seeded_balances(example_paths: dict[str, Path]) -> None:
    """Seeding balances should surface in the resulting trading summary."""

    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))

    seeded_total = Decimal("4321.5")
    seeded_free = Decimal("2100.25")

    handle = bp.start_system(
        config,
        trading_enabled=False,
        initial_balances=[
            {
                "exchange": "binance_spot",
                "asset": "usdt",
                "total": float(seeded_total),
                "free": float(seeded_free),
            }
        ],
    )

    summary: bp.TradingSummary | None = None

    try:
        summary = handle.shutdown_with_summary()
    finally:
        assert not handle.is_running()

    assert summary is not None

    assets = summary.assets
    assert "binance_spot:usdt" in assets

    asset_sheet = assets["binance_spot:usdt"]
    balance_end = asset_sheet.balance_end

    assert balance_end is not None
    assert balance_end.total == seeded_total
    assert balance_end.free == seeded_free
    assert balance_end.used == seeded_total - seeded_free
