# Barter Python Bindings

Python bindings for the [Barter](https://github.com/barter-rs/barter-rs) trading engine built with [PyO3](https://pyo3.rs/).

## Quickstart

```
maturin develop
python -c "import barter_python as bp; print(bp.shutdown_event().is_terminal())"

# Retrieve a trading summary when shutting down a running system
python - <<'PY'
import barter_python as bp

config = bp.SystemConfig.from_json("../barter/examples/config/system_config.json")
handle = bp.start_system(config, trading_enabled=False)
summary = handle.shutdown_with_summary()

print("Summary start:", summary.time_engine_start)
print("Instrument keys:", list(summary.instruments.keys()))
first_name, tear_sheet = next(iter(summary.instruments.items()))
print(first_name, "PnL", tear_sheet.pnl, "Sharpe", tear_sheet.sharpe_ratio.value)

# Convert to plain Python objects if desired
summary_dict = summary.to_dict()
print(summary_dict["instruments"][first_name]["pnl"])
PY
```

## Development

- Requires Python 3.9+
- Install maturin: `pip install maturin`
- Build: `maturin develop`
- Test (Rust): `cargo test -p barter-python`
- Test (Python): `pytest -q tests_py`
