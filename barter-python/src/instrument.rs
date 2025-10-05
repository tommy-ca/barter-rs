use std::str::FromStr;

use barter_instrument::{
    Side,
    asset::{Asset, AssetIndex, QuoteAsset},
    exchange::ExchangeIndex,
    instrument::{
        InstrumentIndex,
        spec::{
            InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity,
            OrderQuantityUnits,
        },
    },
};
use pyo3::{Bound, PyAny, PyResult, Python, exceptions::PyValueError, prelude::*};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::summary::decimal_to_py;

/// Wrapper around [`Asset`] for Python exposure.
#[pyclass(module = "barter_python", name = "Asset", unsendable)]
#[derive(Debug, Clone)]
pub struct PyAsset {
    inner: Asset,
}

impl PyAsset {
    pub(crate) fn from_inner(inner: Asset) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> Asset {
        self.inner.clone()
    }
}

fn parse_decimal(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Decimal> {
    if let Ok(float) = value.extract::<f64>() {
        return Decimal::from_f64(float).ok_or_else(|| {
            PyValueError::new_err(format!("{label} must be a finite numeric value"))
        });
    }

    let text = value.str()?;
    Decimal::from_str(text.to_str()?)
        .map_err(|err| PyValueError::new_err(format!("{label} must be a valid decimal: {err}")))
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
        Ok(format!(
            "Asset(name_internal='{}', name_exchange='{}')",
            self.name_internal(),
            self.name_exchange()
        ))
    }
}

/// Wrapper around [`QuoteAsset`] for Python exposure.
#[pyclass(module = "barter_python", name = "QuoteAsset", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyQuoteAsset {
    inner: QuoteAsset,
}

#[pymethods]
impl PyQuoteAsset {
    /// Create a new [`QuoteAsset`].
    #[new]
    pub(crate) fn new() -> Self {
        Self { inner: QuoteAsset }
    }

    /// Return the string representation.
    fn __str__(&self) -> &'static str {
        "QuoteAsset"
    }

    /// Return the debug representation.
    fn __repr__(&self) -> &'static str {
        "QuoteAsset()"
    }
}

/// Wrapper around [`ExchangeIndex`] for Python exposure.
#[pyclass(module = "barter_python", name = "ExchangeIndex", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExchangeIndex {
    inner: ExchangeIndex,
}

impl PyExchangeIndex {
    pub(crate) fn inner(&self) -> ExchangeIndex {
        self.inner
    }
}

#[pymethods]
impl PyExchangeIndex {
    /// Create a new [`ExchangeIndex`].
    #[new]
    fn new(index: usize) -> Self {
        Self {
            inner: ExchangeIndex(index),
        }
    }

    /// Get the index value.
    #[getter]
    fn index(&self) -> usize {
        self.inner.index()
    }

    /// Return the integer representation.
    fn __int__(&self) -> usize {
        self.index()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("ExchangeIndex({})", self.index())
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
    const BUY: Self = Self { inner: Side::Buy };

    /// Sell side.
    #[classattr]
    const SELL: Self = Self { inner: Side::Sell };

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("Side.{:?}", self.inner)
    }
}

impl PySide {
    pub(crate) fn inner(&self) -> Side {
        self.inner
    }

    pub(crate) fn from_side(side: Side) -> Self {
        Self { inner: side }
    }
}

/// Wrapper around [`InstrumentIndex`] for Python exposure.
#[pyclass(module = "barter_python", name = "InstrumentIndex", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentIndex {
    inner: InstrumentIndex,
}

impl PyInstrumentIndex {
    pub(crate) fn inner(&self) -> InstrumentIndex {
        self.inner
    }
}

#[pymethods]
impl PyInstrumentIndex {
    /// Create a new [`InstrumentIndex`].
    #[new]
    fn new(index: usize) -> Self {
        Self {
            inner: InstrumentIndex(index),
        }
    }

    /// Get the index value.
    #[getter]
    fn index(&self) -> usize {
        self.inner.index()
    }

    /// Return the integer representation.
    fn __int__(&self) -> usize {
        self.index()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("InstrumentIndex({})", self.index())
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

impl PyAssetIndex {
    pub(crate) fn inner(&self) -> AssetIndex {
        self.inner
    }
}

/// Wrapper around [`OrderQuantityUnits`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "OrderQuantityUnits",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyOrderQuantityUnits {
    inner: OrderQuantityUnits<Asset>,
}

impl PyOrderQuantityUnits {
    pub(crate) fn inner(&self) -> OrderQuantityUnits<Asset> {
        self.inner.clone()
    }
}

#[pymethods]
impl PyOrderQuantityUnits {
    /// Construct asset-based quantity units.
    #[staticmethod]
    #[pyo3(signature = (asset))]
    pub fn asset(asset: &PyAsset) -> Self {
        Self {
            inner: OrderQuantityUnits::Asset(asset.inner()),
        }
    }

    /// Construct contract-based quantity units.
    #[staticmethod]
    pub fn contract() -> Self {
        Self {
            inner: OrderQuantityUnits::Contract,
        }
    }

    /// Construct quote-based quantity units.
    #[staticmethod]
    pub fn quote() -> Self {
        Self {
            inner: OrderQuantityUnits::Quote,
        }
    }

    /// Variant kind string ("asset", "contract", or "quote").
    #[getter]
    pub fn kind(&self) -> &'static str {
        match self.inner {
            OrderQuantityUnits::Asset(_) => "asset",
            OrderQuantityUnits::Contract => "contract",
            OrderQuantityUnits::Quote => "quote",
        }
    }

    /// Underlying asset when the variant is `asset`.
    #[getter]
    pub fn asset_value(&self) -> Option<PyAsset> {
        match &self.inner {
            OrderQuantityUnits::Asset(asset) => Some(PyAsset::from_inner(asset.clone())),
            _ => None,
        }
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            OrderQuantityUnits::Asset(asset) => format!(
                "OrderQuantityUnits(kind='asset', asset='{}')",
                asset.name_exchange.name()
            ),
            OrderQuantityUnits::Contract => "OrderQuantityUnits(kind='contract')".to_string(),
            OrderQuantityUnits::Quote => "OrderQuantityUnits(kind='quote')".to_string(),
        }
    }
}

/// Wrapper around [`InstrumentSpecPrice`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentSpecPrice",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentSpecPrice {
    inner: InstrumentSpecPrice,
}

impl PyInstrumentSpecPrice {
    pub(crate) fn inner(&self) -> InstrumentSpecPrice {
        self.inner
    }
}

#[pymethods]
impl PyInstrumentSpecPrice {
    #[new]
    #[pyo3(signature = (min, tick_size))]
    pub fn new(min: PyObject, tick_size: PyObject) -> PyResult<Self> {
        Python::with_gil(|py| {
            let min_bound = min.bind(py);
            let tick_bound = tick_size.bind(py);

            let min_value = parse_decimal(&min_bound, "min")?;
            if min_value.is_sign_negative() {
                return Err(PyValueError::new_err("min must be non-negative"));
            }

            let tick_value = parse_decimal(&tick_bound, "tick_size")?;
            if !tick_value.is_sign_positive() {
                return Err(PyValueError::new_err(
                    "tick_size must be a positive numeric value",
                ));
            }

            Ok(Self {
                inner: InstrumentSpecPrice::new(min_value, tick_value),
            })
        })
    }

    #[getter]
    pub fn min(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.min)
    }

    #[getter]
    pub fn tick_size(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.tick_size)
    }

    fn __repr__(&self) -> String {
        format!(
            "InstrumentSpecPrice(min={}, tick_size={})",
            self.inner.min, self.inner.tick_size
        )
    }
}

/// Wrapper around [`InstrumentSpecQuantity`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentSpecQuantity",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentSpecQuantity {
    inner: InstrumentSpecQuantity<Asset>,
}

impl PyInstrumentSpecQuantity {
    pub(crate) fn inner(&self) -> InstrumentSpecQuantity<Asset> {
        self.inner.clone()
    }
}

#[pymethods]
impl PyInstrumentSpecQuantity {
    #[new]
    #[pyo3(signature = (unit, min, increment))]
    pub fn new(unit: &PyOrderQuantityUnits, min: PyObject, increment: PyObject) -> PyResult<Self> {
        Python::with_gil(|py| {
            let min_bound = min.bind(py);
            let increment_bound = increment.bind(py);

            let min_value = parse_decimal(&min_bound, "min")?;
            if min_value.is_sign_negative() {
                return Err(PyValueError::new_err("min must be non-negative"));
            }

            let increment_value = parse_decimal(&increment_bound, "increment")?;
            if !increment_value.is_sign_positive() {
                return Err(PyValueError::new_err(
                    "increment must be a positive numeric value",
                ));
            }

            Ok(Self {
                inner: InstrumentSpecQuantity::new(unit.inner(), min_value, increment_value),
            })
        })
    }

    #[getter]
    pub fn unit(&self) -> PyOrderQuantityUnits {
        PyOrderQuantityUnits {
            inner: self.inner.unit.clone(),
        }
    }

    #[getter]
    pub fn min(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.min)
    }

    #[getter]
    pub fn increment(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.increment)
    }

    fn __repr__(&self) -> String {
        let unit = match &self.inner.unit {
            OrderQuantityUnits::Asset(asset) => format!(
                "OrderQuantityUnits(kind='asset', asset='{}')",
                asset.name_exchange.name()
            ),
            OrderQuantityUnits::Contract => "OrderQuantityUnits(kind='contract')".to_string(),
            OrderQuantityUnits::Quote => "OrderQuantityUnits(kind='quote')".to_string(),
        };

        format!(
            "InstrumentSpecQuantity(unit={}, min={}, increment={})",
            unit, self.inner.min, self.inner.increment
        )
    }
}

/// Wrapper around [`InstrumentSpecNotional`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentSpecNotional",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentSpecNotional {
    inner: InstrumentSpecNotional,
}

impl PyInstrumentSpecNotional {
    pub(crate) fn inner(&self) -> InstrumentSpecNotional {
        self.inner
    }
}

#[pymethods]
impl PyInstrumentSpecNotional {
    #[new]
    #[pyo3(signature = (min))]
    pub fn new(min: PyObject) -> PyResult<Self> {
        Python::with_gil(|py| {
            let bound = min.bind(py);
            let value = parse_decimal(&bound, "min")?;
            if !value.is_sign_positive() {
                return Err(PyValueError::new_err(
                    "min must be a positive numeric value",
                ));
            }

            Ok(Self {
                inner: InstrumentSpecNotional::new(value),
            })
        })
    }

    #[getter]
    pub fn min(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.min)
    }

    fn __repr__(&self) -> String {
        format!("InstrumentSpecNotional(min={})", self.inner.min)
    }
}

/// Wrapper around [`InstrumentSpec`] for Python exposure.
#[pyclass(module = "barter_python", name = "InstrumentSpec", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentSpec {
    inner: InstrumentSpec<Asset>,
}

#[pymethods]
impl PyInstrumentSpec {
    #[new]
    #[pyo3(signature = (price, quantity, notional))]
    pub fn new(
        price: &PyInstrumentSpecPrice,
        quantity: &PyInstrumentSpecQuantity,
        notional: &PyInstrumentSpecNotional,
    ) -> Self {
        Self {
            inner: InstrumentSpec::new(price.inner(), quantity.inner(), notional.inner()),
        }
    }

    #[getter]
    pub fn price(&self) -> PyInstrumentSpecPrice {
        PyInstrumentSpecPrice {
            inner: self.inner.price,
        }
    }

    #[getter]
    pub fn quantity(&self) -> PyInstrumentSpecQuantity {
        PyInstrumentSpecQuantity {
            inner: self.inner.quantity.clone(),
        }
    }

    #[getter]
    pub fn notional(&self) -> PyInstrumentSpecNotional {
        PyInstrumentSpecNotional {
            inner: self.inner.notional,
        }
    }

    fn __repr__(&self) -> String {
        let price = PyInstrumentSpecPrice {
            inner: self.inner.price,
        };
        let quantity = PyInstrumentSpecQuantity {
            inner: self.inner.quantity.clone(),
        };
        let notional = PyInstrumentSpecNotional {
            inner: self.inner.notional,
        };

        format!(
            "InstrumentSpec(price={}, quantity={}, notional={})",
            price.__repr__(),
            quantity.__repr__(),
            notional.__repr__(),
        )
    }
}
