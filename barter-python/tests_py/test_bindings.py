from __future__ import annotations

import datetime as dt
from decimal import Decimal
from pathlib import Path

import pytest

import barter_python as bp


def test_shutdown_event_is_terminal() -> None:
    event = bp.shutdown_event()
    assert event.is_terminal()


def test_init_tracing_returns_bool() -> None:
    result = bp.init_tracing(filter="barter_python=info")
    assert isinstance(result, bool)


def test_init_tracing_invalid_filter_raises() -> None:
    with pytest.raises(ValueError):
        bp.init_tracing(filter="invalid[filter")


def test_version_matches_package_metadata() -> None:
    from importlib import metadata

    assert bp.__version__ == metadata.version("barter-python")


def test_engine_event_roundtrip() -> None:
    event = bp.EngineEvent.trading_state(True)
    event_dict = event.to_dict()
    restored = bp.EngineEvent.from_dict(event_dict)
    assert not restored.is_terminal()

    event_json = restored.to_json()
    replayed = bp.EngineEvent.from_json(event_json)
    assert not replayed.is_terminal()


def test_engine_event_balance_snapshot_builder() -> None:
    timestamp = dt.datetime(2024, 1, 2, tzinfo=dt.timezone.utc)

    event = bp.EngineEvent.account_balance_snapshot(
        exchange=3,
        asset=7,
        total=125.5,
        free=100.0,
        time_exchange=timestamp,
    )

    assert not event.is_terminal()

    payload = event.to_dict()
    account = payload["Account"]
    assert "Item" in account

    item = account["Item"]
    assert item["exchange"] == 3

    balance_snapshot = item["kind"]["BalanceSnapshot"]
    assert balance_snapshot["asset"] == 7
    assert balance_snapshot["balance"]["total"] == "125.5"
    assert balance_snapshot["balance"]["free"] == "100"
    assert balance_snapshot["time_exchange"] == timestamp.isoformat().replace("+00:00", "Z")

    round_trip = bp.EngineEvent.from_json(event.to_json())
    assert not round_trip.is_terminal()


def test_timed_f64_roundtrip() -> None:
    timestamp = dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc)
    timed = bp.timed_f64(42.5, timestamp)

    assert timed.value == pytest.approx(42.5)
    # PyO3 maps chrono::DateTime<Utc> to timezone-aware datetime.
    assert timed.time == timestamp


def test_system_config_dict_roundtrip(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    config_dict = config.to_dict()
    restored = bp.SystemConfig.from_dict(config_dict)

    assert restored.to_dict() == config_dict


def test_system_config_from_json_str(example_paths: dict[str, Path]) -> None:
    contents = example_paths["system_config"].read_text()
    config = bp.SystemConfig.from_json_str(contents)

    assert config.to_dict()["instruments"], "Config should load instruments from string"


def test_system_config_to_json_file(
    tmp_path: Path, example_paths: dict[str, Path]
) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    output_path = tmp_path / "system_config_copy.json"

    config.to_json_file(str(output_path))
    restored = bp.SystemConfig.from_json(str(output_path))

    assert restored.to_dict() == config.to_dict()


def test_run_historic_backtest_summary(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    summary = bp.run_historic_backtest(
        config, str(example_paths["market_data"])
    )

    assert isinstance(summary, bp.TradingSummary)
    assert summary.time_engine_start <= summary.time_engine_end

    instruments = summary.instruments
    assert instruments, "Summary should include instrument breakdown"

    instrument_name, instrument_summary = next(iter(instruments.items()))
    assert isinstance(instrument_name, str)
    assert isinstance(instrument_summary, bp.InstrumentTearSheet)

    assert instrument_summary.pnl == Decimal("0")
    assert instrument_summary.pnl_return.value == Decimal("0")
    assert instrument_summary.pnl_return.interval == "Daily"

    assert instrument_summary.sharpe_ratio.interval == "Daily"
    assert instrument_summary.sortino_ratio.interval == "Daily"
    assert instrument_summary.calmar_ratio.interval == "Daily"

    assets = summary.assets
    assert assets
    asset_name, asset_summary = next(iter(assets.items()))
    assert isinstance(asset_name, str)
    assert isinstance(asset_summary, bp.AssetTearSheet)


def test_system_handle_lifecycle(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    try:
        assert handle.is_running()

        handle.set_trading_enabled(True)
        handle.set_trading_enabled(False)

        filter_none = bp.InstrumentFilter.none()
        handle.close_positions(filter_none)
        handle.cancel_orders(bp.InstrumentFilter.none())
    finally:
        handle.shutdown()

    assert not handle.is_running()


def test_system_handle_feed_events(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    try:
        assert handle.is_running()

        events = [
            bp.EngineEvent.trading_state(True),
            bp.EngineEvent.trading_state(False),
            bp.EngineEvent.cancel_orders(bp.InstrumentFilter.none()),
        ]
        handle.feed_events(events)
    finally:
        handle.shutdown()

    assert not handle.is_running()


def test_shutdown_with_summary(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config)

    summary = handle.shutdown_with_summary()

    assert isinstance(summary, bp.TradingSummary)
    assert summary.time_engine_start <= summary.time_engine_end

    instruments = summary.instruments
    assert instruments

    tear_sheet = next(iter(instruments.values()))
    assert isinstance(tear_sheet, bp.InstrumentTearSheet)
    assert tear_sheet.pnl == Decimal("0")
    assert tear_sheet.win_rate is None
    assert tear_sheet.profit_factor is None

    summary_dict = summary.to_dict()
    assert summary_dict["instruments"]


def test_order_request_helpers() -> None:
    key = bp.OrderKey(0, 0, "strategy-alpha", "cid-123")
    open_request = bp.OrderRequestOpen(
        key,
        "buy",
        101.25,
        0.5,
        kind="limit",
        time_in_force="good_until_cancelled",
        post_only=True,
    )

    assert open_request.side == "buy"
    assert open_request.kind == "limit"
    assert open_request.time_in_force == "good_until_cancelled"

    cancel_request = bp.OrderRequestCancel(key, "order-1")
    assert cancel_request.has_order_id

    open_event = bp.EngineEvent.send_open_requests([open_request])
    cancel_event = bp.EngineEvent.send_cancel_requests([cancel_request])

    assert not open_event.is_terminal()
    assert not cancel_event.is_terminal()
