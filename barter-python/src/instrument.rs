use barter_instrument::{asset::{Asset, AssetIndex}, Side};
use pyo3::prelude::*;

/// Wrapper around [`Asset`] for Python exposure.
#[pyclass(module = "barter_python", name = "Asset", unsendable)]
#[derive(Debug, Clone)]
pub struct PyAsset {
    inner: Asset,
}

#[pymethods]
impl PyAsset {
    /// Create a new [`Asset`].
    #[new]
    #[pyo3(signature = (name_internal, name_exchange))]
    fn new(name_internal: &str, name_exchange: &str) -> Self {
        Self {
            inner: Asset::new(name_internal, name_exchange),
        }
    }

    /// Create a new [`Asset`] from exchange name only.
    #[staticmethod]
    fn from_exchange_name(name_exchange: &str) -> Self {
        Self {
            inner: Asset::new_from_exchange(name_exchange),
        }
    }

    /// Get the internal name.
    #[getter]
    fn name_internal(&self) -> &str {
        self.inner.name_internal.name()
    }

    /// Get the exchange name.
    #[getter]
    fn name_exchange(&self) -> &str {
        self.inner.name_exchange.name()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        format!(
            "Asset(name_internal='{}', name_exchange='{}')",
            self.name_internal(),
            self.name_exchange()
        )
    }

    /// Return the debug representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Asset(name_internal='{}', name_exchange='{}')", self.name_internal(), self.name_exchange()))
    }
}

/// Wrapper around [`Side`] for Python exposure.
#[pyclass(module = "barter_python", name = "Side", eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySide {
    inner: Side,
}

#[pymethods]
impl PySide {
    /// Buy side.
    #[classattr]
    const BUY: Self = Self {
        inner: Side::Buy,
    };

    /// Sell side.
    #[classattr]
    const SELL: Self = Self {
        inner: Side::Sell,
    };

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("Side.{:?}", self.inner)
    }
}

/// Wrapper around [`AssetIndex`] for Python exposure.
#[pyclass(module = "barter_python", name = "AssetIndex", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyAssetIndex {
    inner: AssetIndex,
}

#[pymethods]
impl PyAssetIndex {
    /// Create a new [`AssetIndex`].
    #[new]
    fn new(index: usize) -> Self {
        Self {
            inner: AssetIndex(index),
        }
    }

    /// Get the index value.
    #[getter]
    fn index(&self) -> usize {
        self.inner.index()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
}