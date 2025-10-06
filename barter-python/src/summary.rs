use barter::backtest::summary::{BacktestSummary, MultiBacktestSummary};
use barter::statistic::{
    metric::{
        calmar::CalmarRatio,
        drawdown::{Drawdown, mean::MeanDrawdown},
        rate_of_return::RateOfReturn,
        sharpe::SharpeRatio,
        sortino::SortinoRatio,
    },
    summary::{
        TradingSummary, TradingSummaryGenerator, asset::TearSheetAsset, instrument::TearSheet,
    },
    time::{Annual252, Annual365, Daily, TimeInterval},
};
use barter_execution::balance::Balance;
use barter_integration::snapshot::Snapshot;
use chrono::{DateTime, Utc};
use pyo3::exceptions::PyTypeError;
use pyo3::{
    PyClass,
    prelude::*,
    types::{IntoPyDict, PyDict, PyModule},
};
use rust_decimal::Decimal;
use std::fmt::Write;

use crate::{
    common::{SummaryInterval, parse_summary_interval},
    execution::PyExecutionAssetBalance,
    system::PyPositionExit,
};

#[pyclass(module = "barter_python", name = "TradingSummaryGenerator", unsendable)]
pub struct PyTradingSummaryGenerator {
    inner: TradingSummaryGenerator,
}

impl PyTradingSummaryGenerator {
    pub(crate) fn from_inner(
        py: Python<'_>,
        generator: TradingSummaryGenerator,
    ) -> PyResult<Py<PyTradingSummaryGenerator>> {
        Py::new(py, PyTradingSummaryGenerator { inner: generator })
    }

    fn duration_to_py(&self, py: Python<'_>) -> PyResult<PyObject> {
        let duration = self
            .inner
            .time_engine_now
            .signed_duration_since(self.inner.time_engine_start);
        let millis = duration.num_milliseconds();
        timedelta_from_millis(py, millis)
    }

    fn generate_internal(
        &mut self,
        py: Python<'_>,
        interval: Option<&str>,
    ) -> PyResult<Py<PyTradingSummary>> {
        let summary_interval = parse_summary_interval(interval)?;
        match summary_interval {
            SummaryInterval::Daily => summary_to_py(py, self.inner.generate(Daily)),
            SummaryInterval::Annual252 => summary_to_py(py, self.inner.generate(Annual252)),
            SummaryInterval::Annual365 => summary_to_py(py, self.inner.generate(Annual365)),
        }
    }
}

#[pymethods]
impl PyTradingSummaryGenerator {
    #[new]
    fn __new__() -> PyResult<Self> {
        Err(PyTypeError::new_err(
            "TradingSummaryGenerator instances are created internally; use backtest or shutdown helpers",
        ))
    }

    #[getter]
    pub fn risk_free_return(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.risk_free_return)
    }

    #[getter]
    pub fn time_engine_start(&self) -> DateTime<Utc> {
        self.inner.time_engine_start
    }

    #[getter]
    pub fn time_engine_now(&self) -> DateTime<Utc> {
        self.inner.time_engine_now
    }

    #[getter]
    pub fn trading_duration(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.duration_to_py(py)
    }

    #[pyo3(signature = (interval = None))]
    pub fn generate(
        &mut self,
        py: Python<'_>,
        interval: Option<&str>,
    ) -> PyResult<Py<PyTradingSummary>> {
        self.generate_internal(py, interval)
    }

    pub fn update_from_balance(&mut self, balance: &PyExecutionAssetBalance) -> PyResult<()> {
        let snapshot = Snapshot::new(&balance.inner);
        self.inner.update_from_balance(snapshot);
        Ok(())
    }

    pub fn update_from_position(&mut self, position: &PyPositionExit) -> PyResult<()> {
        let exited = position.to_position_exited();
        self.inner.update_from_position(&exited);
        Ok(())
    }

    pub fn update_time_now(&mut self, time: DateTime<Utc>) {
        self.inner.update_time_now(time);
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "TradingSummaryGenerator(risk_free_return={}, time_now={})",
            self.inner.risk_free_return, self.inner.time_engine_now
        ))
    }
}

pub fn summary_to_py<Interval>(
    py: Python<'_>,
    summary: TradingSummary<Interval>,
) -> PyResult<Py<PyTradingSummary>>
where
    Interval: TimeInterval,
{
    PyTradingSummary::from_summary(py, summary)
}

pub fn backtest_summary_to_py<Interval>(
    py: Python<'_>,
    summary: BacktestSummary<Interval>,
) -> PyResult<Py<PyBacktestSummary>>
where
    Interval: TimeInterval,
{
    PyBacktestSummary::from_backtest_summary(py, summary)
}

pub fn multi_backtest_summary_to_py<Interval>(
    py: Python<'_>,
    summary: MultiBacktestSummary<Interval>,
) -> PyResult<Py<PyMultiBacktestSummary>>
where
    Interval: TimeInterval,
{
    PyMultiBacktestSummary::from_multi_backtest_summary(py, summary)
}

#[pyclass(module = "barter_python", name = "TradingSummary", unsendable)]
pub struct PyTradingSummary {
    time_engine_start: DateTime<Utc>,
    time_engine_end: DateTime<Utc>,
    instruments: Vec<(String, Py<PyInstrumentTearSheet>)>,
    assets: Vec<(String, Py<PyAssetTearSheet>)>,
}

impl PyTradingSummary {
    fn from_summary<Interval>(
        py: Python<'_>,
        summary: TradingSummary<Interval>,
    ) -> PyResult<Py<PyTradingSummary>>
    where
        Interval: TimeInterval,
    {
        let TradingSummary {
            time_engine_start,
            time_engine_end,
            instruments,
            assets,
        } = summary;

        let mut py_instruments = Vec::with_capacity(instruments.len());
        for (instrument, sheet) in instruments {
            let name = instrument.to_string();
            let sheet = PyInstrumentTearSheet::from_tear_sheet(py, sheet)?;
            py_instruments.push((name, sheet));
        }

        let mut py_assets = Vec::with_capacity(assets.len());
        for (exchange_asset, sheet) in assets {
            let key = format!(
                "{}:{}",
                exchange_asset.exchange.as_str(),
                exchange_asset.asset.as_ref()
            );
            let sheet = PyAssetTearSheet::from_tear_sheet(py, sheet)?;
            py_assets.push((key, sheet));
        }

        Py::new(
            py,
            PyTradingSummary {
                time_engine_start,
                time_engine_end,
                instruments: py_instruments,
                assets: py_assets,
            },
        )
    }

    fn instruments_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        for (name, sheet) in &self.instruments {
            dict.set_item(name, sheet.clone_ref(py))?;
        }
        Ok(dict.into())
    }

    fn assets_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        for (name, sheet) in &self.assets {
            dict.set_item(name, sheet.clone_ref(py))?;
        }
        Ok(dict.into())
    }
}

#[pymethods]
impl PyTradingSummary {
    #[getter]
    pub fn time_engine_start(&self) -> DateTime<Utc> {
        self.time_engine_start
    }

    #[getter]
    pub fn time_engine_end(&self) -> DateTime<Utc> {
        self.time_engine_end
    }

    #[getter]
    pub fn instruments(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.instruments_dict(py)?.into_py(py))
    }

    #[getter]
    pub fn assets(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.assets_dict(py)?.into_py(py))
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        dict.set_item("time_engine_start", self.time_engine_start)?;
        dict.set_item("time_engine_end", self.time_engine_end)?;

        let instruments = PyDict::new_bound(py);
        for (name, sheet) in &self.instruments {
            instruments.set_item(name, sheet.clone_ref(py).call_method0(py, "to_dict")?)?;
        }
        dict.set_item("instruments", instruments)?;

        let assets = PyDict::new_bound(py);
        for (name, sheet) in &self.assets {
            assets.set_item(name, sheet.clone_ref(py).call_method0(py, "to_dict")?)?;
        }
        dict.set_item("assets", assets)?;

        Ok(dict.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut repr = String::new();
        write!(
            &mut repr,
            "TradingSummary(start={}, end={}, instruments={}, assets={})",
            self.time_engine_start,
            self.time_engine_end,
            self.instruments.len(),
            self.assets.len()
        )
        .unwrap();
        Ok(repr)
    }
}

#[pyclass(module = "barter_python", name = "InstrumentTearSheet", unsendable)]
pub struct PyInstrumentTearSheet {
    pnl: Decimal,
    pnl_return: Py<PyMetricWithInterval>,
    sharpe_ratio: Py<PyMetricWithInterval>,
    sortino_ratio: Py<PyMetricWithInterval>,
    calmar_ratio: Py<PyMetricWithInterval>,
    pnl_drawdown: Option<Py<PyDrawdown>>,
    pnl_drawdown_mean: Option<Py<PyMeanDrawdown>>,
    pnl_drawdown_max: Option<Py<PyDrawdown>>,
    win_rate: Option<Decimal>,
    profit_factor: Option<Decimal>,
}

impl PyInstrumentTearSheet {
    fn from_tear_sheet<Interval>(
        py: Python<'_>,
        sheet: TearSheet<Interval>,
    ) -> PyResult<Py<PyInstrumentTearSheet>>
    where
        Interval: TimeInterval,
    {
        let TearSheet {
            pnl,
            pnl_return,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            pnl_drawdown,
            pnl_drawdown_mean,
            pnl_drawdown_max,
            win_rate,
            profit_factor,
        } = sheet;

        let RateOfReturn {
            value: pnl_return_value,
            interval: pnl_return_interval,
        } = pnl_return;
        let SharpeRatio {
            value: sharpe_value,
            interval: sharpe_interval,
        } = sharpe_ratio;
        let SortinoRatio {
            value: sortino_value,
            interval: sortino_interval,
        } = sortino_ratio;
        let CalmarRatio {
            value: calmar_value,
            interval: calmar_interval,
        } = calmar_ratio;

        let pnl_return = PyMetricWithInterval::from_components(
            py,
            pnl_return_value,
            interval_name(&pnl_return_interval),
        )?;
        let sharpe_ratio = PyMetricWithInterval::from_components(
            py,
            sharpe_value,
            interval_name(&sharpe_interval),
        )?;
        let sortino_ratio = PyMetricWithInterval::from_components(
            py,
            sortino_value,
            interval_name(&sortino_interval),
        )?;
        let calmar_ratio = PyMetricWithInterval::from_components(
            py,
            calmar_value,
            interval_name(&calmar_interval),
        )?;

        let pnl_drawdown = pnl_drawdown
            .map(|drawdown| PyDrawdown::from_drawdown(py, drawdown))
            .transpose()?;
        let pnl_drawdown_mean = pnl_drawdown_mean
            .map(|mean| PyMeanDrawdown::from_mean(py, mean))
            .transpose()?;
        let pnl_drawdown_max = pnl_drawdown_max
            .map(|max| PyDrawdown::from_drawdown(py, max.0))
            .transpose()?;

        Py::new(
            py,
            PyInstrumentTearSheet {
                pnl,
                pnl_return,
                sharpe_ratio,
                sortino_ratio,
                calmar_ratio,
                pnl_drawdown,
                pnl_drawdown_mean,
                pnl_drawdown_max,
                win_rate: win_rate.map(|rate| rate.value),
                profit_factor: profit_factor.map(|factor| factor.value),
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("pnl", decimal_to_py(py, self.pnl)?)?;
        dict.set_item(
            "pnl_return",
            self.pnl_return.clone_ref(py).call_method0(py, "to_dict")?,
        )?;
        dict.set_item(
            "sharpe_ratio",
            self.sharpe_ratio
                .clone_ref(py)
                .call_method0(py, "to_dict")?,
        )?;
        dict.set_item(
            "sortino_ratio",
            self.sortino_ratio
                .clone_ref(py)
                .call_method0(py, "to_dict")?,
        )?;
        dict.set_item(
            "calmar_ratio",
            self.calmar_ratio
                .clone_ref(py)
                .call_method0(py, "to_dict")?,
        )?;
        dict.set_item(
            "pnl_drawdown",
            optional_to_object(py, self.pnl_drawdown.as_ref())?,
        )?;
        dict.set_item(
            "pnl_drawdown_mean",
            optional_to_object(py, self.pnl_drawdown_mean.as_ref())?,
        )?;
        dict.set_item(
            "pnl_drawdown_max",
            optional_to_object(py, self.pnl_drawdown_max.as_ref())?,
        )?;
        dict.set_item("win_rate", optional_decimal(py, self.win_rate)?)?;
        dict.set_item("profit_factor", optional_decimal(py, self.profit_factor)?)?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyInstrumentTearSheet {
    #[getter]
    pub fn pnl(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.pnl)
    }

    #[getter]
    pub fn pnl_return(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.pnl_return.clone_ref(py).into_py(py))
    }

    #[getter]
    pub fn sharpe_ratio(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.sharpe_ratio.clone_ref(py).into_py(py))
    }

    #[getter]
    pub fn sortino_ratio(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.sortino_ratio.clone_ref(py).into_py(py))
    }

    #[getter]
    pub fn calmar_ratio(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.calmar_ratio.clone_ref(py).into_py(py))
    }

    #[getter]
    pub fn pnl_drawdown(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.pnl_drawdown.as_ref())
    }

    #[getter]
    pub fn pnl_drawdown_mean(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.pnl_drawdown_mean.as_ref())
    }

    #[getter]
    pub fn pnl_drawdown_max(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.pnl_drawdown_max.as_ref())
    }

    #[getter]
    pub fn win_rate(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_decimal(py, self.win_rate)
    }

    #[getter]
    pub fn profit_factor(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_decimal(py, self.profit_factor)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let pnl = decimal_to_py(py, self.pnl)?;
        Ok(format!(
            "InstrumentTearSheet(pnl={}, drawdown_present={})",
            pnl,
            self.pnl_drawdown.is_some()
        ))
    }
}

#[pyclass(module = "barter_python", name = "AssetTearSheet", unsendable)]
pub struct PyAssetTearSheet {
    balance_end: Option<Py<PyBalance>>,
    drawdown: Option<Py<PyDrawdown>>,
    drawdown_mean: Option<Py<PyMeanDrawdown>>,
    drawdown_max: Option<Py<PyDrawdown>>,
}

impl PyAssetTearSheet {
    fn from_tear_sheet(py: Python<'_>, sheet: TearSheetAsset) -> PyResult<Py<PyAssetTearSheet>> {
        let TearSheetAsset {
            balance_end,
            drawdown,
            drawdown_mean,
            drawdown_max,
        } = sheet;

        let balance_end = balance_end
            .map(|balance| PyBalance::from_balance(py, balance))
            .transpose()?;
        let drawdown = drawdown
            .map(|drawdown| PyDrawdown::from_drawdown(py, drawdown))
            .transpose()?;
        let drawdown_mean = drawdown_mean
            .map(|mean| PyMeanDrawdown::from_mean(py, mean))
            .transpose()?;
        let drawdown_max = drawdown_max
            .map(|max| PyDrawdown::from_drawdown(py, max.0))
            .transpose()?;

        Py::new(
            py,
            PyAssetTearSheet {
                balance_end,
                drawdown,
                drawdown_mean,
                drawdown_max,
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item(
            "balance_end",
            optional_to_object(py, self.balance_end.as_ref())?,
        )?;
        dict.set_item("drawdown", optional_to_object(py, self.drawdown.as_ref())?)?;
        dict.set_item(
            "drawdown_mean",
            optional_to_object(py, self.drawdown_mean.as_ref())?,
        )?;
        dict.set_item(
            "drawdown_max",
            optional_to_object(py, self.drawdown_max.as_ref())?,
        )?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyAssetTearSheet {
    #[getter]
    pub fn balance_end(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.balance_end.as_ref())
    }

    #[getter]
    pub fn drawdown(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.drawdown.as_ref())
    }

    #[getter]
    pub fn drawdown_mean(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.drawdown_mean.as_ref())
    }

    #[getter]
    pub fn drawdown_max(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        optional_to_object(py, self.drawdown_max.as_ref())
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "AssetTearSheet(balance_end={}, drawdown_present={})",
            self.balance_end.is_some(),
            self.drawdown.is_some()
        ))
    }
}

#[pyclass(module = "barter_python", name = "MetricWithInterval", unsendable)]
pub struct PyMetricWithInterval {
    value: Decimal,
    interval: String,
}

impl PyMetricWithInterval {
    pub(crate) fn from_components(
        py: Python<'_>,
        value: Decimal,
        interval: String,
    ) -> PyResult<Py<PyMetricWithInterval>> {
        Py::new(py, PyMetricWithInterval { value, interval })
    }
}

#[pymethods]
impl PyMetricWithInterval {
    #[getter]
    pub fn value(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.value)
    }

    #[getter]
    pub fn interval(&self) -> &str {
        &self.interval
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        dict.set_item("value", decimal_to_py(py, self.value)?)?;
        dict.set_item("interval", &self.interval)?;
        Ok(dict.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let value = decimal_to_py(py, self.value)?;
        Ok(format!(
            "MetricWithInterval(value={}, interval={})",
            value, self.interval
        ))
    }
}

#[pyclass(module = "barter_python", name = "Drawdown", unsendable)]
pub struct PyDrawdown {
    value: Decimal,
    time_start: DateTime<Utc>,
    time_end: DateTime<Utc>,
}

impl PyDrawdown {
    pub(crate) fn from_drawdown(py: Python<'_>, drawdown: Drawdown) -> PyResult<Py<PyDrawdown>> {
        let Drawdown {
            value,
            time_start,
            time_end,
        } = drawdown;
        Py::new(
            py,
            PyDrawdown {
                value,
                time_start,
                time_end,
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("value", decimal_to_py(py, self.value)?)?;
        dict.set_item("time_start", self.time_start)?;
        dict.set_item("time_end", self.time_end)?;
        dict.set_item("duration", timedelta_from_millis(py, self.duration_ms())?)?;
        Ok(dict.into())
    }

    fn duration_ms(&self) -> i64 {
        self.time_end
            .signed_duration_since(self.time_start)
            .num_milliseconds()
    }
}

#[pymethods]
impl PyDrawdown {
    #[getter]
    pub fn value(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.value)
    }

    #[getter]
    pub fn time_start(&self) -> DateTime<Utc> {
        self.time_start
    }

    #[getter]
    pub fn time_end(&self) -> DateTime<Utc> {
        self.time_end
    }

    pub fn duration(&self, py: Python<'_>) -> PyResult<PyObject> {
        timedelta_from_millis(py, self.duration_ms())
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let value = decimal_to_py(py, self.value)?;
        Ok(format!(
            "Drawdown(value={}, start={}, end={})",
            value, self.time_start, self.time_end
        ))
    }
}

#[pyclass(module = "barter_python", name = "MeanDrawdown", unsendable)]
pub struct PyMeanDrawdown {
    mean_drawdown: Decimal,
    mean_drawdown_ms: i64,
}

impl PyMeanDrawdown {
    pub(crate) fn from_mean(py: Python<'_>, mean: MeanDrawdown) -> PyResult<Py<PyMeanDrawdown>> {
        let MeanDrawdown {
            mean_drawdown,
            mean_drawdown_ms,
        } = mean;
        Py::new(
            py,
            PyMeanDrawdown {
                mean_drawdown,
                mean_drawdown_ms,
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("mean_drawdown", decimal_to_py(py, self.mean_drawdown)?)?;
        dict.set_item(
            "mean_duration",
            timedelta_from_millis(py, self.mean_drawdown_ms)?,
        )?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyMeanDrawdown {
    #[getter]
    pub fn mean_drawdown(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.mean_drawdown)
    }

    #[getter]
    pub fn mean_duration(&self, py: Python<'_>) -> PyResult<PyObject> {
        timedelta_from_millis(py, self.mean_drawdown_ms)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let mean = decimal_to_py(py, self.mean_drawdown)?;
        Ok(format!(
            "MeanDrawdown(mean={}, duration_ms={})",
            mean, self.mean_drawdown_ms
        ))
    }
}

#[pyclass(module = "barter_python", name = "Balance", unsendable)]
pub struct PyBalance {
    total: Decimal,
    free: Decimal,
}

impl PyBalance {
    fn from_balance(py: Python<'_>, balance: Balance) -> PyResult<Py<PyBalance>> {
        let Balance { total, free } = balance;
        Py::new(py, PyBalance { total, free })
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("total", decimal_to_py(py, self.total)?)?;
        dict.set_item("free", decimal_to_py(py, self.free)?)?;
        let used = self.total - self.free;
        dict.set_item("used", decimal_to_py(py, used)?)?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyBalance {
    #[getter]
    pub fn total(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.total)
    }

    #[getter]
    pub fn free(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.free)
    }

    #[getter]
    pub fn used(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.total - self.free)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let total = decimal_to_py(py, self.total)?;
        let free = decimal_to_py(py, self.free)?;
        Ok(format!("Balance(total={}, free={})", total, free))
    }
}

#[pyclass(module = "barter_python", name = "BacktestSummary", unsendable)]
pub struct PyBacktestSummary {
    id: String,
    risk_free_return: Decimal,
    trading_summary: Py<PyTradingSummary>,
}

impl PyBacktestSummary {
    #[allow(dead_code)]
    fn from_backtest_summary<Interval>(
        py: Python<'_>,
        summary: BacktestSummary<Interval>,
    ) -> PyResult<Py<PyBacktestSummary>>
    where
        Interval: TimeInterval,
    {
        let BacktestSummary {
            id,
            risk_free_return,
            trading_summary,
        } = summary;

        let trading_summary = summary_to_py(py, trading_summary)?;

        Py::new(
            py,
            PyBacktestSummary {
                id: id.to_string(),
                risk_free_return,
                trading_summary,
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item(
            "risk_free_return",
            decimal_to_py(py, self.risk_free_return)?,
        )?;
        dict.set_item(
            "trading_summary",
            self.trading_summary
                .clone_ref(py)
                .call_method0(py, "to_dict")?,
        )?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyBacktestSummary {
    #[getter]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[getter]
    pub fn risk_free_return(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.risk_free_return)
    }

    #[getter]
    pub fn trading_summary(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.trading_summary.clone_ref(py).into_py(py))
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let rfr = decimal_to_py(py, self.risk_free_return)?;
        Ok(format!(
            "BacktestSummary(id={}, risk_free_return={})",
            self.id, rfr
        ))
    }
}

#[pyclass(module = "barter_python", name = "MultiBacktestSummary", unsendable)]
pub struct PyMultiBacktestSummary {
    num_backtests: usize,
    duration_ms: u128,
    summaries: Vec<Py<PyBacktestSummary>>,
}

impl PyMultiBacktestSummary {
    #[allow(dead_code)]
    fn from_multi_backtest_summary<Interval>(
        py: Python<'_>,
        summary: MultiBacktestSummary<Interval>,
    ) -> PyResult<Py<PyMultiBacktestSummary>>
    where
        Interval: TimeInterval,
    {
        let MultiBacktestSummary {
            num_backtests,
            duration,
            summaries,
        } = summary;

        let mut py_summaries = Vec::with_capacity(summaries.len());
        for summary in summaries {
            let py_summary = PyBacktestSummary::from_backtest_summary(py, summary)?;
            py_summaries.push(py_summary);
        }

        Py::new(
            py,
            PyMultiBacktestSummary {
                num_backtests,
                duration_ms: duration.as_millis(),
                summaries: py_summaries,
            },
        )
    }

    fn dictionary(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("num_backtests", self.num_backtests)?;
        dict.set_item("duration_ms", self.duration_ms)?;
        let summaries = self
            .summaries
            .iter()
            .map(|s| s.clone_ref(py).call_method0(py, "to_dict"))
            .collect::<PyResult<Vec<_>>>()?;
        dict.set_item("summaries", summaries)?;
        Ok(dict.into())
    }
}

#[pymethods]
impl PyMultiBacktestSummary {
    #[getter]
    pub fn num_backtests(&self) -> usize {
        self.num_backtests
    }

    #[getter]
    pub fn duration_ms(&self) -> u128 {
        self.duration_ms
    }

    #[getter]
    pub fn summaries(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        Ok(self
            .summaries
            .iter()
            .map(|s| s.clone_ref(py).into_py(py))
            .collect())
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.dictionary(py)?.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "MultiBacktestSummary(num_backtests={}, duration_ms={})",
            self.num_backtests, self.duration_ms
        ))
    }
}

fn interval_name<Interval>(interval: &Interval) -> String
where
    Interval: TimeInterval,
{
    interval.name().to_string()
}

fn optional_to_object<T>(py: Python<'_>, value: Option<&Py<T>>) -> PyResult<Option<PyObject>>
where
    T: PyClass,
{
    Ok(value.map(|item| item.clone_ref(py).into_py(py)))
}

fn optional_decimal(py: Python<'_>, value: Option<Decimal>) -> PyResult<Option<PyObject>> {
    value.map(|decimal| decimal_to_py(py, decimal)).transpose()
}

pub(crate) fn decimal_to_py(py: Python<'_>, value: Decimal) -> PyResult<PyObject> {
    let module = PyModule::import_bound(py, "decimal")?;
    let decimal_cls = module.getattr("Decimal")?;
    let value_str = value.to_string();
    Ok(decimal_cls.call1((value_str,))?.into_py(py))
}

fn timedelta_from_millis(py: Python<'_>, millis: i64) -> PyResult<PyObject> {
    let module = PyModule::import_bound(py, "datetime")?;
    let delta_cls = module.getattr("timedelta")?;
    let kwargs = [("milliseconds", millis)].into_py_dict_bound(py);
    Ok(delta_cls.call((), Some(&kwargs))?.into_py(py))
}

#[cfg(test)]
mod tests {
    use super::*;
    use barter::engine::state::position::PositionExited;
    use barter::statistic::summary::{
        asset::TearSheetAssetGenerator, instrument::TearSheetGenerator,
    };
    use barter_execution::balance::{AssetBalance as ExecAssetBalance, Balance as ExecBalance};
    use barter_execution::trade::{AssetFees, TradeId};
    use barter_instrument::{
        Side,
        asset::{AssetIndex, ExchangeAsset, name::AssetNameInternal},
        exchange::ExchangeId,
        instrument::{InstrumentIndex, name::InstrumentNameInternal},
    };
    use barter_integration::collection::FnvIndexMap;
    use chrono::{TimeDelta, TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_generator(start: DateTime<Utc>) -> TradingSummaryGenerator {
        let mut instruments = FnvIndexMap::default();
        instruments.insert(
            InstrumentNameInternal::new("binance_spot-btc_usdt"),
            TearSheetGenerator::init(start),
        );

        let mut assets = FnvIndexMap::default();
        assets.insert(
            ExchangeAsset::new(ExchangeId::BinanceSpot, AssetNameInternal::new("usdt")),
            TearSheetAssetGenerator::default(),
        );

        TradingSummaryGenerator::new(Decimal::ZERO, start, start, instruments, assets)
    }

    #[test]
    fn generator_updates_from_balance() {
        Python::with_gil(|py| {
            let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
            let generator = sample_generator(start);

            let py_generator = PyTradingSummaryGenerator::from_inner(py, generator).unwrap();
            let update_time = start + TimeDelta::hours(1);

            let balance = ExecBalance::new(
                Decimal::from_str("1000").unwrap(),
                Decimal::from_str("975").unwrap(),
            );
            let asset_balance = ExecAssetBalance::new(AssetIndex(0), balance, update_time);
            let py_balance =
                Py::new(py, PyExecutionAssetBalance::from_inner(asset_balance)).unwrap();

            {
                let balance_ref = py_balance.borrow(py);
                py_generator
                    .borrow_mut(py)
                    .update_from_balance(&balance_ref)
                    .unwrap();
            }

            py_generator.borrow_mut(py).update_time_now(update_time);

            let generated = py_generator
                .borrow_mut(py)
                .generate(py, Some("annual_252"))
                .unwrap();

            let summary = generated.borrow(py);
            assert_eq!(summary.time_engine_end(), update_time);
        });
    }

    #[test]
    fn generator_updates_from_position() {
        Python::with_gil(|py| {
            let start = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
            let generator = sample_generator(start);

            let py_generator = PyTradingSummaryGenerator::from_inner(py, generator).unwrap();

            let exit_time = start + TimeDelta::minutes(30);
            let position = PositionExited {
                instrument: InstrumentIndex(0),
                side: Side::Buy,
                price_entry_average: Decimal::from_str("10000").unwrap(),
                quantity_abs_max: Decimal::from_str("1").unwrap(),
                pnl_realised: Decimal::from_str("50").unwrap(),
                fees_enter: AssetFees::quote_fees(Decimal::from_str("5").unwrap()),
                fees_exit: AssetFees::quote_fees(Decimal::from_str("5").unwrap()),
                time_enter: start,
                time_exit: exit_time,
                trades: vec![TradeId::new("trade-1")],
            };

            let py_position = Py::new(py, PyPositionExit::from_position(&position)).unwrap();
            {
                let position_ref = py_position.borrow(py);
                py_generator
                    .borrow_mut(py)
                    .update_from_position(&position_ref)
                    .unwrap();
            }

            let summary = py_generator.borrow_mut(py).generate(py, None).unwrap();

            let summary_ref = summary.borrow(py);
            assert_eq!(summary_ref.time_engine_end(), exit_time);
        });
    }
}
