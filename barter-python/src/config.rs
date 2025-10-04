use barter::system::config::{
    RiskConfiguration, RiskInstrumentLimits, RiskLimits, RiskLimitsError, SystemConfig,
};
use pyo3::{
    Bound, PyObject,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict, PyList, PyModule},
};
use rust_decimal::Decimal;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

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
