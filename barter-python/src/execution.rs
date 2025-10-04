use barter_execution::order::id::{ClientOrderId, OrderId, StrategyId};
use pyo3::{Bound, Py, PyAny, PyResult, exceptions::PyValueError, prelude::*, types::PyType};

fn ensure_non_empty(value: &str, label: &str) -> PyResult<()> {
    if value.trim().is_empty() {
        Err(PyValueError::new_err(format!("{label} must not be empty")))
    } else {
        Ok(())
    }
}

/// Wrapper around [`ClientOrderId`] for Python exposure.
#[pyclass(module = "barter_python", name = "ClientOrderId", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyClientOrderId {
    inner: ClientOrderId,
}

impl PyClientOrderId {
    pub(crate) fn inner(&self) -> ClientOrderId {
        self.inner.clone()
    }

    pub(crate) fn from_inner(inner: ClientOrderId) -> Self {
        Self { inner }
    }

    fn format_repr(&self, label: &str) -> String {
        format!("{label}('{}')", self.inner)
    }
}

#[pymethods]
impl PyClientOrderId {
    /// Create a new [`ClientOrderId`].
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: &str) -> PyResult<Self> {
        ensure_non_empty(value, "client order id")?;
        Ok(Self {
            inner: ClientOrderId::new(value),
        })
    }

    #[classmethod]
    #[pyo3(signature = (value))]
    pub fn new(_cls: &Bound<'_, PyType>, value: &str) -> PyResult<Self> {
        Self::__new__(value)
    }

    /// Generate a random [`ClientOrderId`].
    #[staticmethod]
    pub fn random() -> Self {
        Self {
            inner: ClientOrderId::random(),
        }
    }

    /// Access the underlying string value.
    #[getter]
    pub fn value(&self) -> String {
        self.inner.to_string()
    }

    /// String representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Debug representation.
    fn __repr__(&self) -> String {
        self.format_repr("ClientOrderId")
    }
}

/// Wrapper around [`OrderId`] for Python exposure.
#[pyclass(module = "barter_python", name = "OrderId", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyOrderId {
    inner: OrderId,
}

impl PyOrderId {
    pub(crate) fn inner(&self) -> OrderId {
        self.inner.clone()
    }

    pub(crate) fn from_inner(inner: OrderId) -> Self {
        Self { inner }
    }

    fn format_repr(&self, label: &str) -> String {
        format!("{label}('{}')", self.inner)
    }
}

#[pymethods]
impl PyOrderId {
    /// Create a new [`OrderId`].
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: &str) -> PyResult<Self> {
        ensure_non_empty(value, "order id")?;
        Ok(Self {
            inner: OrderId::new(value),
        })
    }

    #[classmethod]
    #[pyo3(signature = (value))]
    pub fn new(_cls: &Bound<'_, PyType>, value: &str) -> PyResult<Self> {
        Self::__new__(value)
    }

    /// Access the underlying string value.
    #[getter]
    pub fn value(&self) -> String {
        self.inner.to_string()
    }

    /// String representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Debug representation.
    fn __repr__(&self) -> String {
        self.format_repr("OrderId")
    }
}

/// Wrapper around [`StrategyId`] for Python exposure.
#[pyclass(module = "barter_python", name = "StrategyId", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyStrategyId {
    inner: StrategyId,
}

impl PyStrategyId {
    pub(crate) fn inner(&self) -> StrategyId {
        self.inner.clone()
    }

    pub(crate) fn from_inner(inner: StrategyId) -> Self {
        Self { inner }
    }

    fn format_repr(&self, label: &str) -> String {
        format!("{label}('{}')", self.inner)
    }
}

#[pymethods]
impl PyStrategyId {
    /// Create a new [`StrategyId`].
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: &str) -> PyResult<Self> {
        ensure_non_empty(value, "strategy id")?;
        Ok(Self {
            inner: StrategyId::new(value),
        })
    }

    #[classmethod]
    #[pyo3(signature = (value))]
    pub fn new(_cls: &Bound<'_, PyType>, value: &str) -> PyResult<Self> {
        Self::__new__(value)
    }

    /// Strategy identifier representing an unknown strategy.
    #[staticmethod]
    pub fn unknown() -> Self {
        Self {
            inner: StrategyId::unknown(),
        }
    }

    /// Access the underlying string value.
    #[getter]
    pub fn value(&self) -> String {
        self.inner.to_string()
    }

    /// String representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Debug representation.
    fn __repr__(&self) -> String {
        self.format_repr("StrategyId")
    }
}

fn extract_string(value: &Bound<'_, PyAny>, label: &str) -> PyResult<String> {
    let extracted: String = value.extract()?;
    ensure_non_empty(&extracted, label)?;
    Ok(extracted)
}

fn extract_strategy_id(value: &Bound<'_, PyAny>) -> Option<StrategyId> {
    value
        .extract::<Py<PyStrategyId>>()
        .ok()
        .map(|owned| owned.borrow(value.py()).inner())
}

fn extract_client_order_id(value: &Bound<'_, PyAny>) -> Option<ClientOrderId> {
    value
        .extract::<Py<PyClientOrderId>>()
        .ok()
        .map(|owned| owned.borrow(value.py()).inner())
}

pub(crate) fn coerce_strategy_id(value: &Bound<'_, PyAny>) -> PyResult<StrategyId> {
    if let Some(strategy) = extract_strategy_id(value) {
        return Ok(strategy);
    }

    let text = extract_string(value, "strategy id")?;
    Ok(StrategyId::new(text))
}

pub(crate) fn coerce_client_order_id(value: Option<&Bound<'_, PyAny>>) -> PyResult<ClientOrderId> {
    match value {
        None => Ok(ClientOrderId::random()),
        Some(bound) => {
            if let Some(cid) = extract_client_order_id(bound) {
                return Ok(cid);
            }

            let text = extract_string(bound, "client order id")?;
            Ok(ClientOrderId::new(text))
        }
    }
}
