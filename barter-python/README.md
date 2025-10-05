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

# Inspect indexed instruments
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
indexed = bp.IndexedInstruments.from_system_config(config)

binance = bp.ExchangeId.BINANCE_SPOT
btc_index = indexed.instrument_index_from_exchange_name(binance, "BTCUSDT")
btc_asset = indexed.asset(indexed.asset_index(binance, "btc"))

print("Exchange index:", indexed.exchange_index(binance).index)
print("Instrument index:", btc_index.index)
print("BTC asset exchange symbol:", btc_asset.name_exchange)
PY

# Seed initial balances when starting from Python
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(
    config,
    trading_enabled=False,
    initial_balances=[
        {"exchange": "binance_spot", "asset": "usdt", "total": 4321.5, "free": 2100.25},
    ],
)
summary = handle.shutdown_with_summary()

assets = summary.assets
print(assets["binance_spot:usdt"].balance_end.total)
PY

# Configure mock execution directly from Rust bindings
python - <<'PY'
import barter_python as bp

mock_execution = bp.MockExecutionConfig(
    mocked_exchange=bp.ExchangeId.BINANCE_SPOT,
    latency_ms=25,
    fees_percent=0.25,
)

execution_config = bp.ExecutionConfig.mock(mock_execution)
config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
config.clear_executions()
config.add_execution(execution_config)

print(execution_config.to_dict())
PY

# Calculate portfolio analytics directly from summary inputs
python - <<'PY'
import barter_python as bp

calmar = bp.calculate_calmar_ratio(
    risk_free_return=0.0015,
    mean_return=0.0025,
    max_drawdown=0.02,
    interval="daily",
)

print(calmar.interval, calmar.value)
PY

# Generate drawdown statistics from equity points
python - <<'PY'
import datetime as dt
from decimal import Decimal

import barter_python as bp

base = dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc)
points = [
    (base, Decimal("100")),
    (base + dt.timedelta(days=1), Decimal("110")),
    (base + dt.timedelta(days=2), Decimal("90")),
    (base + dt.timedelta(days=3), Decimal("115")),
]

drawdowns = bp.generate_drawdown_series(points)
max_drawdown = bp.calculate_max_drawdown(points)
mean_drawdown = bp.calculate_mean_drawdown(points)

 print("drawdowns", [d.value for d in drawdowns])
 print("max value", max_drawdown.value if max_drawdown else None)
 print("mean value", mean_drawdown.mean_drawdown if mean_drawdown else None)
 PY

 # Use Welford online algorithms for streaming statistics
 python - <<'PY'
 import barter_python as bp

 # Calculate running mean incrementally
 prev_mean = 10.0
 new_value = 15.0
 count = 6  # After adding the new value

 mean = bp.welford_calculate_mean(prev_mean, new_value, count)
 print("Updated mean:", mean)

 # Calculate variance components
 prev_m = 50.0
 prev_mean = 12.5
 new_value = 18.0
 new_mean = 13.2

 m = bp.welford_calculate_recurrence_relation_m(prev_m, prev_mean, new_value, new_mean)
 sample_variance = bp.welford_calculate_sample_variance(m, 10)
 population_variance = bp.welford_calculate_population_variance(m, 10)

 print("Sample variance:", sample_variance)
 print("Population variance:", population_variance)
 PY
 
# Use pure Python strategy implementations
python - <<'PY'
import barter_python as bp
from barter_python.strategy import (
    EngineState,
    InstrumentState,
    Position,
    close_open_positions_with_market_orders,
)

# Create a strategy ID
strategy_id = bp.StrategyId.new("close-positions")

# Create mock engine state with positions to close
instruments = [
    bp.strategy.InstrumentState(
        instrument=0,
        exchange=0,
        position=bp.strategy.Position(0, bp.Side.BUY, 100.0, 50000.0),
        price=51000.0,
    ),
    bp.strategy.InstrumentState(
        instrument=1,
        exchange=0,
        position=bp.strategy.Position(1, bp.Side.SELL, 50.0, 30000.0),
        price=29500.0,
    ),
]

state = bp.strategy.EngineState(instruments)

# Generate orders to close all positions
cancel_requests, open_requests = close_open_positions_with_market_orders(strategy_id, state)

print("Cancel requests:", len(list(cancel_requests)))
print("Open requests:", len(list(open_requests)))
PY

# Create market data subscriptions for streaming
python - <<'PY'
import barter_python as bp

# Create a subscription for BTC/USDT public trades on Binance Spot
subscription = bp.Subscription(
    bp.ExchangeId.BINANCE_SPOT,
    "btc",
    "usdt",
    bp.SubKind.PUBLIC_TRADES
)

print("Subscription:", subscription)
print("Exchange:", subscription.exchange)
print("Instrument:", subscription.instrument)
print("Kind:", subscription.kind)
PY

# Abort a running system immediately without waiting for a summary
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(config, trading_enabled=False)
handle.abort()

print("Running after abort:", handle.is_running())
PY

# Enable audit streaming for live systems
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(config, trading_enabled=False, audit=True)

audit = handle.take_audit()
print("Snapshot summary:", audit.snapshot.value)

handle.send_event(bp.EngineEvent.trading_state(True))
update = audit.updates.recv(timeout=1.0)
print("First audit update:", update)

typed_update = audit.updates.recv_tick(timeout=1.0)
print("Typed audit kind:", typed_update.event.kind)
print("Outputs count:", typed_update.event.output_count)

handle.shutdown()
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

# Generate statistical trading summaries from synthetic data
python examples/statistical_trading_summary.py
```

Both `SystemHandle.shutdown_with_summary` and `run_historic_backtest` accept an optional
`interval` argument (`"daily"`, `"annual_252"`, or `"annual_365"`, case-insensitive) which controls
how risk and return metrics are annualised in the generated trading summary.

`start_system` and `run_historic_backtest` also accept an `initial_balances` keyword. Provide an
iterable of mapping objects with `exchange`, `asset`, `total`, and optional `free` fields (using the
snake_case `ExchangeId` names such as `binance_spot`) to seed the engine's account balances before
execution. When omitted, balances default to the data loaded from the system configuration.

Both entry points expose an `engine_feed_mode` keyword to control whether the engine processes
events using the asynchronous stream runner (`"stream"`, default) or the synchronous iterator
runner (`"iterator"`). The value is case-insensitive and validated before initialising the system.

Validation rules:
- `exchange` must map to a known `ExchangeId`; unknown identifiers raise `ValueError`.
- `free` defaults to the provided `total` when omitted and must never exceed `total`.
- `total` and `free` must be finite numbers that can be converted via the binding's decimal parser.

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

### Market Data Streaming

The bindings expose types for creating market data subscriptions that can be used with the
underlying Barter data streaming infrastructure:

```python
import barter_python as bp

# Available exchanges (all ExchangeId variants are exposed)
print(bp.ExchangeId.BINANCE_SPOT)   # BinanceSpot
print(bp.ExchangeId.COINBASE)       # Coinbase
print(bp.ExchangeId.OTHER)          # Other

# Available subscription kinds
print(bp.SubKind.PUBLIC_TRADES)    # PublicTrades
print(bp.SubKind.ORDER_BOOKS_L1)   # OrderBooksL1
print(bp.SubKind.LIQUIDATIONS)     # Liquidations

# Create subscriptions for different instruments
btc_trades = bp.Subscription(bp.ExchangeId.BINANCE_SPOT, "btc", "usdt", bp.SubKind.PUBLIC_TRADES)
eth_trades = bp.Subscription(bp.ExchangeId.BINANCE_SPOT, "eth", "usdt", bp.SubKind.PUBLIC_TRADES)

print(btc_trades.instrument)  # btc_usdt_spot
```

**Note:** Full async streaming functionality is under development. The current bindings provide
the foundational types for subscription creation, with stream initialization planned for a future
release.

### OrderBook Analysis

The bindings provide utilities for analyzing order book data, including mid-price calculations and
order book construction:

```python
import barter_python as bp

# Create an order book from bid/ask levels
bids = [(100.0, 1.0), (99.5, 2.0)]  # (price, amount) tuples
asks = [(100.5, 1.5), (101.0, 1.0)]
book = bp.OrderBook(sequence=123, bids=bids, asks=asks)

# Calculate mid-price (average of best bid/ask)
mid_price = book.mid_price()  # "100.25"

# Calculate volume-weighted mid-price
vw_mid_price = book.volume_weighted_mid_price()  # Considers order sizes

# Access raw levels (sorted bids descending, asks ascending)
bids = book.bids()  # [('100', '1'), ('99.5', '2')]
asks = book.asks()  # [('100.5', '1.5'), ('101', '1')]

# Standalone calculation functions
mid = bp.calculate_mid_price(100.0, 101.0)  # "100.5"
vw_mid = bp.calculate_volume_weighted_mid_price(100.0, 2.0, 101.0, 1.0)  # "100.333..."
```

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

### 0.1.2 — 2025-10-04

- Version bump to reflect analytics helpers stabilization.

### 0.1.1 — 2025-10-04

- Added `calculate_profit_factor` and `calculate_win_rate` helpers mirroring trading summary values.
- Added `calculate_rate_of_return` with optional interval scaling support.
- Added drawdown helpers (`generate_drawdown_series`, `calculate_max_drawdown`,
  `calculate_mean_drawdown`) for downside risk analysis.
- Expanded pytest coverage for portfolio analytics edge cases and custom intervals.

### 0.1.0 — 2025-10-04

- Initial PyO3-based bindings for the Barter engine alongside CLI backtest helper.
- Python access to risk manager thresholds with JSON round-trip support.
- Portfolio analytics helpers (Sharpe, Sortino) exposed for summary inspection.

For future releases, add a new heading with the version, release date, and 3–5 bullet points
highlighting the most impactful changes across both the Rust workspace and the Python package.
