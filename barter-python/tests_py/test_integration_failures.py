from __future__ import annotations

from pathlib import Path

import pytest

import barter_python as bp


@pytest.mark.integration
def test_failure_surfaces(tmp_path: Path, example_paths: dict[str, Path]) -> None:
    """Validate common error paths surface ValueError to Python callers."""

    with pytest.raises(ValueError):
        bp.SystemConfig.from_json_str("not valid json")

    config = bp.SystemConfig.from_json(str(example_paths["system_config"]))

    missing_path = tmp_path / "missing_market_data.json"
    with pytest.raises(ValueError):
        bp.run_historic_backtest(config, str(missing_path))

    handle = bp.start_system(config)

    summary = handle.shutdown_with_summary()
    assert summary is not None

    with pytest.raises(ValueError):
        handle.shutdown_with_summary()
