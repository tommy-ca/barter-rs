from __future__ import annotations

from decimal import Decimal
from pathlib import Path

import pytest

import barter_python as bp


@pytest.mark.integration
def test_live_system_lifecycle(example_paths: dict[str, Path]) -> None:
    """Exercise core lifecycle controls exposed through the Python bindings."""

    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    summary: bp.TradingSummary | None = None

    try:
        assert handle.is_running()

        handle.set_trading_enabled(True)
        handle.set_trading_enabled(False)

        filter_none = bp.InstrumentFilter.none()
        handle.close_positions(filter_none)
        handle.cancel_orders(filter_none)

        key = bp.OrderKey(0, 0, "integration-lifecycle", "cid-live-0")
        open_request = bp.OrderRequestOpen(
            key,
            "buy",
            101.25,
            0.5,
            kind="limit",
            time_in_force="good_until_cancelled",
            post_only=True,
        )
        cancel_request = bp.OrderRequestCancel(key)

        handle.send_open_requests([open_request])
        handle.send_cancel_requests([cancel_request])

        events = [
            bp.EngineEvent.trading_state(True),
            bp.EngineEvent.trading_state(False),
            bp.EngineEvent.cancel_orders(bp.InstrumentFilter.none()),
        ]
        handle.feed_events(events)
    finally:
        summary = handle.shutdown_with_summary(interval="annual_365")

    assert summary is not None
    assert not handle.is_running()

    assert summary.time_engine_start <= summary.time_engine_end

    instruments = summary.instruments
    assert instruments

    name, tear_sheet = next(iter(instruments.items()))
    assert isinstance(name, str)
    assert tear_sheet.pnl == Decimal("0")
    assert tear_sheet.pnl_return.value == Decimal("0")
    assert tear_sheet.pnl_return.interval == "Annual(365)"
    assert tear_sheet.sharpe_ratio.interval == "Annual(365)"
    assert tear_sheet.sortino_ratio.interval == "Annual(365)"
    assert tear_sheet.calmar_ratio.interval == "Annual(365)"

    summary_dict = summary.to_dict()
    assert name in summary_dict["instruments"]


@pytest.mark.integration
def test_take_audit_disabled_returns_none(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    try:
        assert handle.take_audit() is None
    finally:
        handle.shutdown()


@pytest.mark.integration
def test_take_audit_streaming(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False, audit=True)

    try:
        snap_updates = handle.take_audit()
        assert snap_updates is not None

        snapshot = snap_updates.snapshot.value
        assert "state_summary" in snapshot
        assert "context" in snapshot
        sequence = snapshot["context"]["sequence"]
        assert isinstance(sequence, bp.Sequence)
        assert int(sequence) >= 0
        assert "asset_count" in snapshot["state_summary"]

        updates = snap_updates.updates
        assert updates.try_recv() is None

        handle.send_event(bp.EngineEvent.trading_state(True))
        next_tick = updates.recv(timeout=1.0)
        assert next_tick is not None
        assert next_tick["event"]["kind"] in {"Process", "FeedEnded"}
        next_sequence = next_tick["context"]["sequence"]
        assert isinstance(next_sequence, bp.Sequence)
        assert int(next_sequence) >= int(sequence)

        # Audit channel is single-use; subsequent calls return None
        assert handle.take_audit() is None
    finally:
        handle.shutdown()
