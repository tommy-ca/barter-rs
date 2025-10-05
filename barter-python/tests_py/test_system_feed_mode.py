"""Tests covering the system engine feed mode configuration exposed to Python."""

from __future__ import annotations

from pathlib import Path

import pytest

import barter_python as bp


def test_feed_mode_constants() -> None:
    assert bp.ENGINE_FEED_MODE_STREAM == "stream"
    assert bp.ENGINE_FEED_MODE_ITERATOR == "iterator"
    assert bp.ENGINE_FEED_MODES == (
        bp.ENGINE_FEED_MODE_STREAM,
        bp.ENGINE_FEED_MODE_ITERATOR,
    )


def test_start_system_iterator_mode(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))
    handle = bp.start_system(
        config,
        trading_enabled=False,
        engine_feed_mode="iterator",
    )

    try:
        assert handle.is_running()
    finally:
        handle.shutdown()


def test_run_historic_backtest_iterator_mode(example_paths: dict[str, Path]) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))

    summary = bp.run_historic_backtest(
        config,
        str(example_paths["market_data"]),
        engine_feed_mode="iterator",
    )

    assert isinstance(summary, bp.TradingSummary)


@pytest.mark.parametrize("mode", ["invalid", "STREAM"], ids=["unknown", "uppercase"])
def test_start_system_feed_mode_validation(
    example_paths: dict[str, Path], mode: str
) -> None:
    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))

    if mode.upper() == "STREAM":
        handle = bp.start_system(config, engine_feed_mode=mode)
        try:
            assert handle.is_running()
        finally:
            handle.shutdown()
        return

    with pytest.raises(ValueError, match="engine_feed_mode"):
        bp.start_system(config, engine_feed_mode=mode)
