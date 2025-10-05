"""Tests for bridging execution OrderEvent bindings into Python."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from decimal import Decimal

import pytest

import barter_python as bp


@pytest.fixture(name="sample_key")
def fixture_sample_key() -> bp.OrderKey:
    exchange = bp.ExchangeIndex(1)
    instrument = bp.InstrumentIndex(3)
    strategy = bp.StrategyId("mean_reversion")
    cid = bp.ClientOrderId("cid-42")
    return bp.OrderKey.from_indices(exchange, instrument, strategy, client_order_id=cid)


def test_order_event_active_open_round_trip(sample_key: bp.OrderKey) -> None:
    payload = {
        "key": {
            "exchange": sample_key.exchange,
            "instrument": sample_key.instrument,
            "strategy": sample_key.strategy_id,
            "cid": sample_key.client_order_id,
        },
        "state": {
            "Active": {
                "Open": {
                    "id": "order-123",
                    "time_exchange": "2025-10-05T10:15:00+00:00",
                    "filled_quantity": "0.50",
                }
            }
        },
    }

    event = bp.OrderEvent.from_dict(payload)

    key = event.key
    assert isinstance(key, bp.OrderKey)
    assert key.exchange == sample_key.exchange
    assert key.instrument == sample_key.instrument
    assert key.strategy_id == sample_key.strategy_id
    assert key.client_order_id == sample_key.client_order_id

    state = event.state
    assert state.variant == "Active"
    assert state.is_active()
    active = state.active()
    assert active is not None
    assert active.variant == "Open"
    open_state = active.open()
    assert open_state is not None
    assert open_state.order_id.value == "order-123"
    assert open_state.filled_quantity == Decimal("0.50")
    assert open_state.time_exchange == datetime(2025, 10, 5, 10, 15, tzinfo=timezone.utc)

    round_trip = event.to_dict()
    expected = payload.copy()
    expected_state = expected["state"]["Active"]["Open"].copy()
    expected_state["time_exchange"] = "2025-10-05T10:15:00Z"
    expected["state"] = {"Active": {"Open": expected_state}}
    assert round_trip == expected

    json_payload = json.dumps(payload)
    from_json = bp.OrderEvent.from_json(json_payload)
    assert from_json.to_dict() == expected

    state_dict = state.to_dict()
    assert state_dict == expected["state"]


def test_order_event_inactive_cancelled_round_trip(sample_key: bp.OrderKey) -> None:
    timestamp = datetime(2025, 10, 5, 11, 0, tzinfo=timezone.utc).isoformat()
    payload = {
        "key": {
            "exchange": sample_key.exchange,
            "instrument": sample_key.instrument,
            "strategy": sample_key.strategy_id,
            "cid": sample_key.client_order_id,
        },
        "state": {
            "Inactive": {
                "Cancelled": {
                    "id": "order-123",
                    "time_exchange": timestamp,
                }
            }
        },
    }

    event = bp.OrderEvent.from_dict(payload)

    state = event.state
    assert state.variant == "Inactive"
    assert state.is_inactive()
    inactive = state.inactive()
    assert inactive is not None
    assert inactive.variant == "Cancelled"
    cancelled = inactive.cancelled()
    assert cancelled is not None
    assert cancelled.order_id.value == "order-123"
    assert cancelled.time_exchange == datetime(2025, 10, 5, 11, 0, tzinfo=timezone.utc)

    expected = payload.copy()
    expected["state"] = {
        "Inactive": {
            "Cancelled": {
                "id": "order-123",
                "time_exchange": timestamp.replace("+00:00", "Z"),
            }
        }
    }
    assert event.to_dict() == expected
    assert state.to_dict() == expected["state"]


def test_order_event_inactive_fully_filled(sample_key: bp.OrderKey) -> None:
    payload = {
        "key": {
            "exchange": sample_key.exchange,
            "instrument": sample_key.instrument,
            "strategy": sample_key.strategy_id,
            "cid": sample_key.client_order_id,
        },
        "state": {
            "Inactive": "FullyFilled",
        },
    }

    event = bp.OrderEvent.from_dict(payload)

    state = event.state
    assert state.variant == "Inactive"
    inactive = state.inactive()
    assert inactive is not None
    assert inactive.variant == "FullyFilled"
    assert inactive.is_fully_filled()
    assert not inactive.is_cancelled()
    assert not inactive.is_open_failed()

    assert event.to_dict() == payload
    assert state.to_dict() == payload["state"]


def test_order_event_inactive_rejected(sample_key: bp.OrderKey) -> None:
    payload = {
        "key": {
            "exchange": sample_key.exchange,
            "instrument": sample_key.instrument,
            "strategy": sample_key.strategy_id,
            "cid": sample_key.client_order_id,
        },
        "state": {
            "Inactive": {
                "OpenFailed": {
                    "Rejected": "RateLimit",
                }
            }
        },
    }

    event = bp.OrderEvent.from_dict(payload)

    state = event.state
    assert state.variant == "Inactive"
    inactive = state.inactive()
    assert inactive is not None
    assert inactive.variant == "OpenFailed"
    assert inactive.is_open_failed()
    error = inactive.open_failed()
    assert error is not None
    assert error.variant == "Rejected"
    assert error.is_rejected()
    assert not error.is_connectivity()
    assert error.to_dict() == {"Rejected": "RateLimit"}

    expected = payload.copy()
    assert event.to_dict() == expected
    assert state.to_dict() == expected["state"]
