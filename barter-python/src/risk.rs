use crate::{command::parse_side, instrument::PySide, summary::decimal_to_py};
use barter::risk::check::util;
use barter_instrument::Side;
use pyo3::{Bound, PyAny, PyResult, exceptions::PyValueError, prelude::*};
use rust_decimal::Decimal;

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
