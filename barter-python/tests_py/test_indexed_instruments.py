"""Tests for the IndexedInstruments Python bindings."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

import pytest

import barter_python as bp


def _spot_definition(exchange: bp.ExchangeId, pair: str) -> dict[str, Any]:
    base, quote = pair.lower().split("/") if "/" in pair else (pair[:-4].lower(), pair[-4:].lower())
    exchange_value = str(exchange)
    if "." in exchange_value:
        exchange_value = exchange_value.split(".", 1)[1]
    exchange_value = re.sub(r"(?<!^)(?=[A-Z])", "_", exchange_value).lower()

    return {
        "exchange": exchange_value,
        "name_exchange": pair.replace("/", ""),
        "underlying": {"base": base, "quote": quote},
        "quote": "underlying_quote",
        "kind": "spot",
    }


class TestIndexedInstrumentsBindings:
    def test_from_definitions_lookup_helpers(self) -> None:
        definitions = [
            _spot_definition(bp.ExchangeId.BINANCE_SPOT, "BTC/USDT"),
            _spot_definition(bp.ExchangeId.BINANCE_SPOT, "ETH/USDT"),
        ]

        indexed = bp.IndexedInstruments.from_definitions(definitions)

        exchange_index = indexed.exchange_index(bp.ExchangeId.BINANCE_SPOT)
        assert exchange_index.index == 0

        round_trip_id = indexed.exchange_id(exchange_index)
        assert round_trip_id == bp.ExchangeId.BINANCE_SPOT

        btc_index = indexed.asset_index(bp.ExchangeId.BINANCE_SPOT, "btc")
        btc_asset = indexed.asset(btc_index)
        assert btc_asset.name_internal == "btc"
        assert btc_asset.name_exchange.lower() == "btc"

        btc_usdt_index = indexed.instrument_index_from_exchange_name(
            bp.ExchangeId.BINANCE_SPOT,
            "BTCUSDT",
        )
        instrument = indexed.instrument(btc_usdt_index)
        assert instrument["name_exchange"] == "BTCUSDT"
        base_asset = indexed.asset(bp.AssetIndex(instrument["underlying"]["base"]))
        assert base_asset.name_internal == "btc"

        with pytest.raises(ValueError):
            indexed.asset_index(bp.ExchangeId.BINANCE_SPOT, "doge")

    def test_from_system_config(self, example_paths: dict[str, Path]) -> None:
        config_path = example_paths["system_config"]
        config = bp.SystemConfig.from_json(str(config_path))

        indexed = bp.IndexedInstruments.from_system_config(config)

        exchange_index = indexed.exchange_index(bp.ExchangeId.BINANCE_SPOT)
        assert exchange_index.index == 0

        sol_index = indexed.instrument_index_from_exchange_name(
            bp.ExchangeId.BINANCE_SPOT,
            "SOLUSDT",
        )
        instrument = indexed.instrument(sol_index)
        sol_asset = indexed.asset(bp.AssetIndex(instrument["underlying"]["base"]))
        assert sol_asset.name_internal == "sol"

        with pytest.raises(ValueError):
            indexed.exchange_index(bp.ExchangeId.BITFINEX)
