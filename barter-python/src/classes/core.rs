use chrono::{DateTime, Utc};
use pyo3::{prelude::*, pyclass::CompareOp};

use barter::{Sequence, Timed};

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

/// Wrapper around [`Sequence`] for Python exposure.
#[pyclass(module = "barter_python", name = "Sequence", unsendable)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PySequence {
    inner: Sequence,
}

impl PySequence {
    pub(crate) fn from_inner(inner: Sequence) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PySequence {
    /// Create a new [`Sequence`] with the provided starting value.
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: u64) -> Self {
        Self {
            inner: Sequence(value),
        }
    }

    /// Return the current sequence counter as an integer.
    #[getter]
    pub fn value(&self) -> u64 {
        self.inner.value()
    }

    /// Increment the sequence and return the previous value as a new wrapper.
    pub fn fetch_add(&mut self) -> Self {
        let previous = self.inner.fetch_add();
        Self { inner: previous }
    }

    /// Increment the sequence and return the new counter value.
    pub fn next_value(&mut self) -> u64 {
        let _ = self.inner.fetch_add();
        self.inner.value()
    }

    /// Convert the sequence to an integer for Python's `int()`.
    fn __int__(&self) -> u64 {
        self.inner.value()
    }

    /// Represent the sequence for debugging.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Sequence(value={})", self.inner.value()))
    }

    /// Support rich comparisons by comparing the underlying counter.
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            CompareOp::Lt => Ok(self.inner.value() < other.inner.value()),
            CompareOp::Le => Ok(self.inner.value() <= other.inner.value()),
            CompareOp::Gt => Ok(self.inner.value() > other.inner.value()),
            CompareOp::Ge => Ok(self.inner.value() >= other.inner.value()),
        }
    }
}

/// Convenience function returning a shutdown [`EngineEvent`].
#[pyfunction]
pub fn shutdown_event() -> crate::classes::engine::PyEngineEvent {
    crate::classes::engine::PyEngineEvent::shutdown()
}

/// Create a [`Timed`] floating point value.
#[pyfunction]
pub fn timed_f64(value: f64, time: DateTime<Utc>) -> PyTimedF64 {
    PyTimedF64::new(value, time)
}
