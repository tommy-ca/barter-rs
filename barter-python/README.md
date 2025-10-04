# Barter Python Bindings

Python bindings for the [Barter](https://github.com/barter-rs/barter-rs) trading engine built with [PyO3](https://pyo3.rs/).

## Quickstart

```
maturin develop
python -c "import barter_python as bp; print(bp.shutdown_event().is_terminal())"

# Build an account balance snapshot event
python - <<'PY'
import datetime as dt
import barter_python as bp

snapshot = bp.EngineEvent.account_balance_snapshot(
    exchange=0,
    asset=1,
    total=125.5,
    free=100.0,
    time_exchange=dt.datetime(2024, 1, 2, tzinfo=dt.timezone.utc),
)

print(snapshot.to_json())
PY

# Retrieve a trading summary when shutting down a running system
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(config, trading_enabled=False)
summary = handle.shutdown_with_summary(interval="annual_365")

print("Summary start:", summary.time_engine_start)
print("Instrument keys:", list(summary.instruments.keys()))
first_name, tear_sheet = next(iter(summary.instruments.items()))
print(first_name, "PnL", tear_sheet.pnl, "Sharpe", tear_sheet.sharpe_ratio.value)

# Convert to plain Python objects if desired
summary_dict = summary.to_dict()
print(summary_dict["instruments"][first_name]["pnl"])
PY

# Abort a running system immediately without waiting for a summary
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(config, trading_enabled=False)
handle.abort()

print("Running after abort:", handle.is_running())
PY

# Run the packaged CLI to execute a historic backtest
barter-backtest \
  --config ../barter/examples/config/system_config.json \
  --market-data ../barter/examples/data/binance_spot_market_data_with_disconnect_events.json \
  --interval annual-252 \
  --pretty

# Alternatively invoke the example script directly during development
python examples/backtest_cli.py \
  --config ../barter/examples/config/system_config.json \
  --market-data ../barter/examples/data/binance_spot_market_data_with_disconnect_events.json \
  --interval annual-365 \
  --pretty
```

Both `SystemHandle.shutdown_with_summary` and `run_historic_backtest` accept an optional
`interval` argument (`"daily"`, `"annual_252"`, or `"annual_365"`, case-insensitive) which controls
how risk and return metrics are annualised in the generated trading summary.

### Risk Configuration

System configurations expose risk manager thresholds that can be inspected or adjusted before
starting a system:

```python
import decimal

import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")

# Inspect existing limits (None if only JSON defaults are present)
print(config.risk_limits()["global"])  # -> None

# Set global and per-instrument overrides
config.set_global_risk_limits({
    "max_leverage": decimal.Decimal("2.75"),
    "max_position_notional": decimal.Decimal("5000"),
})

config.set_instrument_risk_limits(
    0,
    {
        "max_exposure_percent": decimal.Decimal("0.2"),
        "max_position_quantity": decimal.Decimal("1.5"),
    },
)

# Fetch the instrument-specific override
limits = config.get_instrument_risk_limits(0)
print(limits["max_exposure_percent"])  # -> Decimal('0.2')

# Persist the updated configuration
config.to_json_file("/tmp/system_config_with_risk.json")
```

### Account Event Helpers

Python connectors can construct account events without building raw dictionaries. For example,
order snapshots and cancellation responses can be emitted using the dedicated helpers:

```python
import datetime as dt

import barter_python as bp

key = bp.OrderKey(1, 2, "strategy-alpha", "cid-123")
open_request = bp.OrderRequestOpen(
    key,
    "buy",
    price=105.25,
    quantity=0.75,
    kind="limit",
    time_in_force="good_until_cancelled",
    post_only=True,
)

snapshot = bp.OrderSnapshot.from_open_request(
    open_request,
    order_id="order-789",
    time_exchange=dt.datetime(2025, 9, 10, 11, 12, 13, tzinfo=dt.timezone.utc),
    filled_quantity=0.25,
)

order_event = bp.EngineEvent.account_order_snapshot(exchange=1, snapshot=snapshot)

cancel_request = bp.OrderRequestCancel(key, "order-789")
cancel_event = bp.EngineEvent.account_order_cancelled(
    exchange=1,
    request=cancel_request,
    order_id="order-789",
    time_exchange=dt.datetime(2025, 9, 10, 11, 13, 0, tzinfo=dt.timezone.utc),
)
```

These helpers validate inputs (for example ensuring filled quantity never exceeds the requested
order size) and return strongly typed `EngineEvent` instances that can be fed directly into a
running system.

## Development

- Requires Python 3.9+
- Install maturin: `pip install maturin`
- Build: `maturin develop`
- Rust tests that exercise the Python embedding layer: `cargo test -p barter-python --features python-tests`
  - The `python-tests` feature enables the `extension-module` flag so `libpython` is linked only when needed.
- Python tests (after `maturin develop`): `pytest -q tests_py`

## Releasing

- Wheels are built automatically by the `Build Python Wheels` workflow for Python 3.9–3.12 across
  Linux, macOS and Windows whenever `main` is updated or a tag matching `v*` is pushed.
- To publish to PyPI, configure the `PYPI_API_TOKEN` secret in the repository and push a release tag
  (for example `v0.1.0`). The workflow downloads the matrix artifacts and uploads them to PyPI in a
  dedicated `Publish to PyPI` job.
- Manual `workflow_dispatch` runs can be used to validate wheel builds before cutting a release; they
  will skip the publish step unless the run targets a `v*` tag and the secret is available.

## Release Notes

Track the highlights from each coordinated Rust and Python release here. Align the entries with the
cadence defined in `.agent/specs/release-cadence.md` so both ecosystems stay in sync.

### 0.1.0 — 2025-10-04

- Initial PyO3-based bindings for the Barter engine alongside CLI backtest helper.
- Python access to risk manager thresholds with JSON round-trip support.
- Portfolio analytics helpers (Sharpe, Sortino) exposed for summary inspection.

For future releases, add a new heading with the version, release date, and 3–5 bullet points
highlighting the most impactful changes across both the Rust workspace and the Python package.
