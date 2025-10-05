use std::str::FromStr;

use barter_execution::{
    balance::{AssetBalance as ExecutionAssetBalance, Balance as ExecutionBalance},
    order::id::{ClientOrderId, OrderId, StrategyId},
    trade::{AssetFees as ExecutionAssetFees, Trade as ExecutionTrade, TradeId},
};
use barter_instrument::{
    Side,
    asset::{AssetIndex, QuoteAsset},
    instrument::InstrumentIndex,
};
use chrono::{DateTime, Utc};
use pyo3::{
    Bound, Py, PyAny, PyObject, PyResult, Python, exceptions::PyValueError, prelude::*,
    types::PyType,
};
use rust_decimal::Decimal;

use crate::{
    command::parse_side,
    instrument::{PyAssetIndex, PyInstrumentIndex, PyQuoteAsset, PySide},
    summary::decimal_to_py,
};

fn ensure_non_empty(value: &str, label: &str) -> PyResult<()> {
    if value.trim().is_empty() {
        Err(PyValueError::new_err(format!("{label} must not be empty")))
    } else {
        Ok(())
    }
}

fn extract_decimal(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Decimal> {
    let binding = value.str()?;
    Decimal::from_str(binding.to_str()?)
        .map_err(|err| PyValueError::new_err(format!("{label} must be a valid decimal: {err}")))
}

fn extract_side(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Side> {
    if let Ok(rust_side) = value.extract::<PySide>() {
        return Ok(rust_side.inner());
    }

    if let Ok(text) = value.extract::<&str>() {
        return parse_side(text);
    }

    if let Ok(attr) = value.getattr("value")
        && let Ok(text) = attr.extract::<&str>()
    {
        return parse_side(text);
    }

    Err(PyValueError::new_err(format!(
        "{label} must be 'buy', 'sell', or a Side value"
    )))
}

fn extract_instrument_index(value: &Bound<'_, PyAny>, label: &str) -> PyResult<InstrumentIndex> {
    if let Ok(index) = value.extract::<usize>() {
        return Ok(InstrumentIndex(index));
    }

    if let Ok(py_index) = value.extract::<Py<PyInstrumentIndex>>() {
        let py = value.py();
        let borrowed = py_index.borrow(py);
        return Ok(borrowed.inner());
    }

    Err(PyValueError::new_err(format!(
        "{label} must be an integer or InstrumentIndex"
    )))
}

fn extract_asset_index(value: &Bound<'_, PyAny>, label: &str) -> PyResult<AssetIndex> {
    if let Ok(index) = value.extract::<usize>() {
        return Ok(AssetIndex(index));
    }

    if let Ok(wrapper) = value.extract::<Py<PyAssetIndex>>() {
        let borrowed = wrapper.borrow(value.py());
        return Ok(borrowed.inner());
    }

    Err(PyValueError::new_err(format!(
        "{label} must be an integer or AssetIndex",
    )))
}

/// Wrapper around [`ExecutionBalance`] for Python exposure.
#[pyclass(module = "barter_python", name = "Balance", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExecutionBalance {
    inner: ExecutionBalance,
}

impl PyExecutionBalance {
    pub(crate) fn inner(&self) -> ExecutionBalance {
        self.inner
    }

    pub(crate) fn from_inner(inner: ExecutionBalance) -> Self {
        Self { inner }
    }

    fn from_bounds(total: &Bound<'_, PyAny>, free: &Bound<'_, PyAny>) -> PyResult<Self> {
        let total_decimal = extract_decimal(total, "total")?;
        let free_decimal = extract_decimal(free, "free")?;

        if total_decimal.is_sign_negative() {
            return Err(PyValueError::new_err(
                "total must be a non-negative numeric value",
            ));
        }

        if free_decimal.is_sign_negative() {
            return Err(PyValueError::new_err(
                "free must be a non-negative numeric value",
            ));
        }

        if free_decimal > total_decimal {
            return Err(PyValueError::new_err(
                "free balance cannot exceed total balance",
            ));
        }

        Ok(Self {
            inner: ExecutionBalance::new(total_decimal, free_decimal),
        })
    }
}

#[pyfunction]
#[pyo3(signature = (total, free))]
pub fn balance_new(total: PyObject, free: PyObject) -> PyResult<PyExecutionBalance> {
    Python::with_gil(|py| {
        let total_bound = total.bind(py);
        let free_bound = free.bind(py);
        PyExecutionBalance::from_bounds(&total_bound, &free_bound)
    })
}

#[pyfunction]
#[pyo3(signature = (asset, balance, time_exchange))]
pub fn asset_balance_new(
    asset: PyObject,
    balance: PyObject,
    time_exchange: DateTime<Utc>,
) -> PyResult<PyExecutionAssetBalance> {
    Python::with_gil(|py| {
        let asset_bound = asset.bind(py);
        let balance_bound = balance.bind(py);

        let asset_index = extract_asset_index(&asset_bound, "asset")?;
        let py_balance = balance_bound
            .extract::<Py<PyExecutionBalance>>()
            .map_err(|_| PyValueError::new_err("balance must be a Balance value"))?;
        let rust_balance = py_balance.borrow(py).inner();

        Ok(PyExecutionAssetBalance {
            inner: ExecutionAssetBalance::new(asset_index, rust_balance, time_exchange),
        })
    })
}

#[pymethods]
impl PyExecutionBalance {
    #[new]
    #[pyo3(signature = (total, free))]
    pub fn new(total: PyObject, free: PyObject) -> PyResult<Self> {
        Python::with_gil(|py| {
            let total_bound = total.bind(py);
            let free_bound = free.bind(py);
            Self::from_bounds(&total_bound, &free_bound)
        })
    }

    #[getter]
    pub fn total(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.total)
    }

    #[getter]
    pub fn free(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.free)
    }

    pub fn used(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.used())
    }

    fn __str__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let total = decimal_to_py(py, self.inner.total)?;
            let free = decimal_to_py(py, self.inner.free)?;
            let total_repr: String = total.bind(py).str()?.extract()?;
            let free_repr: String = free.bind(py).str()?.extract()?;
            Ok(format!("Balance(total={total_repr}, free={free_repr})"))
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let total = decimal_to_py(py, self.inner.total)?;
            let free = decimal_to_py(py, self.inner.free)?;
            let total_repr: String = total.bind(py).repr()?.extract()?;
            let free_repr: String = free.bind(py).repr()?.extract()?;
            Ok(format!("Balance(total={total_repr}, free={free_repr})"))
        })
    }
}

/// Wrapper around [`ExecutionAssetBalance`] for Python exposure.
#[pyclass(module = "barter_python", name = "AssetBalance", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExecutionAssetBalance {
    inner: ExecutionAssetBalance<AssetIndex>,
}

impl PyExecutionAssetBalance {
    pub(crate) fn from_inner(inner: ExecutionAssetBalance<AssetIndex>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyExecutionAssetBalance {
    #[new]
    #[pyo3(signature = (asset, balance, time_exchange))]
    pub fn new(asset: PyObject, balance: PyObject, time_exchange: DateTime<Utc>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let asset_bound = asset.bind(py);
            let balance_bound = balance.bind(py);

            let asset_index = extract_asset_index(&asset_bound, "asset")?;

            let py_balance = balance_bound
                .extract::<Py<PyExecutionBalance>>()
                .map_err(|_| PyValueError::new_err("balance must be a Balance value"))?;

            let rust_balance = py_balance.borrow(py).inner();

            Ok(Self {
                inner: ExecutionAssetBalance::new(asset_index, rust_balance, time_exchange),
            })
        })
    }

    #[getter]
    pub fn asset(&self) -> usize {
        self.inner.asset.index()
    }

    #[getter]
    pub fn balance(&self) -> PyExecutionBalance {
        PyExecutionBalance::from_inner(self.inner.balance)
    }

    #[getter]
    pub fn time_exchange(&self) -> DateTime<Utc> {
        self.inner.time_exchange
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let balance = PyExecutionBalance::from_inner(self.inner.balance);
            let balance_py = Py::new(py, balance)?;
            let balance_repr: String = balance_py.bind(py).repr()?.extract()?;
            Ok(format!(
                "AssetBalance(asset={}, balance={}, time_exchange={})",
                self.asset(),
                balance_repr,
                self.inner.time_exchange,
            ))
        })
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PyAssetFeesInner {
    Quote(ExecutionAssetFees<QuoteAsset>),
    Named(ExecutionAssetFees<String>),
}

impl PyAssetFeesInner {
    fn fees(&self) -> Decimal {
        match self {
            Self::Quote(inner) => inner.fees,
            Self::Named(inner) => inner.fees,
        }
    }
}

/// Wrapper around [`TradeId`] for Python exposure.
#[pyclass(module = "barter_python", name = "TradeId", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyTradeId {
    inner: TradeId,
}

impl PyTradeId {
    pub(crate) fn inner(&self) -> TradeId {
        self.inner.clone()
    }

    pub(crate) fn from_inner(inner: TradeId) -> Self {
        Self { inner }
    }

    fn format_repr(&self) -> String {
        format!("TradeId('{}')", self.as_str())
    }

    fn as_str(&self) -> &str {
        self.inner.0.as_str()
    }
}

#[pymethods]
impl PyTradeId {
    /// Create a new [`TradeId`].
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: &str) -> PyResult<Self> {
        ensure_non_empty(value, "trade id")?;
        Ok(Self {
            inner: TradeId::new(value),
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
        self.as_str().to_string()
    }

    /// String representation.
    fn __str__(&self) -> String {
        self.as_str().to_string()
    }

    /// Debug representation.
    fn __repr__(&self) -> String {
        self.format_repr()
    }
}

/// Wrapper around [`AssetFees`] for Python exposure.
#[pyclass(module = "barter_python", name = "AssetFees", eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyAssetFees {
    inner: PyAssetFeesInner,
}

impl PyAssetFees {
    fn from_quote(fees: Decimal) -> Self {
        Self {
            inner: PyAssetFeesInner::Quote(ExecutionAssetFees::quote_fees(fees)),
        }
    }

    fn as_quote(&self) -> PyResult<ExecutionAssetFees<QuoteAsset>> {
        match &self.inner {
            PyAssetFeesInner::Quote(inner) => Ok(inner.clone()),
            PyAssetFeesInner::Named(_) => Err(PyValueError::new_err(
                "trade fees must be denominated in the quote asset",
            )),
        }
    }

    fn asset_display(&self) -> String {
        match &self.inner {
            PyAssetFeesInner::Quote(_) => "QuoteAsset".to_string(),
            PyAssetFeesInner::Named(inner) => inner.asset.clone(),
        }
    }

    fn asset_debug(&self) -> String {
        match &self.inner {
            PyAssetFeesInner::Quote(_) => "QuoteAsset()".to_string(),
            PyAssetFeesInner::Named(inner) => format!("'{}'", inner.asset),
        }
    }
}

#[pymethods]
impl PyAssetFees {
    #[new]
    #[pyo3(signature = (asset, fees))]
    pub fn __new__(asset: &Bound<'_, PyAny>, fees: &Bound<'_, PyAny>) -> PyResult<Self> {
        let fees_decimal = extract_decimal(fees, "fees")?;
        let py = asset.py();

        if let Ok(quote_obj) = asset.extract::<Py<PyQuoteAsset>>() {
            let _borrowed = quote_obj.borrow(py);
            return Ok(Self::from_quote(fees_decimal));
        }

        if let Ok(asset_text) = asset.extract::<&str>() {
            return Ok(Self {
                inner: PyAssetFeesInner::Named(ExecutionAssetFees {
                    asset: asset_text.to_string(),
                    fees: fees_decimal,
                }),
            });
        }

        Err(PyValueError::new_err(
            "asset must be a string or QuoteAsset",
        ))
    }

    #[staticmethod]
    #[pyo3(signature = (fees))]
    pub fn quote_fees(fees: &Bound<'_, PyAny>) -> PyResult<Self> {
        let fees_decimal = extract_decimal(fees, "fees")?;
        Ok(Self::from_quote(fees_decimal))
    }

    #[getter]
    pub fn asset(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner {
            PyAssetFeesInner::Quote(_) => {
                let quote = PyQuoteAsset::new();
                Py::new(py, quote).map(|value| value.into_py(py))
            }
            PyAssetFeesInner::Named(inner) => Ok(inner.asset.clone().into_py(py)),
        }
    }

    #[getter]
    pub fn fees(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.fees())
    }

    fn __str__(&self) -> String {
        format!(
            "AssetFees(asset={}, fees={})",
            self.asset_display(),
            self.inner.fees()
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "AssetFees(asset={}, fees={})",
            self.asset_debug(),
            self.inner.fees()
        )
    }
}

/// Wrapper around [`Trade`] for Python exposure.
#[pyclass(module = "barter_python", name = "Trade", unsendable, eq, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyTrade {
    inner: ExecutionTrade<QuoteAsset, InstrumentIndex>,
}

#[pymethods]
impl PyTrade {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (id, order_id, instrument, strategy, time_exchange, side, price, quantity, fees))]
    pub fn __new__(
        id: &PyTradeId,
        order_id: &PyOrderId,
        instrument: &Bound<'_, PyAny>,
        strategy: &PyStrategyId,
        time_exchange: DateTime<Utc>,
        side: &Bound<'_, PyAny>,
        price: &Bound<'_, PyAny>,
        quantity: &Bound<'_, PyAny>,
        fees: &PyAssetFees,
    ) -> PyResult<Self> {
        let instrument_index = extract_instrument_index(instrument, "instrument")?;
        let side = extract_side(side, "side")?;
        let price_decimal = extract_decimal(price, "price")?;
        let quantity_decimal = extract_decimal(quantity, "quantity")?;
        let fees_inner = fees.as_quote()?;

        let trade = ExecutionTrade {
            id: id.inner(),
            order_id: order_id.inner(),
            instrument: instrument_index,
            strategy: strategy.inner(),
            time_exchange,
            side,
            price: price_decimal,
            quantity: quantity_decimal,
            fees: fees_inner,
        };

        Ok(Self { inner: trade })
    }

    #[getter]
    pub fn id(&self, py: Python<'_>) -> PyResult<PyObject> {
        Py::new(py, PyTradeId::from_inner(self.inner.id.clone())).map(|value| value.into_py(py))
    }

    #[getter]
    pub fn order_id(&self, py: Python<'_>) -> PyResult<PyObject> {
        Py::new(py, PyOrderId::from_inner(self.inner.order_id.clone()))
            .map(|value| value.into_py(py))
    }

    #[getter]
    pub fn instrument(&self) -> usize {
        self.inner.instrument.index()
    }

    #[getter]
    pub fn strategy(&self, py: Python<'_>) -> PyResult<PyObject> {
        Py::new(py, PyStrategyId::from_inner(self.inner.strategy.clone()))
            .map(|value| value.into_py(py))
    }

    #[getter]
    pub fn time_exchange(&self) -> DateTime<Utc> {
        self.inner.time_exchange
    }

    #[getter]
    pub fn side(&self, py: Python<'_>) -> PyResult<PyObject> {
        Py::new(py, PySide::from_side(self.inner.side)).map(|value| value.into_py(py))
    }

    #[getter]
    pub fn price(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.price)
    }

    #[getter]
    pub fn quantity(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.quantity)
    }

    #[getter]
    pub fn fees(&self, py: Python<'_>) -> PyResult<PyObject> {
        let wrapper = PyAssetFees {
            inner: PyAssetFeesInner::Quote(self.inner.fees.clone()),
        };
        Py::new(py, wrapper).map(|value| value.into_py(py))
    }

    pub fn value_quote(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.value_quote())
    }

    fn __str__(&self) -> String {
        format!(
            "Trade(instrument={}, side={}, price={}, quantity={}, time={})",
            self.inner.instrument.index(),
            self.inner.side,
            self.inner.price,
            self.inner.quantity,
            self.inner.time_exchange,
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "Trade(id={:?}, order_id={:?}, instrument={}, strategy={:?}, time_exchange={}, side={:?}, price={}, quantity={}, fees={})",
            self.inner.id,
            self.inner.order_id,
            self.inner.instrument.index(),
            self.inner.strategy,
            self.inner.time_exchange,
            self.inner.side,
            self.inner.price,
            self.inner.quantity,
            self.inner.fees.fees,
        )
    }
}
