use crate::command::parse_decimal;
use barter_execution::balance::Balance;
use barter_instrument::{
    Keyed,
    asset::{ExchangeAsset, name::AssetNameInternal},
    exchange::ExchangeId,
};
use pyo3::{Bound, PyObject, PyResult, Python, exceptions::PyValueError, types::PyDict};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SummaryInterval {
    Daily,
    Annual252,
    Annual365,
}

pub fn parse_summary_interval(value: Option<&str>) -> PyResult<SummaryInterval> {
    match value {
        None => Ok(SummaryInterval::Daily),
        Some(raw) => {
            let normalized: String = raw
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();

            match normalized.as_str() {
                "" => Ok(SummaryInterval::Daily),
                "daily" => Ok(SummaryInterval::Daily),
                "annual252" => Ok(SummaryInterval::Annual252),
                "annual365" => Ok(SummaryInterval::Annual365),
                _ => Err(PyValueError::new_err(format!(
                    "invalid interval '{raw}'. valid values are: daily, annual_252, annual_365",
                    raw = raw.trim()
                ))),
            }
        }
    }
}

pub fn parse_initial_balances(
    py: Python<'_>,
    values: Option<PyObject>,
) -> PyResult<Vec<Keyed<ExchangeAsset<AssetNameInternal>, Balance>>> {
    let Some(values) = values else {
        return Ok(Vec::new());
    };

    let values = values.bind(py);

    if values.is_none() {
        return Ok(Vec::new());
    }

    let items: Vec<PyObject> = values.extract()?;
    let mut results = Vec::with_capacity(items.len());

    for (index, item) in items.into_iter().enumerate() {
        let mapping = item.bind(py).downcast::<PyDict>().map_err(|_| {
            PyValueError::new_err(format!("initial_balances[{index}] must be a mapping"))
        })?;

        let entry = parse_initial_balance_entry(index, &mapping)?;
        results.push(entry);
    }

    Ok(results)
}

fn parse_initial_balance_entry(
    index: usize,
    mapping: &Bound<'_, PyDict>,
) -> PyResult<Keyed<ExchangeAsset<AssetNameInternal>, Balance>> {
    let exchange_value = mapping.get_item("exchange")?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "initial_balances[{index}] missing 'exchange' field"
        ))
    })?;

    let exchange_label: String = exchange_value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "initial_balances[{index}].exchange must be a string"
        ))
    })?;
    let exchange = parse_exchange_identifier(index, &exchange_label)?;

    let asset_value = mapping.get_item("asset")?.ok_or_else(|| {
        PyValueError::new_err(format!("initial_balances[{index}] missing 'asset' field"))
    })?;

    let asset_label: String = asset_value.extract().map_err(|_| {
        PyValueError::new_err(format!("initial_balances[{index}].asset must be a string"))
    })?;

    let total_value = mapping.get_item("total")?.ok_or_else(|| {
        PyValueError::new_err(format!("initial_balances[{index}] missing 'total' field"))
    })?;

    let total_label = format!("initial_balances[{index}].total");
    let total = parse_decimal(
        total_value
            .extract::<f64>()
            .map_err(|_| PyValueError::new_err(format!("{} must be numeric", total_label)))?,
        &total_label,
    )?;

    let free = match mapping.get_item("free")? {
        Some(value) => {
            let free_label = format!("initial_balances[{index}].free");
            parse_decimal(
                value.extract::<f64>().map_err(|_| {
                    PyValueError::new_err(format!("{} must be numeric", free_label))
                })?,
                &free_label,
            )?
        }
        None => total,
    };

    if free > total {
        return Err(PyValueError::new_err(format!(
            "initial_balances[{index}] free balance cannot exceed total"
        )));
    }

    let asset = AssetNameInternal::from(asset_label.as_str());
    let balance = Balance::new(total, free);

    Ok(Keyed::new(ExchangeAsset::new(exchange, asset), balance))
}

fn parse_exchange_identifier(index: usize, raw: &str) -> PyResult<ExchangeId> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(PyValueError::new_err(format!(
            "initial_balances[{index}].exchange must not be empty"
        )));
    }

    let normalized = normalized.to_ascii_lowercase();
    let quoted = format!("\"{}\"", normalized);

    serde_json::from_str::<ExchangeId>(&quoted).map_err(|_| {
        PyValueError::new_err(format!(
            "initial_balances[{index}].exchange '{raw}' is not a recognised exchange"
        ))
    })
}
