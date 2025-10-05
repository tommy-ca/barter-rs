use crate::{
    command::{
        DefaultOrderRequestCancel, DefaultOrderRequestOpen, PyOrderRequestCancel,
        PyOrderRequestOpen, parse_side,
    },
    instrument::PySide,
    summary::decimal_to_py,
};
use barter::risk::{DefaultRiskManager, RiskManager, RiskRefused as RustRiskRefused, check::util};
use barter_instrument::Side;
use pyo3::{
    Bound, PyAny, PyObject, PyResult, Python, exceptions::PyValueError, prelude::*, types::PyType,
};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RequestVariant {
    Cancel(DefaultOrderRequestCancel),
    Open(DefaultOrderRequestOpen),
}

impl RequestVariant {
    fn extract(value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(handle) = value.extract::<Py<PyOrderRequestOpen>>() {
            let borrowed = handle.borrow(value.py());
            return Ok(Self::Open(borrowed.clone_inner()));
        }

        if let Ok(handle) = value.extract::<Py<PyOrderRequestCancel>>() {
            let borrowed = handle.borrow(value.py());
            return Ok(Self::Cancel(borrowed.clone_inner()));
        }

        Err(PyValueError::new_err(
            "order request must be OrderRequestOpen or OrderRequestCancel",
        ))
    }

    fn to_py(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self {
            Self::Cancel(inner) => {
                let wrapper = PyOrderRequestCancel::from_inner(inner.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            }
            Self::Open(inner) => {
                let wrapper = PyOrderRequestOpen::from_inner(inner.clone());
                Py::new(py, wrapper).map(|value| value.into_py(py))
            }
        }
    }

    fn repr(&self, py: Python<'_>) -> PyResult<String> {
        let obj = self.to_py(py)?;
        obj.bind(py).repr()?.extract()
    }
}

#[pyclass(
    module = "barter_python",
    name = "RiskApproved",
    unsendable,
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyRiskApproved {
    request: RequestVariant,
}

impl PyRiskApproved {
    fn from_variant(request: RequestVariant) -> Self {
        Self { request }
    }
}

#[pymethods]
impl PyRiskApproved {
    #[new]
    #[pyo3(signature = (item))]
    pub fn __new__(item: &Bound<'_, PyAny>) -> PyResult<Self> {
        let request = RequestVariant::extract(item)?;
        Ok(Self { request })
    }

    #[getter]
    pub fn item(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.request.to_py(py)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_item(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.request.to_py(py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            self.request
                .repr(py)
                .map(|inner| format!("RiskApproved({inner})"))
        })
    }
}

#[pyclass(
    module = "barter_python",
    name = "RiskRefused",
    unsendable,
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyRiskRefused {
    request: RequestVariant,
    reason: String,
}

impl PyRiskRefused {
    fn from_parts(request: RequestVariant, reason: String) -> Self {
        Self { request, reason }
    }
}

#[pymethods]
impl PyRiskRefused {
    #[new]
    #[pyo3(signature = (item, reason))]
    pub fn __new__(item: &Bound<'_, PyAny>, reason: String) -> PyResult<Self> {
        let request = RequestVariant::extract(item)?;
        Ok(Self::from_parts(request, reason))
    }

    #[classmethod]
    #[pyo3(signature = (item, reason))]
    pub fn new(
        _cls: &Bound<'_, PyType>,
        item: &Bound<'_, PyAny>,
        reason: String,
    ) -> PyResult<Self> {
        Self::__new__(item, reason)
    }

    #[getter]
    pub fn item(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.request.to_py(py)
    }

    #[getter]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_item(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.request.to_py(py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            self.request
                .repr(py)
                .map(|inner| format!("RiskRefused(item={inner}, reason='{}')", self.reason))
        })
    }
}

#[pyclass(module = "barter_python", name = "DefaultRiskManager", unsendable)]
pub struct PyDefaultRiskManager {
    inner: DefaultRiskManager<PyObject>,
}

#[pymethods]
impl PyDefaultRiskManager {
    #[new]
    pub fn __new__() -> Self {
        Self {
            inner: DefaultRiskManager::default(),
        }
    }

    #[allow(clippy::type_complexity)]
    #[pyo3(signature = (state, cancels, opens))]
    pub fn check(
        &self,
        _py: Python<'_>,
        state: PyObject,
        cancels: &Bound<'_, PyAny>,
        opens: &Bound<'_, PyAny>,
    ) -> PyResult<(
        Vec<PyRiskApproved>,
        Vec<PyRiskApproved>,
        Vec<PyRiskRefused>,
        Vec<PyRiskRefused>,
    )> {
        let cancel_requests = collect_cancel_requests(cancels)?;
        let open_requests = collect_open_requests(opens)?;

        let (approved_cancels, approved_opens, refused_cancels, refused_opens) =
            self.inner
                .check(&state, cancel_requests.clone(), open_requests.clone());

        let approved_cancels = approved_cancels
            .into_iter()
            .map(|approved| {
                PyRiskApproved::from_variant(RequestVariant::Cancel(approved.into_item()))
            })
            .collect();

        let approved_opens = approved_opens
            .into_iter()
            .map(|approved| {
                PyRiskApproved::from_variant(RequestVariant::Open(approved.into_item()))
            })
            .collect();

        let refused_cancels = refused_cancels
            .into_iter()
            .map(|refused: RustRiskRefused<DefaultOrderRequestCancel>| {
                let RustRiskRefused { item, reason } = refused;
                PyRiskRefused::from_parts(RequestVariant::Cancel(item), reason)
            })
            .collect();

        let refused_opens = refused_opens
            .into_iter()
            .map(|refused: RustRiskRefused<DefaultOrderRequestOpen>| {
                let RustRiskRefused { item, reason } = refused;
                PyRiskRefused::from_parts(RequestVariant::Open(item), reason)
            })
            .collect();

        Ok((
            approved_cancels,
            approved_opens,
            refused_cancels,
            refused_opens,
        ))
    }
}

fn collect_cancel_requests(
    iterable: &Bound<'_, PyAny>,
) -> PyResult<Vec<DefaultOrderRequestCancel>> {
    let mut results = Vec::new();

    for item in iterable.iter()? {
        let value = item?;
        match RequestVariant::extract(&value)? {
            RequestVariant::Cancel(request) => results.push(request),
            RequestVariant::Open(_) => {
                return Err(PyValueError::new_err(
                    "expected cancel request in cancels iterable",
                ));
            }
        }
    }

    Ok(results)
}

fn collect_open_requests(iterable: &Bound<'_, PyAny>) -> PyResult<Vec<DefaultOrderRequestOpen>> {
    let mut results = Vec::new();

    for item in iterable.iter()? {
        let value = item?;
        match RequestVariant::extract(&value)? {
            RequestVariant::Open(request) => results.push(request),
            RequestVariant::Cancel(_) => {
                return Err(PyValueError::new_err(
                    "expected open request in opens iterable",
                ));
            }
        }
    }

    Ok(results)
}

#[pyfunction]
#[pyo3(signature = (quantity, price, contract_size))]
pub fn calculate_quote_notional(
    py: Python<'_>,
    quantity: &Bound<'_, PyAny>,
    price: &Bound<'_, PyAny>,
    contract_size: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let quantity = decimal_from_py(quantity, "quantity")?;
    let price = decimal_from_py(price, "price")?;
    let contract_size = decimal_from_py(contract_size, "contract_size")?;

    match util::calculate_quote_notional(quantity, price, contract_size) {
        Some(result) => decimal_to_py(py, result),
        None => Ok(py.None()),
    }
}

#[pyfunction]
#[pyo3(signature = (current, other))]
pub fn calculate_abs_percent_difference(
    py: Python<'_>,
    current: &Bound<'_, PyAny>,
    other: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let current = decimal_from_py(current, "current")?;
    let other = decimal_from_py(other, "other")?;

    match util::calculate_abs_percent_difference(current, other) {
        Some(result) => decimal_to_py(py, result),
        None => Ok(py.None()),
    }
}

#[pyfunction]
#[pyo3(signature = (instrument_delta, contract_size, side, quantity_in_kind))]
pub fn calculate_delta(
    py: Python<'_>,
    instrument_delta: &Bound<'_, PyAny>,
    contract_size: &Bound<'_, PyAny>,
    side: &Bound<'_, PyAny>,
    quantity_in_kind: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let instrument_delta = decimal_from_py(instrument_delta, "instrument_delta")?;
    let contract_size = decimal_from_py(contract_size, "contract_size")?;
    let side = side_from_py(side)?;
    let quantity = decimal_from_py(quantity_in_kind, "quantity_in_kind")?;

    let result = util::calculate_delta(instrument_delta, contract_size, side, quantity);

    decimal_to_py(py, result)
}

fn decimal_from_py(value: &Bound<'_, PyAny>, field: &str) -> PyResult<Decimal> {
    let mut text: String = value.str()?.extract()?;
    if text.contains(['e', 'E']) {
        text = value
            .call_method1("__format__", ("f",))?
            .extract::<String>()?;
    }

    text.parse::<Decimal>().map_err(|err| {
        PyValueError::new_err(format!("{field} must be a Decimal-compatible value: {err}"))
    })
}

fn side_from_py(value: &Bound<'_, PyAny>) -> PyResult<Side> {
    if let Ok(handle) = value.extract::<Py<PySide>>() {
        let borrowed = handle.borrow(value.py());
        Ok(borrowed.inner())
    } else {
        let text: String = value.str()?.extract()?;
        parse_side(&text)
    }
}
