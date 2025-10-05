use std::str::FromStr;

use barter::system::config::InstrumentConfig;
use barter_execution::{
    balance::{AssetBalance as ExecutionAssetBalance, Balance as ExecutionBalance},
    error::{ApiError, KeyError, OrderError},
    map::{ExecutionInstrumentMap, generate_execution_instrument_map},
    order::{
        OrderEvent,
        id::{ClientOrderId, OrderId, StrategyId},
        state::{
            ActiveOrderState, CancelInFlight, Cancelled, InactiveOrderState, Open, OrderState,
        },
    },
    trade::{AssetFees as ExecutionAssetFees, Trade as ExecutionTrade, TradeId},
};
use barter_instrument::{
    Side,
    asset::{AssetIndex, QuoteAsset, name::AssetNameExchange},
    exchange::{ExchangeId, ExchangeIndex},
    index::{IndexedInstruments, error::IndexError},
    instrument::{Instrument, InstrumentIndex, name::InstrumentNameExchange},
};
use chrono::{DateTime, Utc};
use pyo3::{
    Bound, Py, PyAny, PyObject, PyResult, Python,
    exceptions::PyValueError,
    prelude::*,
    types::{PyModule, PyType},
};
use rust_decimal::Decimal;

use crate::{
    command::{PyOrderKey, parse_side},
    config::PySystemConfig,
    data::PyExchangeId,
    instrument::{PyAssetIndex, PyExchangeIndex, PyInstrumentIndex, PyQuoteAsset, PySide},
    summary::decimal_to_py,
};
use serde::Serialize;
use serde_json;

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

fn index_error_to_py(error: IndexError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn key_error_to_py(error: KeyError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

pub(crate) fn instrument_configs_from_py(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<InstrumentConfig>> {
    if let Ok(config) = value.extract::<Py<PySystemConfig>>() {
        let borrowed = config.borrow(py);
        let mut system = borrowed.clone_inner();
        return Ok(system.instruments.drain(..).collect());
    }

    let json_module = PyModule::import_bound(py, "json")?;
    let dumps = json_module.getattr("dumps")?;
    let serialized: String = dumps.call1((value,))?.extract()?;
    serde_json::from_str(&serialized).map_err(|err| PyValueError::new_err(err.to_string()))
}

type DefaultOrderEvent =
    OrderEvent<OrderState<AssetIndex, InstrumentIndex>, ExchangeIndex, InstrumentIndex>;

type DefaultOrderState = OrderState<AssetIndex, InstrumentIndex>;
type DefaultActiveOrderState = ActiveOrderState;
type DefaultInactiveOrderState = InactiveOrderState<AssetIndex, InstrumentIndex>;
type DefaultOpenState = Open;
type DefaultCancelInFlight = CancelInFlight;
type DefaultCancelledState = Cancelled;
type DefaultOrderError = OrderError<AssetIndex, InstrumentIndex>;
type DefaultApiError = ApiError<AssetIndex, InstrumentIndex>;

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

pub(crate) fn serialize_to_json<T>(value: &T) -> PyResult<String>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(|err| PyValueError::new_err(err.to_string()))
}

pub(crate) fn serialize_to_py_dict<T>(py: Python<'_>, value: &T) -> PyResult<PyObject>
where
    T: Serialize,
{
    let serialized = serialize_to_json(value)?;
    let json_module = PyModule::import_bound(py, "json")?;
    let loads = json_module.getattr("loads")?;
    let loaded = loads.call1((serialized.into_py(py),))?;
    Ok(loaded.into())
}

/// Wrapper around [`OrderEvent`] for Python exposure.
#[pyclass(module = "barter_python", name = "OrderEvent", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderEvent {
    inner: DefaultOrderEvent,
}

impl PyOrderEvent {
    fn state_inner(&self) -> DefaultOrderState {
        self.inner.state.clone()
    }

    fn state_kind_inner(&self) -> &'static str {
        match &self.inner.state {
            OrderState::Active(_) => "Active",
            OrderState::Inactive(_) => "Inactive",
        }
    }
}

#[pymethods]
impl PyOrderEvent {
    #[staticmethod]
    pub fn from_json(data: &str) -> PyResult<Self> {
        let inner = serde_json::from_str::<DefaultOrderEvent>(data)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self { inner })
    }

    #[staticmethod]
    pub fn from_dict(py: Python<'_>, value: PyObject) -> PyResult<Self> {
        let json_module = PyModule::import_bound(py, "json")?;
        let dumps = json_module.getattr("dumps")?;
        let serialized: String = dumps.call1((&value,))?.extract()?;
        Self::from_json(&serialized)
    }

    #[getter]
    pub fn key(&self) -> PyOrderKey {
        PyOrderKey::from_inner(self.inner.key.clone())
    }

    #[getter]
    pub fn state(&self) -> PyOrderState {
        PyOrderState::from_inner(self.state_inner())
    }

    pub fn is_active(&self) -> bool {
        matches!(self.inner.state, OrderState::Active(_))
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self.inner.state, OrderState::Inactive(_))
    }

    #[getter]
    pub fn state_kind(&self) -> &'static str {
        self.state_kind_inner()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|err| PyValueError::new_err(err.to_string()))
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let serialized = self.to_json()?;
        let json_module = PyModule::import_bound(py, "json")?;
        let loads = json_module.getattr("loads")?;
        let loaded = loads.call1((serialized.into_py(py),))?;
        Ok(loaded.into())
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("OrderEvent({json})"))
    }
}

/// Wrapper around [`OrderState`] for Python exposure.
#[pyclass(module = "barter_python", name = "OrderState", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderState {
    inner: DefaultOrderState,
}

impl PyOrderState {
    fn from_inner(inner: DefaultOrderState) -> Self {
        Self { inner }
    }

    fn variant_inner(&self) -> &'static str {
        match &self.inner {
            OrderState::Active(_) => "Active",
            OrderState::Inactive(_) => "Inactive",
        }
    }
}

#[pymethods]
impl PyOrderState {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant_inner()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.inner, OrderState::Active(_))
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self.inner, OrderState::Inactive(_))
    }

    pub fn active(&self) -> Option<PyActiveOrderState> {
        match &self.inner {
            OrderState::Active(state) => Some(PyActiveOrderState::from_inner(state.clone())),
            _ => None,
        }
    }

    pub fn inactive(&self) -> Option<PyInactiveOrderState> {
        match &self.inner {
            OrderState::Inactive(state) => Some(PyInactiveOrderState::from_inner(state.clone())),
            _ => None,
        }
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("OrderState({json})"))
    }
}

/// Wrapper around [`ActiveOrderState`] for Python exposure.
#[pyclass(module = "barter_python", name = "ActiveOrderState", unsendable)]
#[derive(Debug, Clone)]
pub struct PyActiveOrderState {
    inner: DefaultActiveOrderState,
}

impl PyActiveOrderState {
    fn from_inner(inner: DefaultActiveOrderState) -> Self {
        Self { inner }
    }

    fn variant_inner(&self) -> &'static str {
        match &self.inner {
            ActiveOrderState::OpenInFlight(_) => "OpenInFlight",
            ActiveOrderState::Open(_) => "Open",
            ActiveOrderState::CancelInFlight(_) => "CancelInFlight",
        }
    }
}

#[pymethods]
impl PyActiveOrderState {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant_inner()
    }

    pub fn is_open_in_flight(&self) -> bool {
        matches!(self.inner, ActiveOrderState::OpenInFlight(_))
    }

    pub fn is_open(&self) -> bool {
        matches!(self.inner, ActiveOrderState::Open(_))
    }

    pub fn is_cancel_in_flight(&self) -> bool {
        matches!(self.inner, ActiveOrderState::CancelInFlight(_))
    }

    pub fn open(&self) -> Option<PyOpenState> {
        match &self.inner {
            ActiveOrderState::Open(state) => Some(PyOpenState::from_inner(state.clone())),
            ActiveOrderState::CancelInFlight(state) => {
                state.order.clone().map(PyOpenState::from_inner)
            }
            _ => None,
        }
    }

    pub fn cancel_in_flight(&self) -> Option<PyCancelInFlightState> {
        match &self.inner {
            ActiveOrderState::CancelInFlight(state) => {
                Some(PyCancelInFlightState::from_inner(state.clone()))
            }
            _ => None,
        }
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("ActiveOrderState({json})"))
    }
}

/// Wrapper around [`InactiveOrderState`] for Python exposure.
#[pyclass(module = "barter_python", name = "InactiveOrderState", unsendable)]
#[derive(Debug, Clone)]
pub struct PyInactiveOrderState {
    inner: DefaultInactiveOrderState,
}

impl PyInactiveOrderState {
    fn from_inner(inner: DefaultInactiveOrderState) -> Self {
        Self { inner }
    }

    fn variant_inner(&self) -> &'static str {
        match &self.inner {
            InactiveOrderState::Cancelled(_) => "Cancelled",
            InactiveOrderState::FullyFilled => "FullyFilled",
            InactiveOrderState::OpenFailed(_) => "OpenFailed",
            InactiveOrderState::Expired => "Expired",
        }
    }
}

#[pymethods]
impl PyInactiveOrderState {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant_inner()
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.inner, InactiveOrderState::Cancelled(_))
    }

    pub fn is_fully_filled(&self) -> bool {
        matches!(self.inner, InactiveOrderState::FullyFilled)
    }

    pub fn is_expired(&self) -> bool {
        matches!(self.inner, InactiveOrderState::Expired)
    }

    pub fn is_open_failed(&self) -> bool {
        matches!(self.inner, InactiveOrderState::OpenFailed(_))
    }

    pub fn cancelled(&self) -> Option<PyCancelledState> {
        match &self.inner {
            InactiveOrderState::Cancelled(state) => {
                Some(PyCancelledState::from_inner(state.clone()))
            }
            _ => None,
        }
    }

    pub fn open_failed(&self) -> Option<PyOrderError> {
        match &self.inner {
            InactiveOrderState::OpenFailed(error) => Some(PyOrderError::from_inner(error.clone())),
            _ => None,
        }
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("InactiveOrderState({json})"))
    }
}

/// Wrapper around [`Open`] order metadata for Python exposure.
#[pyclass(module = "barter_python", name = "Open", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOpenState {
    inner: DefaultOpenState,
}

impl PyOpenState {
    fn from_inner(inner: DefaultOpenState) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyOpenState {
    #[getter]
    pub fn order_id(&self) -> PyOrderId {
        PyOrderId::from_inner(self.inner.id.clone())
    }

    #[getter]
    pub fn time_exchange(&self) -> DateTime<Utc> {
        self.inner.time_exchange
    }

    #[getter]
    pub fn filled_quantity(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.filled_quantity)
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("Open({json})"))
    }
}

/// Wrapper around [`CancelInFlight`] for Python exposure.
#[pyclass(module = "barter_python", name = "CancelInFlight", unsendable)]
#[derive(Debug, Clone)]
pub struct PyCancelInFlightState {
    inner: DefaultCancelInFlight,
}

impl PyCancelInFlightState {
    fn from_inner(inner: DefaultCancelInFlight) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyCancelInFlightState {
    pub fn has_order(&self) -> bool {
        self.inner.order.is_some()
    }

    pub fn order(&self) -> Option<PyOpenState> {
        self.inner.order.clone().map(PyOpenState::from_inner)
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("CancelInFlight({json})"))
    }
}

/// Wrapper around [`Cancelled`] order metadata for Python exposure.
#[pyclass(module = "barter_python", name = "Cancelled", unsendable)]
#[derive(Debug, Clone)]
pub struct PyCancelledState {
    inner: DefaultCancelledState,
}

impl PyCancelledState {
    fn from_inner(inner: DefaultCancelledState) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyCancelledState {
    #[getter]
    pub fn order_id(&self) -> PyOrderId {
        PyOrderId::from_inner(self.inner.id.clone())
    }

    #[getter]
    pub fn time_exchange(&self) -> DateTime<Utc> {
        self.inner.time_exchange
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("Cancelled({json})"))
    }
}

/// Wrapper around [`OrderError`] for Python exposure.
#[pyclass(module = "barter_python", name = "OrderError", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderError {
    inner: DefaultOrderError,
}

impl PyOrderError {
    fn from_inner(inner: DefaultOrderError) -> Self {
        Self { inner }
    }

    fn variant_inner(&self) -> &'static str {
        match &self.inner {
            OrderError::Connectivity(_) => "Connectivity",
            OrderError::Rejected(_) => "Rejected",
        }
    }

    fn api_error_inner(&self) -> Option<DefaultApiError> {
        match &self.inner {
            OrderError::Rejected(error) => Some(error.clone()),
            _ => None,
        }
    }
}

#[pymethods]
impl PyOrderError {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant_inner()
    }

    pub fn is_connectivity(&self) -> bool {
        matches!(self.inner, OrderError::Connectivity(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self.inner, OrderError::Rejected(_))
    }

    pub fn message(&self) -> String {
        self.inner.to_string()
    }

    pub fn api_error(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        match self.api_error_inner() {
            Some(error) => serialize_to_py_dict(py, &error).map(Some),
            None => Ok(None),
        }
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        serialize_to_py_dict(py, &self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("OrderError({json})"))
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

#[pyclass(module = "barter_python", name = "ExecutionInstrumentMap", unsendable)]
#[derive(Debug, Clone)]
pub struct PyExecutionInstrumentMap {
    inner: ExecutionInstrumentMap,
}

impl PyExecutionInstrumentMap {
    fn from_configs(exchange: ExchangeId, configs: Vec<InstrumentConfig>) -> PyResult<Self> {
        let instruments = configs
            .into_iter()
            .map(Instrument::from)
            .collect::<Vec<_>>();
        let indexed = IndexedInstruments::new(instruments);
        let inner =
            generate_execution_instrument_map(&indexed, exchange).map_err(index_error_to_py)?;
        Ok(Self { inner })
    }

    fn collect_asset_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .inner
            .exchange_assets()
            .map(|name| name.name().as_str().to_string())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    fn collect_instrument_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .inner
            .exchange_instruments()
            .map(|name| name.name().as_str().to_string())
            .collect();
        names.sort();
        names.dedup();
        names
    }
}

#[pymethods]
impl PyExecutionInstrumentMap {
    #[classmethod]
    #[pyo3(signature = (exchange, config))]
    pub fn from_system_config(
        _cls: &Bound<'_, PyType>,
        exchange: &PyExchangeId,
        config: &PySystemConfig,
    ) -> PyResult<Self> {
        let mut system = config.clone_inner();
        Self::from_configs(exchange.as_inner(), system.instruments.drain(..).collect())
    }

    #[classmethod]
    #[pyo3(signature = (exchange, definitions))]
    pub fn from_definitions(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        exchange: &PyExchangeId,
        definitions: PyObject,
    ) -> PyResult<Self> {
        let value = definitions.bind(py);
        let configs = instrument_configs_from_py(py, &value)?;
        Self::from_configs(exchange.as_inner(), configs)
    }

    #[getter]
    pub fn exchange_id(&self) -> PyExchangeId {
        PyExchangeId::from_inner(self.inner.exchange.value)
    }

    #[getter]
    pub fn exchange_index(&self) -> PyExchangeIndex {
        PyExchangeIndex::from_inner(self.inner.exchange.key)
    }

    pub fn asset_names(&self) -> Vec<String> {
        self.collect_asset_names()
    }

    pub fn instrument_names(&self) -> Vec<String> {
        self.collect_instrument_names()
    }

    #[pyo3(signature = (name))]
    pub fn asset_index(&self, name: &str) -> PyResult<PyAssetIndex> {
        let name_exchange = AssetNameExchange::new(name);
        let index = self
            .inner
            .find_asset_index(&name_exchange)
            .map_err(index_error_to_py)?;
        Ok(PyAssetIndex::from_inner(index))
    }

    #[pyo3(signature = (index))]
    pub fn asset_name(&self, index: &PyAssetIndex) -> PyResult<String> {
        self.inner
            .find_asset_name_exchange(index.inner())
            .map(|name| name.name().as_str().to_string())
            .map_err(key_error_to_py)
    }

    #[pyo3(signature = (name))]
    pub fn instrument_index(&self, name: &str) -> PyResult<PyInstrumentIndex> {
        let name_exchange = InstrumentNameExchange::new(name.to_string());
        let index = self
            .inner
            .find_instrument_index(&name_exchange)
            .map_err(index_error_to_py)?;
        Ok(PyInstrumentIndex::from_inner(index))
    }

    #[pyo3(signature = (index))]
    pub fn instrument_name(&self, index: &PyInstrumentIndex) -> PyResult<String> {
        self.inner
            .find_instrument_name_exchange(index.inner())
            .map(|name| name.name().as_str().to_string())
            .map_err(key_error_to_py)
    }

    fn __repr__(&self) -> String {
        format!(
            "ExecutionInstrumentMap(exchange={}, assets={}, instruments={})",
            self.inner.exchange.value.as_str(),
            self.inner.assets.len(),
            self.inner.instruments.len()
        )
    }
}
