use crate::{command::parse_decimal, summary::PyMetricWithInterval};
use barter::statistic::{
    metric::{sharpe::SharpeRatio, sortino::SortinoRatio},
    time::{Annual252, Annual365, Daily, TimeInterval},
};
use chrono::TimeDelta;
use pyo3::{
    Bound,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDelta},
};
use rust_decimal::Decimal;

#[derive(Debug, Copy, Clone)]
enum IntervalChoice {
    Daily,
    Annual252,
    Annual365,
    Duration(TimeDelta),
}

fn ratio_to_metric<Interval>(
    py: Python<'_>,
    value: Decimal,
    interval: Interval,
) -> PyResult<Py<PyMetricWithInterval>>
where
    Interval: TimeInterval,
{
    let name = interval.name().to_string();
    PyMetricWithInterval::from_components(py, value, name)
}

fn sharpe_metric<Interval>(
    py: Python<'_>,
    ratio: SharpeRatio<Interval>,
) -> PyResult<Py<PyMetricWithInterval>>
where
    Interval: TimeInterval,
{
    let SharpeRatio { value, interval } = ratio;
    ratio_to_metric(py, value, interval)
}

fn sortino_metric<Interval>(
    py: Python<'_>,
    ratio: SortinoRatio<Interval>,
) -> PyResult<Py<PyMetricWithInterval>>
where
    Interval: TimeInterval,
{
    let SortinoRatio { value, interval } = ratio;
    ratio_to_metric(py, value, interval)
}

#[pyfunction]
#[pyo3(signature = (risk_free_return, mean_return, std_dev_returns, interval))]
pub fn calculate_sharpe_ratio(
    py: Python<'_>,
    risk_free_return: f64,
    mean_return: f64,
    std_dev_returns: f64,
    interval: &Bound<'_, PyAny>,
) -> PyResult<Py<PyMetricWithInterval>> {
    let risk_free = parse_decimal(risk_free_return, "risk_free_return")?;
    let mean = parse_decimal(mean_return, "mean_return")?;
    let deviation = parse_decimal(std_dev_returns, "std_dev_returns")?;
    let choice = parse_interval_choice(interval)?;

    match choice {
        IntervalChoice::Daily => sharpe_metric(
            py,
            SharpeRatio::calculate(risk_free, mean, deviation, Daily),
        ),
        IntervalChoice::Annual252 => sharpe_metric(
            py,
            SharpeRatio::calculate(risk_free, mean, deviation, Annual252),
        ),
        IntervalChoice::Annual365 => sharpe_metric(
            py,
            SharpeRatio::calculate(risk_free, mean, deviation, Annual365),
        ),
        IntervalChoice::Duration(delta) => sharpe_metric(
            py,
            SharpeRatio::calculate(risk_free, mean, deviation, delta),
        ),
    }
}

#[pyfunction]
#[pyo3(signature = (risk_free_return, mean_return, std_dev_loss_returns, interval))]
pub fn calculate_sortino_ratio(
    py: Python<'_>,
    risk_free_return: f64,
    mean_return: f64,
    std_dev_loss_returns: f64,
    interval: &Bound<'_, PyAny>,
) -> PyResult<Py<PyMetricWithInterval>> {
    let risk_free = parse_decimal(risk_free_return, "risk_free_return")?;
    let mean = parse_decimal(mean_return, "mean_return")?;
    let deviation = parse_decimal(std_dev_loss_returns, "std_dev_loss_returns")?;
    let choice = parse_interval_choice(interval)?;

    match choice {
        IntervalChoice::Daily => sortino_metric(
            py,
            SortinoRatio::calculate(risk_free, mean, deviation, Daily),
        ),
        IntervalChoice::Annual252 => sortino_metric(
            py,
            SortinoRatio::calculate(risk_free, mean, deviation, Annual252),
        ),
        IntervalChoice::Annual365 => sortino_metric(
            py,
            SortinoRatio::calculate(risk_free, mean, deviation, Annual365),
        ),
        IntervalChoice::Duration(delta) => sortino_metric(
            py,
            SortinoRatio::calculate(risk_free, mean, deviation, delta),
        ),
    }
}

fn parse_interval_choice(value: &Bound<'_, PyAny>) -> PyResult<IntervalChoice> {
    if let Ok(label) = value.extract::<String>() {
        return parse_interval_from_str(&label);
    }

    if value.is_instance_of::<PyDelta>() {
        let seconds: f64 = value.call_method0("total_seconds")?.extract()?;
        if !seconds.is_finite() {
            return Err(PyValueError::new_err(
                "interval timedelta must contain a finite duration",
            ));
        }

        let micros = seconds * 1_000_000.0;
        if !micros.is_finite() {
            return Err(PyValueError::new_err(
                "interval timedelta is too large for conversion",
            ));
        }

        let micros = micros.round();
        if micros < (i64::MIN as f64) || micros > (i64::MAX as f64) {
            return Err(PyValueError::new_err(
                "interval timedelta exceeds supported range",
            ));
        }

        let delta = TimeDelta::microseconds(micros as i64);
        return Ok(IntervalChoice::Duration(delta));
    }

    Err(PyValueError::new_err(
        "interval must be a string identifier or datetime.timedelta",
    ))
}

fn parse_interval_from_str(label: &str) -> PyResult<IntervalChoice> {
    let normalised = label.trim().to_ascii_lowercase();
    match normalised.as_str() {
        "daily" => Ok(IntervalChoice::Daily),
        "annual(252)" | "annual_252" | "annual-252" | "annual252" => Ok(IntervalChoice::Annual252),
        "annual(365)" | "annual_365" | "annual-365" | "annual365" => Ok(IntervalChoice::Annual365),
        other => Err(PyValueError::new_err(format!(
            "unsupported interval identifier: {other}",
        ))),
    }
}
