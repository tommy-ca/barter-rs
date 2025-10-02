# Barter Python Bindings

Python bindings for the [Barter](https://github.com/barter-rs/barter-rs) trading engine built with [PyO3](https://pyo3.rs/).

## Quickstart

```
maturin develop
python -c "import barter_python as bp; print(bp.shutdown_event().is_terminal())"
```

## Development

- Requires Python 3.9+
- Install maturin: `pip install maturin`
- Build: `maturin develop`
- Test: `cargo test -p barter-python`
