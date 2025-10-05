use std::str::FromStr;

use barter::system::config::InstrumentConfig;
use barter_instrument::{
    Side,
    asset::{
        Asset, AssetIndex, QuoteAsset,
        name::{AssetNameExchange, AssetNameInternal},
    },
    exchange::{ExchangeId, ExchangeIndex},
    index::{IndexedInstruments, error::IndexError},
    instrument::{
        InstrumentIndex,
        name::{InstrumentNameExchange, InstrumentNameInternal},
        spec::{
            InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity,
            OrderQuantityUnits,
        },
    },
};
use pyo3::{Bound, PyAny, PyResult, Python, exceptions::PyValueError, prelude::*, types::PyType};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::{
    config::PySystemConfig,
    data::PyExchangeId,
    execution::{instrument_configs_from_py, serialize_to_py_dict},
    summary::decimal_to_py,
};

fn exchange_id_from_str(value: &str) -> PyResult<ExchangeId> {
    let quoted = format!("\"{value}\"");
    serde_json::from_str::<ExchangeId>(&quoted).map_err(|_| {
        PyValueError::new_err(format!(
            "unknown exchange id '{value}'; provide a valid ExchangeId",
        ))
    })
}

fn coerce_exchange_id(value: &Bound<'_, PyAny>) -> PyResult<ExchangeId> {
    if let Ok(py_exchange) = value.extract::<PyExchangeId>() {
        Ok(py_exchange.as_inner())
    } else if let Ok(text) = value.extract::<&str>() {
        exchange_id_from_str(text)
    } else if let Ok(attr) = value.getattr("value") {
        let text = attr.extract::<&str>()?;
        exchange_id_from_str(text)
    } else {
        Err(PyValueError::new_err(
            "exchange must be barter_python.ExchangeId or string",
        ))
    }
}

/// Wrapper around [`AssetNameInternal`] for Python exposure.
#[pyclass(module = "barter_python", name = "AssetNameInternal", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyAssetNameInternal {
    inner: AssetNameInternal,
}

#[pymethods]
impl PyAssetNameInternal {
    /// Create a new [`AssetNameInternal`], normalising to lowercase.
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: &str) -> Self {
        Self {
            inner: AssetNameInternal::new(name),
        }
    }

    /// Underlying lowercase identifier.
    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name().as_str()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.name().as_str().to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("AssetNameInternal('{}')", self.name())
    }
}

/// Wrapper around [`AssetNameExchange`] for Python exposure.
#[pyclass(module = "barter_python", name = "AssetNameExchange", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyAssetNameExchange {
    inner: AssetNameExchange,
}

#[pymethods]
impl PyAssetNameExchange {
    /// Create a new [`AssetNameExchange`].
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: &str) -> Self {
        Self {
            inner: AssetNameExchange::new(name),
        }
    }

    /// Exchange-specific identifier.
    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name().as_str()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.name().as_str().to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("AssetNameExchange('{}')", self.name())
    }
}

/// Wrapper around [`InstrumentNameExchange`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentNameExchange",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentNameExchange {
    inner: InstrumentNameExchange,
}

#[pymethods]
impl PyInstrumentNameExchange {
    /// Create a new [`InstrumentNameExchange`].
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: &str) -> Self {
        Self {
            inner: InstrumentNameExchange::new(name),
        }
    }

    /// Exchange-level identifier.
    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name().as_str()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.name().as_str().to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("InstrumentNameExchange('{}')", self.name())
    }
}

/// Wrapper around [`InstrumentNameInternal`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentNameInternal",
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyInstrumentNameInternal {
    inner: InstrumentNameInternal,
}

#[pymethods]
impl PyInstrumentNameInternal {
    /// Create a new [`InstrumentNameInternal`], normalising to lowercase.
    #[new]
    #[pyo3(signature = (name))]
    pub fn new(name: &str) -> Self {
        Self {
            inner: InstrumentNameInternal::new(name),
        }
    }

    /// Construct from an exchange and exchange-level identifier.
    #[classmethod]
    #[pyo3(signature = (exchange, name_exchange))]
    pub fn new_from_exchange(
        _cls: &Bound<'_, PyType>,
        exchange: &Bound<'_, PyAny>,
        name_exchange: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let exchange_id = coerce_exchange_id(exchange)?;
        if let Ok(wrapper) = name_exchange.extract::<PyInstrumentNameExchange>() {
            let inner = InstrumentNameInternal::new_from_exchange(
                exchange_id,
                InstrumentNameExchange::new(wrapper.name()),
            );
            Ok(Self { inner })
        } else {
            let name = name_exchange.extract::<&str>()?;
            let inner = InstrumentNameInternal::new_from_exchange(
                exchange_id,
                InstrumentNameExchange::new(name),
            );
            Ok(Self { inner })
        }
    }

    /// Internal identifier.
    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name().as_str()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.name().as_str().to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("InstrumentNameInternal('{}')", self.name())
    }
}

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

    pub(crate) fn from_inner(inner: ExchangeIndex) -> Self {
        Self { inner }
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

    pub(crate) fn from_inner(inner: InstrumentIndex) -> Self {
        Self { inner }
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

    pub(crate) fn from_inner(inner: AssetIndex) -> Self {
        Self { inner }
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

fn index_error_to_py(error: IndexError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

#[pyclass(module = "barter_python", name = "IndexedInstruments", unsendable)]
#[derive(Debug, Clone)]
pub struct PyIndexedInstruments {
    inner: IndexedInstruments,
}

impl PyIndexedInstruments {
    fn from_configs(configs: Vec<InstrumentConfig>) -> Self {
        Self {
            inner: IndexedInstruments::new(configs),
        }
    }
}

#[pymethods]
impl PyIndexedInstruments {
    #[classmethod]
    #[pyo3(signature = (config))]
    pub fn from_system_config(_cls: &Bound<'_, PyType>, config: &PySystemConfig) -> Self {
        let mut config = config.clone_inner();
        Self::from_configs(std::mem::take(&mut config.instruments))
    }

    #[classmethod]
    #[pyo3(signature = (definitions))]
    pub fn from_definitions(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        definitions: PyObject,
    ) -> PyResult<Self> {
        let definitions = definitions.bind(py);
        let configs = instrument_configs_from_py(py, &definitions)?;
        Ok(Self::from_configs(configs))
    }

    pub fn __len__(&self) -> usize {
        self.inner.instruments().len()
    }

    #[getter]
    pub fn exchanges_len(&self) -> usize {
        self.inner.exchanges().len()
    }

    #[getter]
    pub fn assets_len(&self) -> usize {
        self.inner.assets().len()
    }

    #[pyo3(signature = (exchange))]
    pub fn exchange_index(&self, exchange: &PyExchangeId) -> PyResult<PyExchangeIndex> {
        let index = self
            .inner
            .find_exchange_index(exchange.as_inner())
            .map_err(index_error_to_py)?;
        Ok(PyExchangeIndex::from_inner(index))
    }

    #[pyo3(signature = (index))]
    pub fn exchange_id(&self, index: &PyExchangeIndex) -> PyResult<PyExchangeId> {
        let exchange = self
            .inner
            .find_exchange(index.inner())
            .map_err(index_error_to_py)?;
        Ok(PyExchangeId::from_inner(exchange))
    }

    #[pyo3(signature = (exchange, asset_name_internal))]
    pub fn asset_index(
        &self,
        exchange: &PyExchangeId,
        asset_name_internal: &str,
    ) -> PyResult<PyAssetIndex> {
        let asset_name = AssetNameInternal::new(asset_name_internal);
        let index = self
            .inner
            .find_asset_index(exchange.as_inner(), &asset_name)
            .map_err(index_error_to_py)?;
        Ok(PyAssetIndex::from_inner(index))
    }

    #[pyo3(signature = (index))]
    pub fn asset(&self, index: &PyAssetIndex) -> PyResult<PyAsset> {
        let exchange_asset = self
            .inner
            .find_asset(index.inner())
            .map_err(index_error_to_py)?;
        Ok(PyAsset::from_inner(exchange_asset.asset.clone()))
    }

    #[pyo3(signature = (exchange, name_exchange))]
    pub fn instrument_index_from_exchange_name(
        &self,
        exchange: &PyExchangeId,
        name_exchange: &str,
    ) -> PyResult<PyInstrumentIndex> {
        let exchange_id = exchange.as_inner();
        let name_exchange = InstrumentNameExchange::new(name_exchange.to_string());
        let maybe_index = self.inner.instruments().iter().find_map(|keyed| {
            (keyed.value.exchange.value == exchange_id
                && keyed.value.name_exchange == name_exchange)
                .then_some(keyed.key)
        });

        maybe_index
            .map(PyInstrumentIndex::from_inner)
            .ok_or_else(|| {
                index_error_to_py(IndexError::InstrumentIndex(format!(
                    "instrument {} not found for exchange {}",
                    name_exchange.as_ref(),
                    exchange_id.as_str()
                )))
            })
    }

    #[pyo3(signature = (index))]
    pub fn instrument(&self, py: Python<'_>, index: &PyInstrumentIndex) -> PyResult<PyObject> {
        let instrument = self
            .inner
            .find_instrument(index.inner())
            .map_err(index_error_to_py)?;
        serialize_to_py_dict(py, instrument)
    }

    fn __repr__(&self) -> String {
        format!(
            "IndexedInstruments(exchanges={}, assets={}, instruments={})",
            self.inner.exchanges().len(),
            self.inner.assets().len(),
            self.inner.instruments().len()
        )
    }
}
