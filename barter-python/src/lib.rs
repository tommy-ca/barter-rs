#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]
#![allow(unsafe_op_in_unsafe_fn)]

//! Python bindings for the Barter trading engine.

mod command;
mod config;
mod logging;
mod summary;
mod system;

use barter::engine::{command::Command, state::trading::TradingState};
use barter::execution::AccountStreamEvent;
use barter::{EngineEvent, Timed};
use barter_execution::{
    AccountEvent, AccountEventKind,
    balance::{AssetBalance, Balance},
};
use barter_instrument::{asset::AssetIndex, exchange::ExchangeIndex};
use barter_integration::{Terminal, snapshot::Snapshot};
use chrono::{DateTime, Utc};
use command::{
    PyInstrumentFilter, PyOrderKey, PyOrderRequestCancel, PyOrderRequestOpen, clone_filter,
    collect_cancel_requests, collect_open_requests, parse_decimal,
};
use config::PySystemConfig;
use logging::init_tracing;
use pyo3::{Bound, exceptions::PyValueError, prelude::*, types::PyModule};
use summary::{
    PyAssetTearSheet, PyBalance, PyDrawdown, PyInstrumentTearSheet, PyMeanDrawdown,
    PyMetricWithInterval, PyTradingSummary,
};
use system::{PySystemHandle, run_historic_backtest, start_system};

/// Wrapper around [`Timed`] with a floating point value for Python exposure.
#[pyclass(module = "barter_python", name = "TimedF64", unsendable)]
#[derive(Debug, Clone)]
pub struct PyTimedF64 {
    inner: Timed<f64>,
}

#[pymethods]
impl PyTimedF64 {
    /// Create a new [`Timed`] value.
    #[new]
    #[pyo3(signature = (value, time))]
    pub fn new(value: f64, time: DateTime<Utc>) -> Self {
        Self {
            inner: Timed { value, time },
        }
    }

    /// Value component of the timed pair.
    #[getter]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    /// Timestamp component of the timed pair.
    #[getter]
    pub fn time(&self) -> DateTime<Utc> {
        self.inner.time
    }

    /// Return a formatted representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "TimedF64(value={}, time={})",
            self.inner.value, self.inner.time
        ))
    }
}

/// Wrapper around [`EngineEvent`] value for Python.
#[pyclass(module = "barter_python", name = "EngineEvent", unsendable)]
#[derive(Debug, Clone)]
pub struct PyEngineEvent {
    inner: EngineEvent,
}

#[pymethods]
impl PyEngineEvent {
    /// Construct a shutdown [`EngineEvent`].
    #[staticmethod]
    pub fn shutdown() -> Self {
        Self {
            inner: EngineEvent::shutdown(),
        }
    }

    /// Construct an [`EngineEvent`] from a JSON string.
    #[staticmethod]
    pub fn from_json(data: &str) -> PyResult<Self> {
        let inner =
            serde_json::from_str(data).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self { inner })
    }

    /// Construct an [`EngineEvent`] from a Python dictionary-like object.
    #[staticmethod]
    pub fn from_dict(py: Python<'_>, value: PyObject) -> PyResult<Self> {
        let json_module = PyModule::import_bound(py, "json")?;
        let dumps = json_module.getattr("dumps")?;
        let serialized: String = dumps.call1((value,))?.extract()?;

        Self::from_json(&serialized)
    }

    /// Construct a trading state update event.
    #[staticmethod]
    pub fn trading_state(enabled: bool) -> Self {
        let state = if enabled {
            TradingState::Enabled
        } else {
            TradingState::Disabled
        };

        Self {
            inner: EngineEvent::TradingStateUpdate(state),
        }
    }

    /// Construct an [`EngineEvent::Command`] to send open order requests.
    #[staticmethod]
    pub fn send_open_requests(
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestOpen>>,
    ) -> PyResult<Self> {
        let command = Command::SendOpenRequests(collect_open_requests(py, requests)?);
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to send cancel order requests.
    #[staticmethod]
    pub fn send_cancel_requests(
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestCancel>>,
    ) -> PyResult<Self> {
        let command = Command::SendCancelRequests(collect_cancel_requests(py, requests)?);
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to close positions using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn close_positions(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::ClosePositions(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Command`] to cancel orders using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn cancel_orders(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::CancelOrders(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Account`] with a balance snapshot update.
    #[staticmethod]
    #[pyo3(signature = (exchange, asset, total, free, time_exchange))]
    pub fn account_balance_snapshot(
        exchange: usize,
        asset: usize,
        total: f64,
        free: f64,
        time_exchange: DateTime<Utc>,
    ) -> PyResult<Self> {
        if free > total {
            return Err(PyValueError::new_err(
                "free balance cannot exceed total balance",
            ));
        }

        let total_decimal = parse_decimal(total, "total balance")?;
        let free_decimal = parse_decimal(free, "free balance")?;

        let balance = Balance::new(total_decimal, free_decimal);
        let asset_balance = AssetBalance::new(AssetIndex(asset), balance, time_exchange);
        let snapshot = Snapshot::new(asset_balance);

        let event = AccountEvent::new(
            ExchangeIndex(exchange),
            AccountEventKind::BalanceSnapshot(snapshot),
        );

        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Item(event)),
        })
    }

    /// Check if the underlying event is terminal.
    pub fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
    }

    /// Serialize the [`EngineEvent`] to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Convert the [`EngineEvent`] into a Python dictionary via JSON round-trip.
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json = self.to_json()?;
        let json_module = PyModule::import_bound(py, "json")?;
        let loads = json_module.getattr("loads")?;
        Ok(loads.call1((json,))?.into_py(py))
    }

    /// Debug style string representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("EngineEvent({:?})", self.inner))
    }
}

/// Convenience function returning a shutdown [`EngineEvent`].
#[pyfunction]
pub fn shutdown_event() -> PyEngineEvent {
    PyEngineEvent::shutdown()
}

/// Create a [`Timed`] floating point value.
#[pyfunction]
pub fn timed_f64(value: f64, time: DateTime<Utc>) -> PyTimedF64 {
    PyTimedF64::new(value, time)
}

/// Python module definition entry point.
#[pymodule]
pub fn barter_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySystemConfig>()?;
    m.add_class::<PyEngineEvent>()?;
    m.add_class::<PyTimedF64>()?;
    m.add_class::<PySystemHandle>()?;
    m.add_class::<PyOrderKey>()?;
    m.add_class::<PyOrderRequestOpen>()?;
    m.add_class::<PyOrderRequestCancel>()?;
    m.add_class::<PyInstrumentFilter>()?;
    m.add_class::<PyTradingSummary>()?;
    m.add_class::<PyInstrumentTearSheet>()?;
    m.add_class::<PyAssetTearSheet>()?;
    m.add_class::<PyMetricWithInterval>()?;
    m.add_class::<PyDrawdown>()?;
    m.add_class::<PyMeanDrawdown>()?;
    m.add_class::<PyBalance>()?;
    m.add_function(wrap_pyfunction!(init_tracing, m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_event, m)?)?;
    m.add_function(wrap_pyfunction!(timed_f64, m)?)?;
    m.add_function(wrap_pyfunction!(run_historic_backtest, m)?)?;
    m.add_function(wrap_pyfunction!(start_system, m)?)?;

    // Expose module level constants.
    let shutdown = PyEngineEvent::shutdown();
    m.add("SHUTDOWN_EVENT", shutdown.into_py(py))?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pyo3::{Python, types::PyDict};

    #[test]
    fn engine_event_shutdown_is_terminal() {
        let event = PyEngineEvent {
            inner: EngineEvent::shutdown(),
        };
        assert!(event.inner.is_terminal());
    }

    #[test]
    fn engine_event_trading_state_constructor() {
        let enabled = PyEngineEvent::trading_state(true);
        match enabled.inner {
            EngineEvent::TradingStateUpdate(state) => assert_eq!(state, TradingState::Enabled),
            other => panic!("unexpected event variant: {other:?}"),
        }

        let disabled = PyEngineEvent::trading_state(false);
        match disabled.inner {
            EngineEvent::TradingStateUpdate(state) => assert_eq!(state, TradingState::Disabled),
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn timed_f64_surfaces_value_and_time() {
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let timed = PyTimedF64 {
            inner: Timed { value: 42.5, time },
        };

        assert_eq!(timed.value(), 42.5);
        assert_eq!(timed.time(), time);
    }

    #[test]
    fn engine_event_json_roundtrip() {
        let event = PyEngineEvent::trading_state(true);
        let json = event.to_json().unwrap();
        let restored = PyEngineEvent::from_json(&json).unwrap();
        assert_eq!(restored.inner, event.inner);
    }

    #[test]
    fn engine_event_dict_roundtrip() {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("Shutdown", PyDict::new_bound(py)).unwrap();

            let event = PyEngineEvent::from_dict(py, dict.into_py(py)).unwrap();
            assert!(event.inner.is_terminal());

            let object = event.to_dict(py).unwrap();
            let json_module = PyModule::import_bound(py, "json").unwrap();
            let dumps = json_module.getattr("dumps").unwrap();
            let dumped: String = dumps
                .call1((object.clone_ref(py),))
                .unwrap()
                .extract()
                .unwrap();
            assert!(dumped.contains("Shutdown"));
        });
    }
}
