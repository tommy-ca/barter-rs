#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]
#![allow(unsafe_op_in_unsafe_fn)]

//! Python bindings for the Barter trading engine.

mod config;
mod system;

use barter::{EngineEvent, Timed};
use barter_integration::Terminal;
use chrono::{DateTime, Utc};
use config::PySystemConfig;
use pyo3::{Bound, prelude::*, types::PyModule};
use system::run_historic_backtest;

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
    m.add_function(wrap_pyfunction!(shutdown_event, m)?)?;
    m.add_function(wrap_pyfunction!(timed_f64, m)?)?;
    m.add_function(wrap_pyfunction!(run_historic_backtest, m)?)?;

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
    fn timed_f64_surfaces_value_and_time() {
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let timed = PyTimedF64 {
            inner: Timed { value: 42.5, time },
        };

        assert_eq!(timed.value(), 42.5);
        assert_eq!(timed.time(), time);
    }
}
