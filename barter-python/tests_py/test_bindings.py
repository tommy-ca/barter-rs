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


def test_subscription_id() -> None:
    sid = bp.SubscriptionId("test-id")
    assert sid.value == "test-id"
    assert str(sid) == "test-id"
    assert repr(sid) == "SubscriptionId('test-id')"

    # Test equality
    sid2 = bp.SubscriptionId("test-id")
    sid3 = bp.SubscriptionId("other-id")
    assert sid == sid2
    assert sid != sid3


def test_init_tracing_invalid_filter_raises() -> None:
    with pytest.raises(ValueError):
        bp.init_tracing(filter="invalid[filter")


def test_init_json_logging_py() -> None:
    result = bp.init_json_logging_py()
    assert isinstance(result, bool)


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
    assert balance_snapshot["time_exchange"] == timestamp.isoformat().replace(
        "+00:00", "Z"
    )

    round_trip = bp.EngineEvent.from_json(event.to_json())
    assert not round_trip.is_terminal()


def test_engine_event_market_trade_builder() -> None:
    timestamp = dt.datetime(2024, 3, 4, 5, 6, 7, tzinfo=dt.timezone.utc)

    event = bp.EngineEvent.market_trade(
        "binance_spot",
        1,
        101.25,
        0.75,
        "buy",
        timestamp,
        trade_id="py-trade-1",
    )

    payload = event.to_dict()
    market = payload["Market"]["Item"]

    assert market["exchange"] == "binance_spot"
    assert market["instrument"] == 1
    assert market["time_exchange"] == timestamp.isoformat().replace("+00:00", "Z")
    assert market["time_received"] == timestamp.isoformat().replace("+00:00", "Z")

    trade = market["kind"]["Trade"]
    assert trade["id"] == "py-trade-1"
    assert trade["price"] == pytest.approx(101.25)
    assert trade["amount"] == pytest.approx(0.75)
    assert trade["side"].lower() == "buy"


def test_engine_event_market_order_book_l1_builder() -> None:
    timestamp = dt.datetime(2025, 1, 2, 3, 4, 5, tzinfo=dt.timezone.utc)

    event = bp.EngineEvent.market_order_book_l1(
        "binance_spot",
        7,
        last_update_time=timestamp,
        best_bid=(100.5, 2.0),
        best_ask=(101.0, 1.5),
    )

    payload = event.to_dict()
    market = payload["Market"]["Item"]
    book = market["kind"]["OrderBookL1"]

    assert book["last_update_time"] == timestamp.isoformat().replace("+00:00", "Z")

    best_bid = book["best_bid"]
    assert Decimal(best_bid["price"]) == Decimal("100.5")
    assert Decimal(best_bid["amount"]) == Decimal("2")

    best_ask = book["best_ask"]
    assert Decimal(best_ask["price"]) == Decimal("101.0")
    assert Decimal(best_ask["amount"]) == Decimal("1.5")


def test_engine_event_market_candle_builder() -> None:
    time_exchange = dt.datetime(2025, 2, 3, 4, 5, 6, tzinfo=dt.timezone.utc)
    close_time = time_exchange + dt.timedelta(minutes=1)

    event = bp.EngineEvent.market_candle(
        "kraken",
        4,
        time_exchange=time_exchange,
        close_time=close_time,
        open=100.0,
        high=110.0,
        low=95.0,
        close=105.0,
        volume=250.5,
        trade_count=42,
    )

    market = event.to_dict()["Market"]["Item"]
    candle = market["kind"]["Candle"]

    assert market["time_exchange"] == time_exchange.isoformat().replace("+00:00", "Z")
    assert candle["close_time"] == close_time.isoformat().replace("+00:00", "Z")
    assert candle["open"] == pytest.approx(100.0)
    assert candle["high"] == pytest.approx(110.0)
    assert candle["low"] == pytest.approx(95.0)
    assert candle["close"] == pytest.approx(105.0)
    assert candle["volume"] == pytest.approx(250.5)
    assert candle["trade_count"] == 42


def test_engine_event_market_liquidation_builder() -> None:
    timestamp = dt.datetime(2025, 3, 4, 5, 6, 7, tzinfo=dt.timezone.utc)

    event = bp.EngineEvent.market_liquidation(
        "mock",
        2,
        price=20550.25,
        quantity=0.35,
        side="sell",
        time_exchange=timestamp,
    )

    market = event.to_dict()["Market"]["Item"]
    liquidation = market["kind"]["Liquidation"]

    assert liquidation["price"] == pytest.approx(20550.25)
    assert liquidation["quantity"] == pytest.approx(0.35)
    assert liquidation["side"].lower() == "sell"
    assert liquidation["time"] == timestamp.isoformat().replace("+00:00", "Z")


def test_engine_event_market_order_book_snapshot_builder() -> None:
    time_engine = dt.datetime(2025, 4, 5, 6, 7, 8, tzinfo=dt.timezone.utc)
    time_exchange = time_engine + dt.timedelta(seconds=1)

    event = bp.EngineEvent.market_order_book_snapshot(
        "binance_spot",
        3,
        sequence=12345,
        time_engine=time_engine,
        bids=[(100.5, 2.0), (100.0, 1.5)],
        asks=[(101.0, 1.0), (101.5, 0.5)],
        time_exchange=time_exchange,
    )

    market = event.to_dict()["Market"]["Item"]
    order_book_event = market["kind"]["OrderBook"]["Snapshot"]

    assert order_book_event["sequence"] == 12345
    assert order_book_event["time_engine"] == time_engine.isoformat().replace(
        "+00:00", "Z"
    )

    bids = order_book_event["bids"]["levels"]
    assert len(bids) == 2
    # Bids sorted descending by price
    assert Decimal(bids[0]["price"]) == Decimal("100.5")
    assert Decimal(bids[0]["amount"]) == Decimal("2")
    assert Decimal(bids[1]["price"]) == Decimal("100")
    assert Decimal(bids[1]["amount"]) == Decimal("1.5")

    asks = order_book_event["asks"]["levels"]
    assert len(asks) == 2
    # Asks sorted ascending by price
    assert Decimal(asks[0]["price"]) == Decimal("101")
    assert Decimal(asks[0]["amount"]) == Decimal("1")
    assert Decimal(asks[1]["price"]) == Decimal("101.5")
    assert Decimal(asks[1]["amount"]) == Decimal("0.5")


def test_engine_event_market_reconnecting_builder() -> None:
    event = bp.EngineEvent.market_reconnecting("kraken")

    payload = event.to_dict()
    reconnecting = payload["Market"]["Reconnecting"]

    assert reconnecting == "kraken"


def test_engine_event_account_reconnecting_builder() -> None:
    event = bp.EngineEvent.account_reconnecting("binance_spot")

    payload = event.to_dict()
    reconnecting = payload["Account"]["Reconnecting"]

    assert reconnecting == "binance_spot"


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
    summary = bp.run_historic_backtest(config, str(example_paths["market_data"]))

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


def test_system_handle_abort(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(config, trading_enabled=False)

    assert handle.is_running()

    handle.abort()

    assert not handle.is_running()

    with pytest.raises(ValueError):
        handle.shutdown()


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


def test_order_snapshot_open_helper() -> None:
    timestamp = dt.datetime(2025, 9, 10, 11, 12, 13, tzinfo=dt.timezone.utc)
    key = bp.OrderKey(1, 2, "strategy-alpha", "cid-1")
    open_request = bp.OrderRequestOpen(
        key,
        "buy",
        105.25,
        0.75,
        kind="limit",
        time_in_force="good_until_cancelled",
        post_only=True,
    )

    snapshot = bp.OrderSnapshot.from_open_request(
        open_request,
        order_id="order-789",
        time_exchange=timestamp,
        filled_quantity=0.25,
    )

    event = bp.EngineEvent.account_order_snapshot(exchange=1, snapshot=snapshot)

    account_item = event.to_dict()["Account"]["Item"]
    order_snapshot = account_item["kind"]["OrderSnapshot"]

    assert account_item["exchange"] == 1
    assert order_snapshot["key"]["exchange"] == 1
    assert order_snapshot["key"]["instrument"] == 2
    assert order_snapshot["key"]["strategy"] == "strategy-alpha"
    assert order_snapshot["side"].lower() == "buy"
    assert Decimal(order_snapshot["price"]).quantize(Decimal("0.01")) == Decimal(
        "105.25"
    )
    assert Decimal(order_snapshot["quantity"]).quantize(Decimal("0.01")) == Decimal(
        "0.75"
    )
    assert order_snapshot["kind"] == "Limit"
    assert order_snapshot["time_in_force"]["GoodUntilCancelled"]["post_only"] is True

    active_state = order_snapshot["state"]["Active"]["Open"]
    assert active_state["id"] == "order-789"
    assert active_state["time_exchange"] == timestamp.isoformat().replace("+00:00", "Z")
    assert Decimal(active_state["filled_quantity"]) == Decimal("0.25")


def test_order_snapshot_open_inflight_helper() -> None:
    key = bp.OrderKey(3, 4, "strategy-beta", "cid-2")
    open_request = bp.OrderRequestOpen(
        key,
        "sell",
        250.0,
        1.5,
        kind="limit",
    )

    snapshot = bp.OrderSnapshot.from_open_request(open_request)

    event = bp.EngineEvent.account_order_snapshot(exchange=3, snapshot=snapshot)

    order_snapshot = event.to_dict()["Account"]["Item"]["kind"]["OrderSnapshot"]

    assert order_snapshot["key"]["exchange"] == 3
    assert order_snapshot["key"]["instrument"] == 4
    assert order_snapshot["side"].lower() == "sell"
    assert "OpenInFlight" in order_snapshot["state"]["Active"]


def test_account_order_cancelled_helper() -> None:
    timestamp = dt.datetime(2025, 12, 1, 2, 3, 4, tzinfo=dt.timezone.utc)
    key = bp.OrderKey(2, 5, "strategy-gamma", "cid-3")
    cancel_request = bp.OrderRequestCancel(key, "order-456")

    event = bp.EngineEvent.account_order_cancelled(
        exchange=2,
        request=cancel_request,
        order_id="order-456",
        time_exchange=timestamp,
    )

    cancelled = event.to_dict()["Account"]["Item"]["kind"]["OrderCancelled"]

    assert cancelled["key"]["exchange"] == 2
    assert cancelled["key"]["instrument"] == 5
    assert cancelled["state"]["Ok"]["id"] == "order-456"
    assert cancelled["state"]["Ok"]["time_exchange"] == timestamp.isoformat().replace(
        "+00:00", "Z"
    )


def test_calculate_rate_of_return() -> None:
    metric = bp.calculate_rate_of_return(mean_return=0.01, interval="daily")

    assert metric.interval == "Daily"
    assert metric.value == Decimal("0.01")

    # Test scaling to annual
    annual = bp.calculate_rate_of_return(
        mean_return=0.01, interval="daily", target_interval="annual_365"
    )

    assert annual.interval == "Annual(365)"
    # Approximate annualization: (1 + 0.01)^365 - 1 ≈ 3.651757
    assert annual.value > Decimal("3.6")
    assert annual.value < Decimal("3.7")


def test_calculate_profit_factor() -> None:
    factor = bp.calculate_profit_factor(profits_gross_abs=100.0, losses_gross_abs=50.0)

    assert factor is not None
    assert factor == Decimal("2.0")

    # No losses (returns MAX)
    no_losses = bp.calculate_profit_factor(
        profits_gross_abs=100.0, losses_gross_abs=0.0
    )
    assert no_losses is not None
    assert str(no_losses) == "79228162514264337593543950335"  # Decimal::MAX


def test_calculate_win_rate() -> None:
    rate = bp.calculate_win_rate(wins=7, total=10)

    assert rate is not None
    assert rate == Decimal("0.7")

    # No trades
    no_trades = bp.calculate_win_rate(wins=0, total=0)
    assert no_trades is None


def test_exchange_id_constants() -> None:
    assert str(bp.ExchangeId.BINANCE_SPOT) == "BinanceSpot"
    assert str(bp.ExchangeId.COINBASE) == "Coinbase"
    assert str(bp.ExchangeId.KRAKEN) == "Kraken"


def test_sub_kind_constants() -> None:
    assert str(bp.SubKind.PUBLIC_TRADES) == "PublicTrades"
    assert str(bp.SubKind.ORDER_BOOKS_L1) == "OrderBooksL1"
    assert str(bp.SubKind.ORDER_BOOKS_L3) == "OrderBooksL3"
    assert str(bp.SubKind.LIQUIDATIONS) == "Liquidations"
    assert str(bp.SubKind.CANDLES) == "Candles"


def test_subscription_creation() -> None:
    sub = bp.Subscription(
        bp.ExchangeId.BINANCE_SPOT, "btc", "usdt", bp.SubKind.PUBLIC_TRADES
    )

    assert sub.exchange == bp.ExchangeId.BINANCE_SPOT
    assert sub.kind == bp.SubKind.PUBLIC_TRADES
    assert "btc_usdt_spot" in sub.instrument

    # Test string representation
    assert "Subscription" in str(sub)
    assert "BinanceSpot" in str(sub)

    sub_l3 = bp.Subscription(
        bp.ExchangeId.BINANCE_SPOT, "btc", "usdt", bp.SubKind.ORDER_BOOKS_L3
    )
    assert sub_l3.kind == bp.SubKind.ORDER_BOOKS_L3

    sub_candles = bp.Subscription(
        bp.ExchangeId.BINANCE_SPOT, "btc", "usdt", bp.SubKind.CANDLES
    )
    assert sub_candles.kind == bp.SubKind.CANDLES


def test_dynamic_streams_placeholder() -> None:
    streams = bp.DynamicStreams()

    # Test that methods exist (even if they return None for now)
    result = streams.select_trades(bp.ExchangeId.BINANCE_SPOT)
    assert result is None

    result = streams.select_all_trades()
    assert result is None


class TestWelfordOnlineAlgorithms:
    """Test Welford online algorithm functions bound from Rust."""

    def test_welford_calculate_mean(self) -> None:
        # Test case: dataset = [0.1, -0.2, -0.05, 0.2, 0.15, -0.17]
        # TC0: first value
        result = bp.welford_calculate_mean(0.0, 0.1, 1.0)
        assert result == Decimal("0.1")

        # TC1: second value
        result = bp.welford_calculate_mean(0.1, -0.2, 2.0)
        assert result == Decimal("-0.05")

        # TC2: third value
        result = bp.welford_calculate_mean(-0.05, -0.05, 3.0)
        assert result == Decimal("-0.05")

        # TC3: fourth value
        result = bp.welford_calculate_mean(-0.05, 0.2, 4.0)
        assert result == Decimal("0.0125")

        # TC4: fifth value
        result = bp.welford_calculate_mean(0.0125, 0.15, 5.0)
        assert result == Decimal("0.04")

        # TC5: sixth value
        result = bp.welford_calculate_mean(0.04, -0.17, 6.0)
        assert result == Decimal("0.005")

    def test_welford_calculate_recurrence_relation_m(self) -> None:
        # Test cases from Rust implementation
        # dataset_1 = [10, 100, -10]
        result = bp.welford_calculate_recurrence_relation_m(0.0, 0.0, 10.0, 10.0)
        assert result == Decimal("0.0")

        result = bp.welford_calculate_recurrence_relation_m(0.0, 10.0, 100.0, 55.0)
        assert result == Decimal("4050.0")

        result = bp.welford_calculate_recurrence_relation_m(
            4050.0, 55.0, -10.0, Decimal("33.333333333333333333")
        )
        assert result == Decimal("6866.66666666666710")

        # dataset_2 = [-5, -50, -1000]
        result = bp.welford_calculate_recurrence_relation_m(0.0, 0.0, -5.0, -5.0)
        assert result == Decimal("0.0")

        result = bp.welford_calculate_recurrence_relation_m(0.0, -5.0, -50.0, -27.5)
        assert result == Decimal("1012.5")

        result = bp.welford_calculate_recurrence_relation_m(
            1012.5, -27.5, -1000.0, Decimal("-351.666666666666666667")
        )
        assert result == Decimal("631516.66666666663425")

    def test_welford_calculate_sample_variance(self) -> None:
        # Test cases from Rust implementation
        result = bp.welford_calculate_sample_variance(0.0, 1)
        assert result == Decimal("0.0")

        result = bp.welford_calculate_sample_variance(1050.0, 5)
        assert result == Decimal("262.5")

        result = bp.welford_calculate_sample_variance(1012.5, 123223)
        assert result == Decimal("0.0082168768564055120027267858")

        result = bp.welford_calculate_sample_variance(16200000000.0, 3)
        assert result == Decimal("8100000000.0")

        result = bp.welford_calculate_sample_variance(99999.9999, 23232)
        assert result == Decimal("4.3045929964271878093926219276")

    def test_welford_calculate_population_variance(self) -> None:
        # Test cases from Rust implementation
        result = bp.welford_calculate_population_variance(0.0, 1)
        assert result == Decimal("0.0")

        result = bp.welford_calculate_population_variance(1050.0, 5)
        assert result == Decimal("210.0")

        result = bp.welford_calculate_population_variance(1012.5, 123223)
        assert result == Decimal("0.0082168101734254157097295148")

        result = bp.welford_calculate_population_variance(16200000000.0, 3)
        assert result == Decimal("5400000000.0")

        result = bp.welford_calculate_population_variance(99999.9999, 23232)
        assert result == Decimal("4.3044077091942148760330578512")
