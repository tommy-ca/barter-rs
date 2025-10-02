use barter::system::config::SystemConfig;
use pyo3::{PyObject, exceptions::PyValueError, prelude::*, types::PyModule};
use std::{fs::File, io::BufReader, path::Path};

/// Python wrapper around [`SystemConfig`].
#[pyclass(module = "barter_python", name = "SystemConfig")]
#[derive(Clone)]
pub struct PySystemConfig {
    pub(crate) inner: SystemConfig,
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

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "SystemConfig(instruments={}, executions={})",
            self.inner.instruments.len(),
            self.inner.executions.len()
        ))
    }
}
