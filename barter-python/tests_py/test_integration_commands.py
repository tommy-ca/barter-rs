from __future__ import annotations

import pytest

import barter_python as bp


@pytest.mark.integration
def test_command_builders_round_trip(example_paths):
    """Build command payloads in Python and exercise handle helpers."""

    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    key = bp.OrderKey(0, 0, "integration-commands", "cid-cmd-0")
    open_request = bp.OrderRequestOpen(
        key,
        "sell",
        102.5,
        0.25,
        kind="limit",
        time_in_force="good_until_cancelled",
    )
    cancel_request = bp.OrderRequestCancel(key)

    filter_none = bp.InstrumentFilter.none()
    filter_exchanges = bp.InstrumentFilter.exchanges([0])
    filter_instruments = bp.InstrumentFilter.instruments([0])
    filter_underlyings = bp.InstrumentFilter.underlyings([(0, 1)])

    try:
        handle.send_open_requests([open_request])
        handle.send_cancel_requests([cancel_request])

        handle.close_positions(filter_none)
        handle.cancel_orders(filter_exchanges)
        handle.cancel_orders(filter_instruments)
        handle.cancel_orders(filter_underlyings)

        assert "OrderRequestOpen" in repr(open_request)
        assert "OrderRequestCancel" in repr(cancel_request)
        assert "InstrumentFilter" in repr(filter_exchanges)

        open_event = bp.EngineEvent.send_open_requests([open_request])
        cancel_event = bp.EngineEvent.send_cancel_requests([cancel_request])

        assert not open_event.is_terminal()
        assert not cancel_event.is_terminal()
    finally:
        handle.shutdown()

    assert not handle.is_running()
