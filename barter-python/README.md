# Barter Python Bindings

Python bindings for the [Barter](https://github.com/barter-rs/barter-rs) trading engine built with [PyO3](https://pyo3.rs/).

## Table of Contents

- [Installation](#installation)
- [Quickstart](#quickstart)
- [Core Concepts](#core-concepts)
  - [System Configuration](#system-configuration)
  - [Engine Lifecycle](#engine-lifecycle)
  - [Trading Summaries](#trading-summaries)
- [API Reference](#api-reference)
  - [Engine Management](#engine-management)
  - [Backtesting](#backtesting)
  - [Market Data](#market-data)
  - [Analytics](#analytics)
  - [Risk Management](#risk-management)
- [Examples](#examples)
- [Development](#development)
- [Releasing](#releasing)
- [Release Notes](#release-notes)

## Installation

### From PyPI (Recommended)

```bash
pip install barter-python
```

### From Source

```bash
git clone https://github.com/barter-rs/barter-rs.git
cd barter-rs/barter-python
pip install maturin
maturin develop
```

## Quickstart

```bash
# Install
pip install barter-python

# Basic usage
python -c "
import barter_python as bp
config = bp.SystemConfig.from_json('path/to/config.json')
handle = bp.start_system(config, trading_enabled=False)
summary = handle.shutdown_with_summary()
print(f'Total PnL: {summary.total_pnl}')
"
```

### Advanced Examples

```bash
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

# Open limit orders through the mock execution client
python - <<'PY'
from decimal import Decimal

import barter_python as bp

config = bp.MockExecutionConfig()
instrument_map = bp.ExecutionInstrumentMap.from_definitions(
    bp.ExchangeId.MOCK,
    [
        {
            "exchange": "mock",
            "name_exchange": "BTCUSDT",
            "underlying": {"base": "btc", "quote": "usdt"},
            "quote": "underlying_quote",
            "kind": "spot",
        }
    ],
)

with bp.MockExecutionClient(config, instrument_map) as client:
    order = client.open_limit_order(
        "BTCUSDT",
        "sell",
        Decimal("45000"),
        Decimal("0.05"),
        time_in_force="good_until_cancelled",
        post_only=True,
    )
    print(order["kind"], order["time_in_force"])
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

## Analytics

Barter Python provides comprehensive financial analytics functions for evaluating trading performance and risk metrics.

### Risk-Adjusted Return Metrics

#### Sharpe Ratio
Measures risk-adjusted return by comparing excess returns to volatility.

```python
import barter_python as bp

# Calculate Sharpe ratio with daily returns
sharpe = bp.calculate_sharpe_ratio(
    risk_free_return=0.02,  # 2% annual risk-free rate
    mean_return=0.08,       # 8% annual return
    std_dev_returns=0.12,   # 12% volatility
    interval="daily"
)
print(f"Sharpe Ratio: {sharpe.value} ({sharpe.interval})")

# Scale to annual (252 trading days)
sharpe_annual = bp.calculate_sharpe_ratio(
    risk_free_return=0.02,
    mean_return=0.08,
    std_dev_returns=0.12,
    interval="annual_252"
)
```

#### Sortino Ratio
Similar to Sharpe ratio but only considers downside volatility.

```python
sortino = bp.calculate_sortino_ratio(
    risk_free_return=0.02,
    mean_return=0.08,
    std_dev_loss_returns=0.08,  # Only negative returns volatility
    interval="daily"
)
```

#### Calmar Ratio
Risk-adjusted return using Maximum Drawdown as the risk measure.

```python
calmar = bp.calculate_calmar_ratio(
    risk_free_return=0.02,
    mean_return=0.08,
    max_drawdown=0.15,  # 15% maximum drawdown
    interval="annual_365"
)
```

### Performance Metrics

#### Profit Factor
Ratio of gross profits to gross losses.

```python
profit_factor = bp.calculate_profit_factor(
    profits_gross_abs=15000.0,  # $15,000 gross profits
    losses_gross_abs=10000.0    # $10,000 gross losses
)
# Result: 1.5 (profitable strategy)
```

#### Win Rate
Percentage of winning trades.

```python
win_rate = bp.calculate_win_rate(
    wins=65,    # 65 winning trades
    total=100   # 100 total trades
)
# Result: 0.65 (65% win rate)
```

#### Rate of Return
Simple rate of return with interval scaling.

```python
# Calculate daily return
daily_return = bp.calculate_rate_of_return(
    mean_return=0.01,  # 1% daily return
    interval="daily"
)

# Scale to annual return
annual_return = bp.calculate_rate_of_return(
    mean_return=0.01,
    interval="daily",
    target_interval="annual_252"
)
# Result: ~2.52 (252% annual return)
```

### Drawdown Analysis

Track portfolio drawdowns over time from equity curves.

```python
import datetime as dt
from decimal import Decimal

# Equity curve data points
equity_points = [
    (dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc), Decimal("10000")),
    (dt.datetime(2024, 1, 2, tzinfo=dt.timezone.utc), Decimal("10500")),
    (dt.datetime(2024, 1, 3, tzinfo=dt.timezone.utc), Decimal("9800")),
    (dt.datetime(2024, 1, 4, tzinfo=dt.timezone.utc), Decimal("10200")),
    (dt.datetime(2024, 1, 5, tzinfo=dt.timezone.utc), Decimal("9500")),
    (dt.datetime(2024, 1, 6, tzinfo=dt.timezone.utc), Decimal("10800")),
]

# Generate drawdown series
drawdowns = bp.generate_drawdown_series(equity_points)
print(f"Number of drawdown periods: {len(drawdowns)}")

# Calculate maximum drawdown
max_dd = bp.calculate_max_drawdown(equity_points)
if max_dd:
    print(f"Maximum drawdown: {max_dd.drawdown.value}")
    print(f"Duration: {max_dd.drawdown.duration}")

# Calculate mean drawdown
mean_dd = bp.calculate_mean_drawdown(equity_points)
if mean_dd:
    print(f"Mean drawdown: {mean_dd.mean_drawdown}")
    print(f"Mean duration: {mean_dd.mean_duration}")
```

### Welford Online Algorithms

Efficient streaming calculation of statistical moments.

```python
# Calculate running mean incrementally
prev_mean = 10.5
new_value = 12.3
count = 101  # After adding new value

updated_mean = bp.welford_calculate_mean(prev_mean, new_value, count)

# Calculate variance components
prev_m = 45.2
prev_mean = 10.5
new_value = 12.3
new_mean = 10.7

# Update recurrence relation M
m = bp.welford_calculate_recurrence_relation_m(
    prev_m, prev_mean, new_value, new_mean
)

# Calculate variances
sample_variance = bp.welford_calculate_sample_variance(m, count)
population_variance = bp.welford_calculate_population_variance(m, count)
```

### Time Intervals

All ratio calculations support different time intervals:

- `"daily"` - Daily interval
- `"annual_252"` - Annual with 252 trading days
- `"annual_365"` - Annual with 365 calendar days
- `datetime.timedelta` - Custom duration

```python
# Custom interval
import datetime as dt

sharpe = bp.calculate_sharpe_ratio(
    risk_free_return=0.02,
    mean_return=0.08,
    std_dev_returns=0.12,
    interval=dt.timedelta(hours=4)  # 4-hour intervals
)
```

## Development

### Prerequisites
- **uv** (modern Python package manager): `curl -LsSf https://astral.sh/uv/install.sh | sh`
- **Rust toolchain** with components: `rustup component add rustfmt clippy`

### Setup
```bash
# Clone the repository
git clone https://github.com/barter-rs/barter-rs.git
cd barter-rs/barter-python

# Install all dependencies
uv sync --dev

# Install pre-commit hooks
uv run pre-commit install
```

### Development Workflow
```bash
# Format code
make format

# Run linting
make lint

# Run tests
make test

# Run all checks
make check

# Build package
make build
```

### Pre-commit Hooks
The project uses pre-commit hooks to ensure code quality:

- **ruff**: Linting and formatting for Python
- **rustfmt**: Code formatting for Rust
- **clippy**: Linting for Rust
- **General**: Trailing whitespace, file size checks, YAML validation

Run hooks manually:
```bash
uv run pre-commit run --all-files
```

## Core Concepts

### System Configuration

Barter systems are configured using JSON files that define exchanges, instruments, execution clients, and risk parameters:

```python
import barter_python as bp

# Load from JSON file
config = bp.SystemConfig.from_json("path/to/system_config.json")

# Inspect configured exchanges and instruments
print("Exchanges:", [str(ex) for ex in config.exchanges()])
print("Instruments:", len(config.instruments()))
```

### Engine Lifecycle

Start and manage trading systems with simple Python calls:

```python
# Start a live system
handle = bp.start_system(config, trading_enabled=False)

# Check if running
print("Running:", handle.is_running())

# Send events
handle.send_event(bp.EngineEvent.trading_state(True))

# Shutdown with summary
summary = handle.shutdown_with_summary(interval="annual_252")
```

### Trading Summaries

Analyze performance with comprehensive metrics:

```python
# Access summary data
print("Total PnL:", summary.total_pnl)
print("Sharpe Ratio:", summary.sharpe_ratio.value)

# Per-instrument analysis
for name, tear_sheet in summary.instruments.items():
    print(f"{name}: PnL={tear_sheet.pnl}, Sharpe={tear_sheet.sharpe_ratio.value}")
```

#### Incremental Updates

Use `TradingSummaryGenerator` to continue evolving a summary after a backtest or shutdown:

```python
from decimal import Decimal

summary, generator = bp.run_historic_backtest_with_generator(
    config,
    "tests_py/data/synthetic_market_data.json",
    risk_free_return=0.015,
)

instrument_map = bp.ExecutionInstrumentMap.from_system_config(
    bp.ExchangeId.BINANCE_SPOT,
    config,
)
asset_index = instrument_map.asset_index(instrument_map.asset_names()[0])
balance = bp.Balance.new(Decimal("1000"), Decimal("975"))
generator.update_from_balance(bp.AssetBalance.new(asset_index, balance, summary.time_engine_end))

next_summary = generator.generate("annual_365")
print(next_summary.time_engine_end)
```

## API Reference

### Engine Management

#### System Configuration
- `SystemConfig.from_json(path)` - Load configuration from JSON file
- `SystemConfig.from_dict(data)` - Load configuration from dictionary
- `config.to_json()` - Export configuration as JSON string
- `config.exchanges()` - List configured exchanges
- `config.instruments()` - List configured instruments

#### System Control
- `start_system(config, **kwargs)` - Start a trading system
- `run_historic_backtest(config, market_data, **kwargs)` - Run backtest
- `SystemHandle.is_running()` - Check if system is active
- `SystemHandle.send_event(event)` - Send event to running system
- `SystemHandle.shutdown_with_summary(**kwargs)` - Shutdown and get summary
- `SystemHandle.shutdown_with_summary_generator(**kwargs)` - Shutdown and get summary plus generator
- `SystemHandle.abort()` - Immediately terminate system

#### Engine Events
- `EngineEvent.trading_state(enabled)` - Enable/disable trading
- `EngineEvent.account_balance_snapshot(...)` - Balance updates
- `EngineEvent.account_order_snapshot(...)` - Order status updates
- `EngineEvent.account_order_cancelled(...)` - Order cancellations

### Backtesting

#### Market Data
- `MarketDataInMemory.from_json_file(path)` - Load market data
- `market_data.events()` - Iterate through events
- `market_data.time_first_event()` - Get first event timestamp

#### Backtest Execution
- `backtest(args_constant, args_dynamic)` - Run single backtest
- `run_backtests(args_constant, dynamic_args_list)` - Run multiple backtests
- `run_historic_backtest_with_generator(config, market_data, **kwargs)` - Backtest returning a generator for incremental updates

#### Argument Workflow
Use the strongly typed wrappers to supply configuration, market data, and risk settings to the
Rust backtest engine.

```python
from decimal import Decimal

import barter_python as bp
from barter_python import backtest

config = bp.SystemConfig.from_json("examples/config/system_config.json")
market_data = backtest.MarketDataInMemory.from_json_file(
    "tests_py/data/synthetic_market_data.json",
)

args_constant = backtest.BacktestArgsConstant(
    system_config=config,
    market_data=market_data,
    summary_interval="annual_252",  # daily | annual_252 | annual_365
    initial_balances=[
        {"exchange": "binance_spot", "asset": "usdt", "total": "10000", "free": "10000"},
    ],
)

baseline = backtest.BacktestArgsDynamic(
    id="baseline",
    risk_free_return=Decimal("0.02"),
)

alt = backtest.BacktestArgsDynamic(
    id="alternative",
    risk_free_return=Decimal("0.01"),
)

single_summary = backtest.backtest(args_constant, baseline)
multi_summary = backtest.run_backtests(args_constant, [baseline, alt])

print(single_summary.trading_summary.total_pnl)
for summary in multi_summary.summaries:
    print(summary.id, summary.risk_free_return)
```

- `summary_interval` accepts `"daily"`, `"annual_252"`, or `"annual_365"`.
- `initial_balances` lets you seed the engine with balances before replaying market data.
- Custom strategy and risk managers are not yet exposed; the defaults are applied automatically.

### Market Data

#### Subscriptions
- `Subscription(exchange, base, quote, kind)` - Create subscription
- `ExchangeId.*` - Available exchanges (BINANCE_SPOT, COINBASE, etc.)
- `SubKind.*` - Subscription types (PUBLIC_TRADES, ORDER_BOOKS_L1, etc.)

#### Order Books
- `OrderBook(sequence, bids, asks)` - Create order book
- `book.mid_price()` - Calculate mid price
- `book.volume_weighted_mid_price()` - Volume-weighted mid price

### Analytics

#### Risk Metrics
- `calculate_sharpe_ratio(...)` - Sharpe ratio calculation
- `calculate_sortino_ratio(...)` - Sortino ratio calculation
- `calculate_calmar_ratio(...)` - Calmar ratio calculation
- `calculate_max_drawdown(...)` - Maximum drawdown
- `calculate_mean_drawdown(...)` - Mean drawdown

#### Performance Metrics
- `calculate_profit_factor(...)` - Profit factor
- `calculate_win_rate(...)` - Win rate
- `calculate_rate_of_return(...)` - Rate of return

#### Streaming Statistics
- `welford_calculate_mean(...)` - Online mean calculation
- `welford_calculate_sample_variance(...)` - Online variance

### Risk Management

#### Configuration
- `config.risk_limits()` - Get current risk limits
- `config.set_global_risk_limits(limits)` - Set global limits
- `config.set_instrument_risk_limits(index, limits)` - Set per-instrument limits

#### Mock Execution
- `MockExecutionClient(config, instrument_map)` - Create mock client
- `client.account_snapshot()` - Get account state
- `client.open_market_order(...)` - Submit market order
- `client.open_limit_order(...)` - Submit limit order

## Examples

### Basic System Operation

```python
import barter_python as bp

# Load configuration
config = bp.SystemConfig.from_json("config.json")

# Start system
handle = bp.start_system(config, trading_enabled=False)

# Enable trading
handle.send_event(bp.EngineEvent.trading_state(True))

# Shutdown with performance summary
summary = handle.shutdown_with_summary(interval="annual_252")
print(f"Total PnL: {summary.total_pnl}")
```

### Backtesting

```python
import barter_python as bp

# Load configuration and market data
config = bp.SystemConfig.from_json("config.json")
market_data = bp.MarketDataInMemory.from_json_file("market_data.json")

# Run backtest
summary = bp.run_historic_backtest(
    config,
    market_data,
    interval="annual_252",
    initial_balances=[{"exchange": "binance_spot", "asset": "usdt", "total": 10000}]
)

print(f"Sharpe Ratio: {summary.sharpe_ratio.value}")
```

### Risk Management

```python
import barter_python as bp
from decimal import Decimal

config = bp.SystemConfig.from_json("config.json")

# Configure risk limits
config.set_global_risk_limits({
    "max_leverage": Decimal("2.0"),
    "max_position_notional": Decimal("10000"),
})

# Set per-instrument limits
config.set_instrument_risk_limits(0, {
    "max_exposure_percent": Decimal("0.1"),
})
```

### Analytics

```python
import barter_python as bp

# Calculate risk metrics
sharpe = bp.calculate_sharpe_ratio(
    risk_free_return=0.02,
    strategy_return=0.08,
    strategy_std_dev=0.15,
    interval="annual_252"
)

print(f"Sharpe Ratio: {sharpe.value}")
```

### Mock Execution

```python
import barter_python as bp
from decimal import Decimal

# Create mock execution setup
mock_config = bp.MockExecutionConfig(
    mocked_exchange=bp.ExchangeId.BINANCE_SPOT,
    latency_ms=50,
    fees_percent=0.1
)

instrument_map = bp.ExecutionInstrumentMap.from_definitions(
    bp.ExchangeId.MOCK,
    [{"exchange": "mock", "name_exchange": "BTCUSDT", "underlying": {"base": "btc", "quote": "usdt"}, "quote": "underlying_quote", "kind": "spot"}]
)

# Use mock client
with bp.MockExecutionClient(mock_config, instrument_map) as client:
    order = client.open_limit_order(
        "BTCUSDT", "buy", Decimal("50000"), Decimal("0.1"),
        time_in_force="good_until_cancelled"
    )
    print(f"Order placed: {order}")
```

## Development

### Prerequisites
- Python 3.9+
- Rust 1.70+

### Setup
```bash
# Install maturin for building
pip install maturin

# Build in development mode
maturin develop

# Run tests
pytest tests_py/
cargo test -p barter-python
```

### Testing
- **Python tests**: `pytest tests_py/` (requires `maturin develop` first)
- **Rust tests**: `cargo test -p barter-python`
- **Integration tests**: `pytest tests_py/test_integration_*.py`

## Releasing

### Automated Release Process
1. Update version in `barter-python/Cargo.toml`
2. Update `barter-python/README.md` release notes
3. Push changes to `main`
4. Create and push a version tag: `git tag v0.1.3 && git push origin v0.1.3`
5. CI automatically builds wheels and publishes to PyPI

### Manual Testing
```bash
# Build wheels locally
maturin build --release

# Test installation
pip install target/wheels/*.whl --force-reinstall
python -c "import barter_python; print('Installation successful')"
```

## Release Notes

### 0.1.2 — 2025-10-04
- Version bump to reflect analytics helpers stabilization.

### 0.1.1 — 2025-10-04
- Added `calculate_profit_factor` and `calculate_win_rate` helpers
- Added `calculate_rate_of_return` with interval scaling
- Added drawdown analysis helpers
- Expanded pytest coverage for analytics

### 0.1.0 — 2025-10-04
- Initial PyO3-based bindings for Barter engine
- Python access to risk manager configuration
- Portfolio analytics helpers (Sharpe, Sortino ratios)
- CLI backtest helper

---

For the latest updates, see the [Barter repository](https://github.com/barter-rs/barter-rs).
