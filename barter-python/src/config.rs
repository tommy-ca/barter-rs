use crate::{command::parse_decimal, data::PyExchangeId};
use barter::system::config::{
    ExecutionConfig, RiskConfiguration, RiskInstrumentLimits, RiskLimits, RiskLimitsError,
    SystemConfig,
};
use barter_execution::{UnindexedAccountSnapshot, client::mock::MockExecutionConfig};
use barter_instrument::exchange::ExchangeId;
use pyo3::{
    Bound, Py, PyObject,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict, PyList, PyModule, PyType},
};
use rust_decimal::Decimal;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

/// Python wrapper around [`MockExecutionConfig`].
#[pyclass(module = "barter_python", name = "MockExecutionConfig", unsendable)]
#[derive(Clone)]
pub struct PyMockExecutionConfig {
    pub(crate) inner: MockExecutionConfig,
}

impl PyMockExecutionConfig {
    pub(crate) fn from_inner(inner: MockExecutionConfig) -> Self {
        Self { inner }
    }

    fn parse_snapshot(
        py: Python<'_>,
        value: Option<PyObject>,
        exchange: ExchangeId,
    ) -> PyResult<UnindexedAccountSnapshot> {
        match value {
            Some(obj) => {
                let json = PyModule::import_bound(py, "json")?;
                let dumps = json.getattr("dumps")?;
                let serialized: String = dumps.call1((obj,))?.extract()?;
                let mut value: serde_json::Value = serde_json::from_str(&serialized)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?;

                if let serde_json::Value::Object(ref mut obj) = value {
                    obj.insert(
                        "exchange".to_string(),
                        serde_json::Value::String(exchange.as_str().to_string()),
                    );
                }

                serde_json::from_value(value).map_err(|err| PyValueError::new_err(err.to_string()))
            }
            None => Ok(UnindexedAccountSnapshot::new(
                exchange,
                Vec::new(),
                Vec::new(),
            )),
        }
    }

    fn snapshot_to_py(py: Python<'_>, snapshot: &UnindexedAccountSnapshot) -> PyResult<PyObject> {
        let serialized = serde_json::to_string(snapshot)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let json = PyModule::import_bound(py, "json")?;
        let loads = json.getattr("loads")?;
        Ok(loads.call1((serialized,))?.into_py(py))
    }
}

#[pymethods]
impl PyMockExecutionConfig {
    #[new]
    #[pyo3(signature = (mocked_exchange=None, initial_state=None, latency_ms=0, fees_percent=0.0))]
    pub fn __new__(
        py: Python<'_>,
        mocked_exchange: Option<&PyExchangeId>,
        initial_state: Option<PyObject>,
        latency_ms: u64,
        fees_percent: f64,
    ) -> PyResult<Self> {
        if !fees_percent.is_finite() || fees_percent < 0.0 {
            return Err(PyValueError::new_err(
                "fees_percent must be a non-negative finite value",
            ));
        }

        let exchange = mocked_exchange
            .map(|value| value.as_inner())
            .unwrap_or(ExchangeId::Mock);

        let snapshot = Self::parse_snapshot(py, initial_state, exchange)?;
        let fees_percent = parse_decimal(fees_percent, "fees_percent")?;

        Ok(Self {
            inner: MockExecutionConfig::new(exchange, snapshot, latency_ms, fees_percent),
        })
    }

    #[getter]
    pub fn mocked_exchange(&self) -> PyExchangeId {
        PyExchangeId::from_inner(self.inner.mocked_exchange)
    }

    #[setter]
    pub fn set_mocked_exchange(&mut self, exchange: &PyExchangeId) {
        let inner = exchange.as_inner();
        self.inner.mocked_exchange = inner;
        self.inner.initial_state.exchange = inner;
    }

    #[getter]
    pub fn latency_ms(&self) -> u64 {
        self.inner.latency_ms
    }

    #[setter]
    pub fn set_latency_ms(&mut self, value: u64) {
        self.inner.latency_ms = value;
    }

    #[getter]
    pub fn fees_percent(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.inner.fees_percent)
    }

    #[setter]
    pub fn set_fees_percent(&mut self, value: f64) -> PyResult<()> {
        if !value.is_finite() || value < 0.0 {
            return Err(PyValueError::new_err(
                "fees_percent must be a non-negative finite value",
            ));
        }

        self.inner.fees_percent = parse_decimal(value, "fees_percent")?;
        Ok(())
    }

    #[getter]
    pub fn initial_state(&self, py: Python<'_>) -> PyResult<PyObject> {
        Self::snapshot_to_py(py, &self.inner.initial_state)
    }

    #[pyo3(signature = (state))]
    pub fn set_initial_state(&mut self, py: Python<'_>, state: PyObject) -> PyResult<()> {
        let exchange = self.inner.mocked_exchange;
        self.inner.initial_state = Self::parse_snapshot(py, Some(state), exchange)?;
        Ok(())
    }

    pub fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let serialized = serde_json::to_string(&self.inner)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let json = PyModule::import_bound(py, "json")?;
        let loads = json.getattr("loads")?;
        Ok(loads.call1((serialized,))?.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "MockExecutionConfig(exchange={}, latency_ms={}, fees_percent={})",
            self.inner.mocked_exchange.as_str(),
            self.inner.latency_ms,
            self.inner.fees_percent
        ))
    }
}

/// Python wrapper around [`ExecutionConfig`].
#[pyclass(module = "barter_python", name = "ExecutionConfig", unsendable)]
#[derive(Clone)]
pub struct PyExecutionConfig {
    pub(crate) inner: ExecutionConfig,
}

impl PyExecutionConfig {
    pub(crate) fn from_inner(inner: ExecutionConfig) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyExecutionConfig {
    #[classmethod]
    #[pyo3(signature = (config))]
    pub fn mock(_cls: &Bound<'_, PyType>, config: &PyMockExecutionConfig) -> Self {
        Self {
            inner: ExecutionConfig::Mock(config.inner.clone()),
        }
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        match self.inner {
            ExecutionConfig::Mock(_) => "mock",
        }
    }

    #[getter]
    pub fn mock_config(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner {
            ExecutionConfig::Mock(config) => {
                Py::new(py, PyMockExecutionConfig::from_inner(config.clone()))
                    .map(|value| value.into_py(py))
            }
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let serialized = serde_json::to_string(&self.inner)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let json = PyModule::import_bound(py, "json")?;
        let loads = json.getattr("loads")?;
        Ok(loads.call1((serialized,))?.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ExecutionConfig(kind='{}')", self.kind()))
    }
}

/// Python wrapper around [`SystemConfig`].
#[pyclass(module = "barter_python", name = "SystemConfig")]
#[derive(Clone)]
pub struct PySystemConfig {
    pub(crate) inner: SystemConfig,
}

impl PySystemConfig {
    pub(crate) fn clone_inner(&self) -> SystemConfig {
        self.inner.clone()
    }
}

#[pymethods]
impl PySystemConfig {
    /// Load a [`SystemConfig`] from a JSON file located at `path`.
    #[staticmethod]
    pub fn from_json(path: &str) -> PyResult<Self> {
        let reader = File::open(Path::new(path))
            .map(BufReader::new)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        let config = serde_json::from_reader(reader)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner: config })
    }

    /// Construct a [`SystemConfig`] from a JSON string.
    #[staticmethod]
    pub fn from_json_str(data: &str) -> PyResult<Self> {
        let config =
            serde_json::from_str(data).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner: config })
    }

    /// Construct a [`SystemConfig`] from a Python dictionary-like object.
    #[staticmethod]
    pub fn from_dict(py: Python<'_>, value: PyObject) -> PyResult<Self> {
        let json_module = PyModule::import_bound(py, "json")?;
        let dumps = json_module.getattr("dumps")?;
        let serialized: String = dumps.call1((value,))?.extract()?;

        let config = serde_json::from_str(&serialized)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner: config })
    }

    /// Return a dictionary describing the configured risk limits.
    pub fn risk_limits(&self, py: Python<'_>) -> PyResult<PyObject> {
        risk_configuration_to_py(py, &self.inner.risk)
    }

    /// Set or clear global risk limits.
    #[pyo3(signature = (limits=None))]
    pub fn set_global_risk_limits(
        &mut self,
        py: Python<'_>,
        limits: Option<PyObject>,
    ) -> PyResult<()> {
        let limits = parse_optional_limits(py, limits)?;
        self.inner
            .set_global_risk_limits(limits)
            .map_err(risk_error_to_py)
    }

    /// Set or clear per-instrument risk limits by instrument index.
    #[pyo3(signature = (index, limits=None))]
    pub fn set_instrument_risk_limits(
        &mut self,
        py: Python<'_>,
        index: usize,
        limits: Option<PyObject>,
    ) -> PyResult<()> {
        let limits = parse_optional_limits(py, limits)?;
        self.inner
            .set_instrument_risk_limits(index, limits)
            .map_err(risk_error_to_py)
    }

    /// Return the configured execution definitions.
    pub fn executions(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.inner
            .executions
            .iter()
            .map(|execution| {
                Py::new(py, PyExecutionConfig::from_inner(execution.clone()))
                    .map(|value| value.into_py(py))
            })
            .collect()
    }

    /// Append an execution configuration to the system configuration.
    pub fn add_execution(&mut self, execution: &PyExecutionConfig) {
        self.inner.executions.push(execution.inner.clone());
    }

    /// Remove all execution configurations from the system configuration.
    pub fn clear_executions(&mut self) {
        self.inner.executions.clear();
    }

    /// Retrieve per-instrument risk limits for the provided index.
    pub fn get_instrument_risk_limits(&self, py: Python<'_>, index: usize) -> PyResult<PyObject> {
        match self.inner.instrument_risk_limits(index) {
            Some(limits) => Ok(risk_limits_to_py(py, limits)?.into_py(py)),
            None => Ok(py.None()),
        }
    }

    /// Return the configuration as a Python dictionary.
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json = serde_json::to_string(&self.inner)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        let json_module = PyModule::import_bound(py, "json")?;
        let loads = json_module.getattr("loads")?;
        let dictionary = loads.call1((json,))?;

        Ok(dictionary.into_py(py))
    }

    /// Serialize the configuration to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.inner)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Serialize the configuration to `path` in JSON format.
    pub fn to_json_file(&self, path: &str) -> PyResult<()> {
        let file = File::create(Path::new(path))
            .map(BufWriter::new)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        serde_json::to_writer_pretty(file, &self.inner)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __repr__(&self) -> PyResult<String> {
        let risk_overrides = self.inner.risk.instruments.len();
        Ok(format!(
            "SystemConfig(instruments={}, executions={}, risk_overrides={})",
            self.inner.instruments.len(),
            self.inner.executions.len(),
            risk_overrides
        ))
    }
}

fn risk_configuration_to_py(py: Python<'_>, config: &RiskConfiguration) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    dict.set_item(
        "global",
        option_risk_limits_to_py(py, config.global.as_ref())?,
    )?;

    let entries = PyList::empty_bound(py);
    for RiskInstrumentLimits { index, limits } in &config.instruments {
        let entry = PyDict::new_bound(py);
        entry.set_item("index", *index)?;
        entry.set_item("limits", risk_limits_to_py(py, limits)?.into_py(py))?;
        entries.append(entry)?;
    }
    dict.set_item("instruments", entries)?;

    Ok(dict.into_py(py))
}

fn parse_optional_limits(py: Python<'_>, limits: Option<PyObject>) -> PyResult<Option<RiskLimits>> {
    match limits {
        Some(value) if !value.is_none(py) => {
            let bound = value.bind(py);
            Ok(Some(risk_limits_from_py(&bound.as_any())?))
        }
        _ => Ok(None),
    }
}

fn risk_limits_to_py(py: Python<'_>, limits: &RiskLimits) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item(
        "max_position_notional",
        option_decimal_to_py(py, limits.max_position_notional)?,
    )?;
    dict.set_item(
        "max_position_quantity",
        option_decimal_to_py(py, limits.max_position_quantity)?,
    )?;
    dict.set_item(
        "max_leverage",
        option_decimal_to_py(py, limits.max_leverage)?,
    )?;
    dict.set_item(
        "max_exposure_percent",
        option_decimal_to_py(py, limits.max_exposure_percent)?,
    )?;

    Ok(dict.into())
}

fn option_risk_limits_to_py(py: Python<'_>, limits: Option<&RiskLimits>) -> PyResult<PyObject> {
    match limits {
        Some(limits) => Ok(risk_limits_to_py(py, limits)?.into_py(py)),
        None => Ok(py.None()),
    }
}

fn risk_limits_from_py(dict: &Bound<'_, PyAny>) -> PyResult<RiskLimits> {
    let dict = dict.downcast::<PyDict>()?;
    let max_position_notional = optional_decimal_from_dict(&dict, "max_position_notional")?;
    let max_position_quantity = optional_decimal_from_dict(&dict, "max_position_quantity")?;
    let max_leverage = optional_decimal_from_dict(&dict, "max_leverage")?;
    let max_exposure_percent = optional_decimal_from_dict(&dict, "max_exposure_percent")?;

    Ok(RiskLimits {
        max_position_notional,
        max_position_quantity,
        max_leverage,
        max_exposure_percent,
    })
}

fn optional_decimal_from_dict(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<Decimal>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => decimal_from_py(&value).map(Some),
        _ => Ok(None),
    }
}

fn option_decimal_to_py(py: Python<'_>, value: Option<Decimal>) -> PyResult<PyObject> {
    match value {
        Some(decimal) => decimal_to_py(py, decimal),
        None => Ok(py.None()),
    }
}

fn decimal_to_py(py: Python<'_>, value: Decimal) -> PyResult<PyObject> {
    let module = PyModule::import_bound(py, "decimal")?;
    let decimal_cls = module.getattr("Decimal")?;
    let value_str = value.to_string();
    Ok(decimal_cls.call1((value_str,))?.into_py(py))
}

fn decimal_from_py(value: &Bound<'_, PyAny>) -> PyResult<Decimal> {
    let text: String = value.str()?.extract()?;
    text.parse::<Decimal>()
        .map_err(|err| PyValueError::new_err(err.to_string()))
}

fn risk_error_to_py(error: RiskLimitsError) -> PyErr {
    PyValueError::new_err(error.to_string())
}
