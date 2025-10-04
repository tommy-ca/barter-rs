use std::collections::HashSet;

use crate::{
    command::{PyOrderSnapshot, parse_decimal},
    summary::decimal_to_py,
};
use barter_execution::{
    AccountSnapshot, InstrumentAccountSnapshot,
    balance::{AssetBalance, Balance},
};
use barter_instrument::{asset::AssetIndex, exchange::ExchangeIndex, instrument::InstrumentIndex};
use chrono::{DateTime, Utc};
use pyo3::{
    Bound, Py, PyObject, PyResult, Python, exceptions::PyValueError, prelude::*, types::PyModule,
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

#[pymethods]
impl PyInstrumentAccountSnapshot {
    #[new]
    #[pyo3(signature = (instrument, orders))]
    pub fn __new__(instrument: usize, orders: Vec<Py<PyOrderSnapshot>>) -> PyResult<Self> {
        let instrument_index = InstrumentIndex(instrument);

        let mut converted = Vec::with_capacity(orders.len());
        Python::with_gil(|py| {
            for order in orders {
                let borrowed = order.borrow(py);
                converted.push(borrowed.clone_inner());
            }
        });

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

    fn execution_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
        PyModule::import_bound(py, "barter_python.execution")
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

        let mut converted_instruments = Vec::with_capacity(instruments.len());
        Python::with_gil(|py| {
            for instrument in instruments {
                let borrowed = instrument.borrow(py);
                converted_instruments.push(borrowed.clone_inner());
            }
        });

        Ok(Self {
            inner: AccountSnapshot::new(exchange_index, converted_balances, converted_instruments),
        })
    }

    #[getter]
    pub fn exchange(&self) -> usize {
        self.inner.exchange.index()
    }

    pub fn balances(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let module = Self::execution_module(py)?;
        let balance_cls = module.getattr("Balance")?;
        let asset_balance_cls = module.getattr("AssetBalance")?;

        self.inner
            .balances
            .iter()
            .map(|balance| {
                let total = decimal_to_py(py, balance.balance.total)?;
                let free = decimal_to_py(py, balance.balance.free)?;
                let balance_obj = balance_cls.call1((total, free))?;
                let asset_balance_obj = asset_balance_cls.call1((
                    balance.asset.index(),
                    balance_obj,
                    balance.time_exchange,
                ))?;
                Ok(asset_balance_obj.into_py(py))
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

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "AccountSnapshot(exchange={}, balances={}, instruments={})",
            self.exchange(),
            self.inner.balances.len(),
            self.inner.instruments.len()
        ))
    }
}
