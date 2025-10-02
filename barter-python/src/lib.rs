#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]
#![allow(unsafe_op_in_unsafe_fn)]

//! Python bindings for the Barter trading engine.

mod command;
mod config;
mod system;

use barter::engine::state::trading::TradingState;
use barter::{EngineEvent, Timed};
use barter_integration::Terminal;
use chrono::{DateTime, Utc};
use command::{
    PyInstrumentFilter, PyOrderKey, PyOrderRequestCancel, PyOrderRequestOpen, clone_filter,
    collect_cancel_requests, collect_open_requests,
};
use config::PySystemConfig;
use pyo3::{Bound, prelude::*, types::PyModule};
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
        let command = barter::Command::SendOpenRequests(collect_open_requests(py, requests)?);
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
        let command = barter::Command::SendCancelRequests(collect_cancel_requests(py, requests)?);
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to close positions using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn close_positions(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = barter::Command::ClosePositions(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Command`] to cancel orders using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn cancel_orders(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = barter::Command::CancelOrders(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Check if the underlying event is terminal.
    pub fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
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
    m.add_function(wrap_pyfunction!(shutdown_event, m)?)?;
    m.add_function(wrap_pyfunction!(timed_f64, m)?)?;
    m.add_function(wrap_pyfunction!(run_historic_backtest, m)?)?;
    m.add_function(wrap_pyfunction!(start_system, m)?)?;

    // Expose module level constants.
    let shutdown = PyEngineEvent::shutdown();
    m.add("SHUTDOWN_EVENT", shutdown.into_py(py))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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
}
