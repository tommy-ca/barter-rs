use barter_integration::error::SocketError as IntegrationSocketError;
use pyo3::{
    PyErr, PyResult,
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyBytes, PyDict},
};

#[cfg(feature = "python-tests")]
use pyo3::exceptions::PyValueError;

create_exception!(barter_python, SocketError, PyException);

#[derive(Debug, Clone)]
enum SocketErrorDetails {
    Json { error: String, payload: String },
    JsonBinary { error: String, payload: Vec<u8> },
    Serialise { error: String },
    QueryParams { error: String },
    UrlEncoded { error: String },
    UrlParse { error: String },
    Subscribe { message: String },
    Terminated { reason: String },
    Unsupported { entity: String, item: String },
    WebSocket { error: String },
    Http { error: String },
    HttpTimeout { error: String },
    HttpResponse { status: u16, error: String },
    Unidentifiable { subscription_id: String },
    Exchange { message: String },
}

impl SocketErrorDetails {
    fn to_py_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new_bound(py);
        match self {
            Self::Json { error, payload } => {
                dict.set_item("error", error)?;
                dict.set_item("payload", payload)?;
            }
            Self::JsonBinary { error, payload } => {
                dict.set_item("error", error)?;
                dict.set_item("payload", PyBytes::new_bound(py, payload))?;
            }
            Self::Serialise { error }
            | Self::QueryParams { error }
            | Self::UrlEncoded { error }
            | Self::UrlParse { error }
            | Self::WebSocket { error }
            | Self::Http { error }
            | Self::HttpTimeout { error } => {
                dict.set_item("error", error)?;
            }
            Self::Subscribe { message }
            | Self::Terminated { reason: message }
            | Self::Exchange { message } => {
                dict.set_item("message", message)?;
            }
            Self::Unsupported { entity, item } => {
                dict.set_item("entity", entity)?;
                dict.set_item("item", item)?;
            }
            Self::HttpResponse { status, error } => {
                dict.set_item("status", *status as u64)?;
                dict.set_item("error", error)?;
            }
            Self::Unidentifiable { subscription_id } => {
                dict.set_item("subscription_id", subscription_id)?;
            }
        }
        Ok(dict.into())
    }
}

#[derive(Debug, Clone)]
struct SocketErrorInfoInner {
    kind: &'static str,
    message: String,
    details: Option<SocketErrorDetails>,
}

impl SocketErrorInfoInner {
    fn from_error(error: IntegrationSocketError) -> Self {
        use IntegrationSocketError::*;

        let message = error.to_string();
        let (kind, details) = match error {
            Sink => ("Sink", None),
            Deserialise { error, payload } => (
                "Deserialise",
                Some(SocketErrorDetails::Json {
                    error: error.to_string(),
                    payload,
                }),
            ),
            DeserialiseBinary { error, payload } => (
                "DeserialiseBinary",
                Some(SocketErrorDetails::JsonBinary {
                    error: error.to_string(),
                    payload,
                }),
            ),
            Serialise(error) => (
                "Serialise",
                Some(SocketErrorDetails::Serialise {
                    error: error.to_string(),
                }),
            ),
            QueryParams(error) => (
                "QueryParams",
                Some(SocketErrorDetails::QueryParams {
                    error: error.to_string(),
                }),
            ),
            UrlEncoded(error) => (
                "UrlEncoded",
                Some(SocketErrorDetails::UrlEncoded {
                    error: error.to_string(),
                }),
            ),
            UrlParse(error) => (
                "UrlParse",
                Some(SocketErrorDetails::UrlParse {
                    error: error.to_string(),
                }),
            ),
            Subscribe(message) => (
                "Subscribe",
                Some(SocketErrorDetails::Subscribe { message }),
            ),
            Terminated(reason) => (
                "Terminated",
                Some(SocketErrorDetails::Terminated { reason }),
            ),
            Unsupported { entity, item } => (
                "Unsupported",
                Some(SocketErrorDetails::Unsupported { entity, item }),
            ),
            WebSocket(error) => (
                "WebSocket",
                Some(SocketErrorDetails::WebSocket {
                    error: error.to_string(),
                }),
            ),
            Http(error) => (
                "Http",
                Some(SocketErrorDetails::Http {
                    error: error.to_string(),
                }),
            ),
            HttpTimeout(error) => (
                "HttpTimeout",
                Some(SocketErrorDetails::HttpTimeout {
                    error: error.to_string(),
                }),
            ),
            HttpResponse(status, error) => (
                "HttpResponse",
                Some(SocketErrorDetails::HttpResponse {
                    status: status.as_u16(),
                    error,
                }),
            ),
            Unidentifiable(id) => (
                "Unidentifiable",
                Some(SocketErrorDetails::Unidentifiable {
                    subscription_id: id.to_string(),
                }),
            ),
            Exchange(message) => (
                "Exchange",
                Some(SocketErrorDetails::Exchange { message }),
            ),
        };

        Self {
            kind,
            message,
            details,
        }
    }

    fn kind(&self) -> &'static str {
        self.kind
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn details(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        match &self.details {
            Some(details) => details.to_py_dict(py).map(Some),
            None => Ok(None),
        }
    }
}

#[pyclass(module = "barter_python", name = "SocketErrorInfo", unsendable)]
#[derive(Debug, Clone)]
pub struct PySocketErrorInfo {
    inner: SocketErrorInfoInner,
}

impl PySocketErrorInfo {
    pub fn from_socket_error(error: IntegrationSocketError) -> Self {
        Self {
            inner: SocketErrorInfoInner::from_error(error),
        }
    }
}

#[pymethods]
impl PySocketErrorInfo {
    #[getter]
    pub fn kind(&self) -> &str {
        self.inner.kind()
    }

    #[getter]
    pub fn message(&self) -> &str {
        self.inner.message()
    }

    #[getter]
    pub fn details(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        self.inner.details(py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("SocketErrorInfo(kind='{}')", self.inner.kind()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.inner.message().to_string())
    }
}

#[allow(dead_code)]
fn build_socket_error(py: Python<'_>, error: IntegrationSocketError) -> PyResult<PyErr> {
    let info = PySocketErrorInfo::from_socket_error(error);
    let info_obj = Py::new(py, info)?;

    let (kind, message, details) = {
        let info_ref = info_obj.borrow(py);
        let details = info_ref.details(py)?;
        (info_ref.kind().to_string(), info_ref.message().to_string(), details)
    };

    let args = (message.clone(), info_obj.clone_ref(py));
    let err = SocketError::new_err(args);
    let instance = err.to_object(py).into_bound(py);
    instance.setattr("info", info_obj.clone_ref(py))?;
    instance.setattr("kind", kind)?;
    instance.setattr("message", message)?;
    match details {
        Some(dict) => instance.setattr("details", dict.into_py(py))?,
        None => instance.setattr("details", py.None())?,
    }

    Ok(err)
}

#[allow(dead_code)]
pub fn socket_error_to_py_err(error: IntegrationSocketError) -> PyErr {
    Python::with_gil(|py| build_socket_error(py, error)).unwrap_or_else(|err| err)
}

#[cfg(feature = "python-tests")]
#[pyfunction]
pub fn _testing_raise_socket_error(kind: &str) -> PyResult<()> {
    let error = match kind {
        "subscribe" => IntegrationSocketError::Subscribe("subscription failed".to_string()),
        "deserialise_binary" => IntegrationSocketError::DeserialiseBinary {
            error: serde_json::from_slice::<serde_json::Value>(b"not-json").unwrap_err(),
            payload: vec![1, 2, 3],
        },
        other => {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "unsupported socket error test kind: {other}"
            )))
        }
    };

    Err(socket_error_to_py_err(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn details_for_subscribe_variant() {
        let error = IntegrationSocketError::Subscribe("subscription failed".to_string());
        let info = PySocketErrorInfo::from_socket_error(error);

        Python::with_gil(|py| {
            assert_eq!(info.kind(), "Subscribe");
            let details = info.details(py).unwrap().unwrap();
            let bound = details.bind(py);
            let message_obj = bound
                .get_item("message")
                .unwrap()
                .expect("message entry");
            let message: String = message_obj.extract().unwrap();
            assert_eq!(message, "subscription failed");
        });
    }
}
