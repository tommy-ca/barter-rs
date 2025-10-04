import decimal

import barter_python as bp


def _load_config(example_paths) -> bp.SystemConfig:
    return bp.SystemConfig.from_json(str(example_paths["system_config"]))


def test_risk_limits_round_trip(example_paths, tmp_path):
    config = _load_config(example_paths)

    risk = config.risk_limits()
    assert risk["global"] is None
    assert risk["instruments"] == []

    config.set_global_risk_limits(
        {
            "max_leverage": decimal.Decimal("2.75"),
            "max_position_notional": 5_000,
        }
    )

    config.set_instrument_risk_limits(
        1,
        {
            "max_exposure_percent": decimal.Decimal("0.2"),
            "max_position_quantity": 1.5,
        },
    )

    risk = config.risk_limits()
    assert risk["global"]["max_leverage"] == decimal.Decimal("2.75")
    assert risk["global"]["max_position_notional"] == decimal.Decimal("5000")

    entries = {entry["index"]: entry["limits"] for entry in risk["instruments"]}
    assert entries[1]["max_exposure_percent"] == decimal.Decimal("0.2")
    assert entries[1]["max_position_quantity"] == decimal.Decimal("1.5")

    json_repr = config.to_json()
    assert "\"max_exposure_percent\": \"0.2\"" in json_repr

    config.set_instrument_risk_limits(1, None)
    assert config.get_instrument_risk_limits(1) is None

    config.set_global_risk_limits(None)
    assert config.risk_limits()["global"] is None
