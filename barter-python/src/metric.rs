#![allow(clippy::too_many_arguments)]

use barter_integration::metric::Value;
use pyo3::{prelude::*, pyclass::CompareOp};

/// Python wrapper for [`Metric`].
#[pyclass(module = "barter_python", name = "Metric", unsendable)]
#[derive(Debug, Clone)]
pub struct PyMetric {
    name: String,
    time: u64,
    tags: Vec<PyTag>,
    fields: Vec<PyField>,
}

#[pymethods]
impl PyMetric {
    /// Create a new [`Metric`].
    #[new]
    #[pyo3(signature = (name, time, tags, fields))]
    pub fn new(name: String, time: u64, tags: Vec<PyTag>, fields: Vec<PyField>) -> PyResult<Self> {
        Ok(Self {
            name,
            time,
            tags,
            fields,
        })
    }

    /// Metric name.
    #[getter]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Milliseconds since the Unix epoch.
    #[getter]
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Key-Value pairs to categorise the Metric.
    #[getter]
    pub fn tags(&self) -> Vec<PyTag> {
        self.tags.clone()
    }

    /// Observed measurements.
    #[getter]
    pub fn fields(&self) -> Vec<PyField> {
        self.fields.clone()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Metric(name='{}', time={}, tags=[...], fields=[...])",
            self.name, self.time
        ))
    }
}

/// Python wrapper for [`Tag`].
#[pyclass(module = "barter_python", name = "Tag", unsendable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyTag {
    key: String,
    value: String,
}

#[pymethods]
impl PyTag {
    /// Create a new [`Tag`].
    #[new]
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    /// Tag key.
    #[getter]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Tag value.
    #[getter]
    pub fn value(&self) -> &str {
        &self.value
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Tag(key='{}', value='{}')", self.key, self.value))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Lt => Ok(self < other),
            CompareOp::Le => Ok(self <= other),
            CompareOp::Gt => Ok(self > other),
            CompareOp::Ge => Ok(self >= other),
        }
    }
}

/// Python wrapper for [`Field`].
#[pyclass(module = "barter_python", name = "Field", unsendable)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PyField {
    key: String,
    value: PyValue,
}

#[pymethods]
impl PyField {
    /// Create a new [`Field`].
    #[new]
    pub fn new(key: String, value: PyValue) -> Self {
        Self { key, value }
    }

    /// Field key.
    #[getter]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Field value.
    #[getter]
    pub fn value(&self) -> PyValue {
        self.value.clone()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Field(key='{}', value={:?})", self.key, self.value))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Lt => Ok(self < other),
            CompareOp::Le => Ok(self <= other),
            CompareOp::Gt => Ok(self > other),
            CompareOp::Ge => Ok(self >= other),
        }
    }
}

/// Python wrapper for [`Value`].
#[pyclass(module = "barter_python", name = "Value", unsendable)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PyValue {
    inner: Value,
}

#[pymethods]
impl PyValue {
    /// Create a float [`Value`].
    #[staticmethod]
    pub fn float(value: f64) -> Self {
        Self {
            inner: Value::Float(value),
        }
    }

    /// Create an int [`Value`].
    #[staticmethod]
    pub fn int(value: i64) -> Self {
        Self {
            inner: Value::Int(value),
        }
    }

    /// Create a uint [`Value`].
    #[staticmethod]
    pub fn uint(value: u64) -> Self {
        Self {
            inner: Value::UInt(value),
        }
    }

    /// Create a bool [`Value`].
    #[staticmethod]
    pub fn bool(value: bool) -> Self {
        Self {
            inner: Value::Bool(value),
        }
    }

    /// Create a string [`Value`].
    #[staticmethod]
    pub fn string(value: String) -> Self {
        Self {
            inner: Value::String(value),
        }
    }

    /// Check if this is a float value.
    pub fn is_float(&self) -> bool {
        matches!(self.inner, Value::Float(_))
    }

    /// Check if this is an int value.
    pub fn is_int(&self) -> bool {
        matches!(self.inner, Value::Int(_))
    }

    /// Check if this is a uint value.
    pub fn is_uint(&self) -> bool {
        matches!(self.inner, Value::UInt(_))
    }

    /// Check if this is a bool value.
    pub fn is_bool(&self) -> bool {
        matches!(self.inner, Value::Bool(_))
    }

    /// Check if this is a string value.
    pub fn is_string(&self) -> bool {
        matches!(self.inner, Value::String(_))
    }

    /// Get the float value if it is one.
    pub fn as_float(&self) -> PyResult<f64> {
        match &self.inner {
            Value::Float(v) => Ok(*v),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Value is not a float",
            )),
        }
    }

    /// Get the int value if it is one.
    pub fn as_int(&self) -> PyResult<i64> {
        match &self.inner {
            Value::Int(v) => Ok(*v),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Value is not an int",
            )),
        }
    }

    /// Get the uint value if it is one.
    pub fn as_uint(&self) -> PyResult<u64> {
        match &self.inner {
            Value::UInt(v) => Ok(*v),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Value is not a uint",
            )),
        }
    }

    /// Get the bool value if it is one.
    pub fn as_bool(&self) -> PyResult<bool> {
        match &self.inner {
            Value::Bool(v) => Ok(*v),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Value is not a bool",
            )),
        }
    }

    /// Get the string value if it is one.
    pub fn as_string(&self) -> PyResult<&str> {
        match &self.inner {
            Value::String(v) => Ok(v),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Value is not a string",
            )),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            CompareOp::Lt => Ok(self.inner < other.inner),
            CompareOp::Le => Ok(self.inner <= other.inner),
            CompareOp::Gt => Ok(self.inner > other.inner),
            CompareOp::Ge => Ok(self.inner >= other.inner),
        }
    }
}
