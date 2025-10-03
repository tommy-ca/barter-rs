use barter::statistic::{
    metric::{
        calmar::CalmarRatio,
        drawdown::{Drawdown, mean::MeanDrawdown},
        rate_of_return::RateOfReturn,
        sharpe::SharpeRatio,
        sortino::SortinoRatio,
    },
    summary::{TradingSummary, asset::TearSheetAsset, instrument::TearSheet},
    time::TimeInterval,
};
use barter_execution::balance::Balance;
use chrono::{DateTime, Utc};
use pyo3::{
    PyClass,
    prelude::*,
    types::{IntoPyDict, PyDict, PyModule},
};
use rust_decimal::Decimal;
use std::fmt::Write;

pub fn summary_to_py<Interval>(
    py: Python<'_>,
    summary: TradingSummary<Interval>,
) -> PyResult<Py<PyTradingSummary>>
where
    Interval: TimeInterval,
{
    PyTradingSummary::from_summary(py, summary)
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
    fn from_components(
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
    fn from_drawdown(py: Python<'_>, drawdown: Drawdown) -> PyResult<Py<PyDrawdown>> {
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
    fn from_mean(py: Python<'_>, mean: MeanDrawdown) -> PyResult<Py<PyMeanDrawdown>> {
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

fn decimal_to_py(py: Python<'_>, value: Decimal) -> PyResult<PyObject> {
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
