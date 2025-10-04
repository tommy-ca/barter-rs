#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]

//! Python bindings for the Barter backtest module.

use crate::summary::PyTradingSummary;
use barter::backtest::{
    market_data::{BacktestMarketData, MarketDataInMemory},
    summary::{BacktestSummary, MultiBacktestSummary},
    run_backtests, BacktestArgsConstant, BacktestArgsDynamic,
};
use barter::engine::state::EngineState;
use barter::statistic::time::TimeInterval;
use barter_data::streams::consumer::MarketStreamEvent;
use barter_execution::AccountEvent;
use barter_instrument::{index::IndexedInstruments, instrument::InstrumentIndex};
use barter_integration::snapshot::Snapshot;
use pyo3::{prelude::*, pyclass, pymethods, PyResult};
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::sync::Arc;

/// Wrapper around [`BacktestSummary`] for Python.
#[pyclass(module = "barter_python", name = "BacktestSummary", unsendable)]
#[derive(Debug, Clone)]
pub struct PyBacktestSummary {
    inner: BacktestSummary<barter::statistic::time::Annual365>,
}

#[pymethods]
impl PyBacktestSummary {
    /// Create a new [`BacktestSummary`].
    #[new]
    pub fn new(id: &str, risk_free_return: f64, trading_summary: &PyTradingSummary) -> Self {
        Self {
            inner: BacktestSummary {
                id: SmolStr::new(id),
                risk_free_return: Decimal::from_f64_retain(risk_free_return).unwrap_or_default(),
                trading_summary: trading_summary.clone_inner(),
            },
        }
    }

    /// Unique identifier for this backtest.
    #[getter]
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    /// Risk-free return rate used for performance metrics.
    #[getter]
    pub fn risk_free_return(&self) -> f64 {
        self.inner.risk_free_return.to_f64().unwrap_or(0.0)
    }

    /// Performance metrics and statistics from the backtest.
    #[getter]
    pub fn trading_summary(&self) -> PyTradingSummary {
        PyTradingSummary::new(self.inner.trading_summary.clone())
    }
}

/// Wrapper around [`MultiBacktestSummary`] for Python.
#[pyclass(module = "barter_python", name = "MultiBacktestSummary", unsendable)]
#[derive(Debug, Clone)]
pub struct PyMultiBacktestSummary {
    inner: MultiBacktestSummary<barter::statistic::time::Annual365>,
}

#[pymethods]
impl PyMultiBacktestSummary {
    /// Number of backtests run in this batch.
    #[getter]
    pub fn num_backtests(&self) -> usize {
        self.inner.num_backtests
    }

    /// Total execution time for all backtests in seconds.
    #[getter]
    pub fn duration_seconds(&self) -> f64 {
        self.inner.duration.as_secs_f64()
    }

    /// Collection of `BacktestSummary`s.
    #[getter]
    pub fn summaries(&self) -> Vec<PyBacktestSummary> {
        self.inner
            .summaries
            .iter()
            .map(|summary| PyBacktestSummary {
                inner: summary.clone(),
            })
            .collect()
    }
}

/// Wrapper around [`MarketDataInMemory`] for Python.
#[pyclass(module = "barter_python", name = "MarketDataInMemory", unsendable)]
#[derive(Debug, Clone)]
pub struct PyMarketDataInMemory {
    inner: MarketDataInMemory<barter_data::event::DataKind>,
}

#[pymethods]
impl PyMarketDataInMemory {
    /// Create a new in-memory market data source.
    #[new]
    pub fn new(events: Vec<PyObject>) -> PyResult<Self> {
        // This is a placeholder - proper implementation would need to convert PyObjects to MarketStreamEvent
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "MarketDataInMemory constructor not yet implemented"
        ))
    }
}

/// Run multiple backtests concurrently from Python.
///
/// This is a simplified interface for Python users.
#[pyfunction]
pub fn run_backtests_py(
    py: Python<'_>,
    instruments: PyObject,
    executions: PyObject,
    market_data: PyObject,
    strategies: Vec<PyObject>,
) -> PyResult<PyMultiBacktestSummary> {
    // This is a placeholder - proper implementation would need to handle Python objects
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "run_backtests_py not yet implemented"
    ))
}</content>
</xai:function_call: <parameter name="write">  
