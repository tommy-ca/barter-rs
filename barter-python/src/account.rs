use crate::{
    command::{PyOrderSnapshot, parse_decimal},
    execution::{PyExecutionAssetBalance, PyOrderResponseCancel, PyTrade, serialize_to_json},
    instrument::PyExchangeIndex,
    integration::PySnapshot,
};
use barter_execution::{
    AccountEvent as ExecutionAccountEvent, AccountEventKind as ExecutionAccountEventKind,
    AccountSnapshot, InstrumentAccountSnapshot,
    balance::{AssetBalance, Balance},
};
use barter_instrument::{asset::AssetIndex, exchange::ExchangeIndex, instrument::InstrumentIndex};
use barter_integration::snapshot::Snapshot;
use chrono::{DateTime, Utc};
use pyo3::{
    Bound, Py, PyAny, PyObject, PyResult, Python, basic::CompareOp,
    exceptions::PyNotImplementedError, exceptions::PyValueError, prelude::*, types::PyType,
};
use serde_json;
use std::{
    collections::{HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};
/// Wrapper around [`InstrumentAccountSnapshot`] for Python exposure.
#[pyclass(
    module = "barter_python",
    name = "InstrumentAccountSnapshot",
    unsendable
)]
#[derive(Debug, Clone)]
pub struct PyInstrumentAccountSnapshot {
    pub(crate) inner: InstrumentAccountSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex>,
}

impl PyInstrumentAccountSnapshot {
    pub(crate) fn clone_inner(
        &self,
    ) -> InstrumentAccountSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex> {
        self.inner.clone()
    }
}

#[pyclass(module = "barter_python", name = "AccountEventKind", unsendable)]
#[derive(Debug, Clone)]
pub struct PyAccountEventKind {
    inner: ExecutionAccountEventKind<ExchangeIndex, AssetIndex, InstrumentIndex>,
}

#[pyclass(module = "barter_python", name = "AccountEvent", unsendable)]
#[derive(Debug, Clone)]
pub struct PyAccountEvent {
    inner: ExecutionAccountEvent<ExchangeIndex, AssetIndex, InstrumentIndex>,
}

impl PyAccountEventKind {
    fn from_inner(
        inner: ExecutionAccountEventKind<ExchangeIndex, AssetIndex, InstrumentIndex>,
    ) -> Self {
        Self { inner }
    }

    fn clone_inner(&self) -> ExecutionAccountEventKind<ExchangeIndex, AssetIndex, InstrumentIndex> {
        self.inner.clone()
    }

    fn variant_str(&self) -> &'static str {
        match &self.inner {
            ExecutionAccountEventKind::Snapshot(_) => "snapshot",
            ExecutionAccountEventKind::BalanceSnapshot(_) => "balance_snapshot",
            ExecutionAccountEventKind::OrderSnapshot(_) => "order_snapshot",
            ExecutionAccountEventKind::OrderCancelled(_) => "order_cancelled",
            ExecutionAccountEventKind::Trade(_) => "trade",
        }
    }

    fn value_object(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner {
            ExecutionAccountEventKind::Snapshot(snapshot) => {
                let wrapper = PyAccountSnapshot::from_inner(snapshot.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            }
            ExecutionAccountEventKind::BalanceSnapshot(balance_snapshot) => {
                let balance = balance_snapshot.value().clone();
                let balance_wrapper = PyExecutionAssetBalance::from_inner(balance);
                let balance_py = Py::new(py, balance_wrapper)?;
                let snapshot = Snapshot::new(balance_py.into_py(py));
                Py::new(py, PySnapshot::from_inner(snapshot)).map(|value| value.into_py(py))
            }
            ExecutionAccountEventKind::OrderSnapshot(order_snapshot) => {
                let order = order_snapshot.value().clone();
                let order_wrapper = PyOrderSnapshot::from_inner(order);
                let order_py = Py::new(py, order_wrapper)?;
                let snapshot = Snapshot::new(order_py.into_py(py));
                Py::new(py, PySnapshot::from_inner(snapshot)).map(|value| value.into_py(py))
            }
            ExecutionAccountEventKind::OrderCancelled(response) => {
                let wrapper = PyOrderResponseCancel::from_inner(response.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            }
            ExecutionAccountEventKind::Trade(trade) => {
                let wrapper = PyTrade::from_inner(trade.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            }
        }
    }

    fn hash_repr(&self) -> PyResult<isize> {
        let json = serialize_to_json(&self.inner)?;
        Ok(hash_string(&json))
    }
}

impl PyAccountEvent {
    pub(crate) fn from_inner(
        inner: ExecutionAccountEvent<ExchangeIndex, AssetIndex, InstrumentIndex>,
    ) -> Self {
        Self { inner }
    }

    fn hash_repr(&self) -> PyResult<isize> {
        let json = serialize_to_json(&self.inner)?;
        Ok(hash_string(&json))
    }
}

#[pymethods]
impl PyAccountEventKind {
    #[classmethod]
    pub fn snapshot(_cls: &Bound<'_, PyType>, snapshot: &PyAccountSnapshot) -> Self {
        Self {
            inner: ExecutionAccountEventKind::Snapshot(snapshot.clone_inner()),
        }
    }

    #[classmethod]
    pub fn balance_snapshot(_cls: &Bound<'_, PyType>, balance: &PyExecutionAssetBalance) -> Self {
        Self {
            inner: ExecutionAccountEventKind::BalanceSnapshot(Snapshot::new(balance.inner.clone())),
        }
    }

    #[classmethod]
    pub fn order_snapshot(_cls: &Bound<'_, PyType>, order: &PyOrderSnapshot) -> Self {
        Self {
            inner: ExecutionAccountEventKind::OrderSnapshot(Snapshot::new(order.clone_inner())),
        }
    }

    #[classmethod]
    pub fn order_cancelled(_cls: &Bound<'_, PyType>, response: &PyOrderResponseCancel) -> Self {
        Self {
            inner: ExecutionAccountEventKind::OrderCancelled(response.clone_inner()),
        }
    }

    #[classmethod]
    pub fn trade(_cls: &Bound<'_, PyType>, trade: &PyTrade) -> Self {
        Self {
            inner: ExecutionAccountEventKind::Trade(trade.clone_inner()),
        }
    }

    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant_str()
    }

    #[getter]
    pub fn value(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.value_object(py)
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let value_obj = self.value(py)?;
            let value_repr: String = value_obj.bind(py).repr()?.extract()?;
            Ok(format!(
                "AccountEventKind(variant='{}', value={value_repr})",
                self.variant_str()
            ))
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(PyNotImplementedError::new_err(
                "ordering comparisons are not supported for AccountEventKind",
            )),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        self.hash_repr()
    }
}

#[pymethods]
impl PyAccountEvent {
    #[classmethod]
    #[pyo3(signature = (exchange, kind))]
    pub fn new(_cls: &Bound<'_, PyType>, exchange: usize, kind: &PyAccountEventKind) -> Self {
        Self {
            inner: ExecutionAccountEvent::new(ExchangeIndex(exchange), kind.clone_inner()),
        }
    }

    #[getter]
    pub fn exchange(&self) -> usize {
        self.inner.exchange.index()
    }

    #[getter]
    pub fn exchange_index(&self) -> PyExchangeIndex {
        PyExchangeIndex::from_inner(self.inner.exchange)
    }

    #[getter]
    pub fn kind(&self) -> PyAccountEventKind {
        PyAccountEventKind::from_inner(self.inner.kind.clone())
    }

    pub fn to_json(&self) -> PyResult<String> {
        serialize_to_json(&self.inner)
    }

    #[classmethod]
    pub fn from_json(_cls: &Bound<'_, PyType>, data: &str) -> PyResult<Self> {
        let inner = serde_json::from_str::<
            ExecutionAccountEvent<ExchangeIndex, AssetIndex, InstrumentIndex>,
        >(data)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> PyResult<String> {
        let json = self.to_json()?;
        Ok(format!("AccountEvent({json})"))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            _ => Err(PyNotImplementedError::new_err(
                "ordering comparisons are not supported for AccountEvent",
            )),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        self.hash_repr()
    }
}

fn hash_string(value: &str) -> isize {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as isize
}

#[pymethods]
impl PyInstrumentAccountSnapshot {
    #[new]
    #[pyo3(signature = (instrument, orders))]
    pub fn __new__(instrument: usize, orders: Vec<Py<PyOrderSnapshot>>) -> PyResult<Self> {
        let instrument_index = InstrumentIndex(instrument);

        let converted: Vec<_> = Python::with_gil(|py| {
            orders
                .into_iter()
                .map(|order| {
                    let borrowed = order.borrow(py);
                    borrowed.clone_inner()
                })
                .collect()
        });

        for order in &converted {
            if order.key.instrument != instrument_index {
                return Err(PyValueError::new_err(format!(
                    "order instrument {} does not match snapshot instrument {}",
                    order.key.instrument.index(),
                    instrument_index.index(),
                )));
            }
        }

        Ok(Self {
            inner: InstrumentAccountSnapshot::new(instrument_index, converted),
        })
    }

    #[getter]
    pub fn instrument(&self) -> usize {
        self.inner.instrument.index()
    }

    pub fn orders(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.inner
            .orders
            .iter()
            .map(|order| {
                let snapshot = PyOrderSnapshot::from_inner(order.clone());
                Py::new(py, snapshot).map(|value| value.into_py(py))
            })
            .collect()
    }

    pub fn __len__(&self) -> usize {
        self.inner.orders.len()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "InstrumentAccountSnapshot(instrument={}, orders={})",
            self.instrument(),
            self.inner.orders.len()
        ))
    }

    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyObject {
        let py = other.py();
        let other_handle = other.extract::<Py<Self>>().ok();

        match op {
            CompareOp::Eq => {
                let result = other_handle
                    .map(|handle| {
                        let other = handle.borrow(py);
                        self.inner.instrument == other.inner.instrument
                            && self.inner.orders == other.inner.orders
                    })
                    .unwrap_or(false);
                result.into_py(py)
            }
            CompareOp::Ne => {
                let result = other_handle
                    .map(|handle| {
                        let other = handle.borrow(py);
                        self.inner.instrument != other.inner.instrument
                            || self.inner.orders != other.inner.orders
                    })
                    .unwrap_or(true);
                result.into_py(py)
            }
            _ => py.NotImplemented(),
        }
    }
}

/// Wrapper around [`AccountSnapshot`] for Python exposure.
#[pyclass(module = "barter_python", name = "AccountSnapshot", unsendable)]
#[derive(Debug, Clone)]
pub struct PyAccountSnapshot {
    pub(crate) inner: AccountSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex>,
}

impl PyAccountSnapshot {
    pub(crate) fn clone_inner(
        &self,
    ) -> AccountSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex> {
        self.inner.clone()
    }

    pub(crate) fn from_inner(
        inner: AccountSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex>,
    ) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyAccountSnapshot {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (exchange, balances, instruments))]
    pub fn __new__(
        exchange: usize,
        balances: Vec<(usize, f64, f64, DateTime<Utc>)>,
        instruments: Vec<Py<PyInstrumentAccountSnapshot>>,
    ) -> PyResult<Self> {
        let exchange_index = ExchangeIndex(exchange);

        let mut converted_balances = Vec::with_capacity(balances.len());
        for (asset, total, free, time_exchange) in balances {
            if !total.is_finite() || total < 0.0 {
                return Err(PyValueError::new_err(
                    "total balance must be a non-negative finite value",
                ));
            }

            if !free.is_finite() || free < 0.0 {
                return Err(PyValueError::new_err(
                    "free balance must be a non-negative finite value",
                ));
            }

            if free > total {
                return Err(PyValueError::new_err(
                    "free balance cannot exceed total balance",
                ));
            }

            let total_decimal = parse_decimal(total, "total balance")?;
            let free_decimal = parse_decimal(free, "free balance")?;

            let balance = Balance::new(total_decimal, free_decimal);
            converted_balances.push(AssetBalance::new(AssetIndex(asset), balance, time_exchange));
        }

        let converted_instruments: Vec<_> = Python::with_gil(|py| {
            instruments
                .into_iter()
                .map(|instrument| {
                    let borrowed = instrument.borrow(py);
                    borrowed.clone_inner()
                })
                .collect()
        });

        for snapshot in &converted_instruments {
            for order in &snapshot.orders {
                if order.key.exchange != exchange_index {
                    return Err(PyValueError::new_err(format!(
                        "order exchange {} does not match snapshot exchange {}",
                        order.key.exchange.index(),
                        exchange_index.index(),
                    )));
                }
            }
        }

        Ok(Self {
            inner: AccountSnapshot::new(exchange_index, converted_balances, converted_instruments),
        })
    }

    #[getter]
    pub fn exchange(&self) -> usize {
        self.inner.exchange.index()
    }

    pub fn balances(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.inner
            .balances
            .iter()
            .map(|balance| {
                let wrapper = PyExecutionAssetBalance::from_inner(balance.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            })
            .collect()
    }

    pub fn instruments(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.inner
            .instruments
            .iter()
            .map(|snapshot| {
                let instrument = PyInstrumentAccountSnapshot {
                    inner: snapshot.clone(),
                };
                Py::new(py, instrument).map(|value| value.into_py(py))
            })
            .collect()
    }

    pub fn time_most_recent(&self) -> Option<DateTime<Utc>> {
        self.inner.time_most_recent()
    }

    pub fn assets(&self) -> Vec<usize> {
        let mut seen = HashSet::new();
        let mut values = Vec::new();
        for asset in self.inner.assets() {
            let index = asset.index();
            if seen.insert(index) {
                values.push(index);
            }
        }

        values
    }

    pub fn instruments_iter(&self) -> Vec<usize> {
        let mut seen = HashSet::new();
        let mut values = Vec::new();
        for instrument in self.inner.instruments() {
            let index = instrument.index();
            if seen.insert(index) {
                values.push(index);
            }
        }

        values
    }

    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyObject {
        let py = other.py();
        let other_handle = other.extract::<Py<Self>>().ok();

        match op {
            CompareOp::Eq => {
                let result = other_handle
                    .map(|handle| {
                        let other = handle.borrow(py);
                        self.inner.exchange == other.inner.exchange
                            && self.inner.balances == other.inner.balances
                            && self.inner.instruments == other.inner.instruments
                    })
                    .unwrap_or(false);
                result.into_py(py)
            }
            CompareOp::Ne => {
                let result = other_handle
                    .map(|handle| {
                        let other = handle.borrow(py);
                        self.inner.exchange != other.inner.exchange
                            || self.inner.balances != other.inner.balances
                            || self.inner.instruments != other.inner.instruments
                    })
                    .unwrap_or(true);
                result.into_py(py)
            }
            _ => py.NotImplemented(),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "AccountSnapshot(exchange={}, balances={}, instruments={})",
            self.exchange(),
            self.inner.balances.len(),
            self.inner.instruments.len()
        ))
    }
}
