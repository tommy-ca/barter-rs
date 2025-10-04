use crate::{
    command::parse_decimal,
    summary::{PyDrawdown, PyMeanDrawdown, PyMetricWithInterval, decimal_to_py},
};
use barter::{
    Timed,
    statistic::{
        metric::{
            calmar::CalmarRatio,
            drawdown::{
                Drawdown, DrawdownGenerator,
                max::{MaxDrawdown, MaxDrawdownGenerator},
                mean::MeanDrawdownGenerator,
            },
            profit_factor::ProfitFactor,
            rate_of_return::RateOfReturn,
            sharpe::SharpeRatio,
            sortino::SortinoRatio,
            win_rate::WinRate,
        },
        time::{Annual252, Annual365, Daily, TimeInterval},
    },
};
use chrono::{DateTime, NaiveDateTime, TimeDelta, TimeZone, Utc};
use pyo3::{
    Bound, PyObject,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDelta, PySequence},
};
use rust_decimal::Decimal;
use std::str::FromStr;

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

fn calmar_metric<Interval>(
    py: Python<'_>,
    ratio: CalmarRatio<Interval>,
) -> PyResult<Py<PyMetricWithInterval>>
where
    Interval: TimeInterval,
{
    let CalmarRatio { value, interval } = ratio;
    ratio_to_metric(py, value, interval)
}

fn rate_metric<Interval>(
    py: Python<'_>,
    rate: RateOfReturn<Interval>,
    target_choice: Option<IntervalChoice>,
) -> PyResult<Py<PyMetricWithInterval>>
where
    Interval: TimeInterval,
{
    match target_choice {
        Some(IntervalChoice::Daily) => {
            let scaled = rate.scale(Daily);
            let RateOfReturn { value, interval } = scaled;
            ratio_to_metric(py, value, interval)
        }
        Some(IntervalChoice::Annual252) => {
            let scaled = rate.scale(Annual252);
            let RateOfReturn { value, interval } = scaled;
            ratio_to_metric(py, value, interval)
        }
        Some(IntervalChoice::Annual365) => {
            let scaled = rate.scale(Annual365);
            let RateOfReturn { value, interval } = scaled;
            ratio_to_metric(py, value, interval)
        }
        Some(IntervalChoice::Duration(delta)) => {
            let scaled = rate.scale(delta);
            let RateOfReturn { value, interval } = scaled;
            ratio_to_metric(py, value, interval)
        }
        None => {
            let RateOfReturn { value, interval } = rate;
            ratio_to_metric(py, value, interval)
        }
    }
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

#[pyfunction]
#[pyo3(signature = (risk_free_return, mean_return, max_drawdown, interval))]
pub fn calculate_calmar_ratio(
    py: Python<'_>,
    risk_free_return: f64,
    mean_return: f64,
    max_drawdown: f64,
    interval: &Bound<'_, PyAny>,
) -> PyResult<Py<PyMetricWithInterval>> {
    let risk_free = parse_decimal(risk_free_return, "risk_free_return")?;
    let mean = parse_decimal(mean_return, "mean_return")?;
    let drawdown = parse_decimal(max_drawdown, "max_drawdown")?;
    let choice = parse_interval_choice(interval)?;

    match choice {
        IntervalChoice::Daily => {
            calmar_metric(py, CalmarRatio::calculate(risk_free, mean, drawdown, Daily))
        }
        IntervalChoice::Annual252 => calmar_metric(
            py,
            CalmarRatio::calculate(risk_free, mean, drawdown, Annual252),
        ),
        IntervalChoice::Annual365 => calmar_metric(
            py,
            CalmarRatio::calculate(risk_free, mean, drawdown, Annual365),
        ),
        IntervalChoice::Duration(delta) => {
            calmar_metric(py, CalmarRatio::calculate(risk_free, mean, drawdown, delta))
        }
    }
}

#[pyfunction]
#[pyo3(signature = (profits_gross_abs, losses_gross_abs))]
pub fn calculate_profit_factor(
    py: Python<'_>,
    profits_gross_abs: f64,
    losses_gross_abs: f64,
) -> PyResult<Option<PyObject>> {
    let profits = parse_decimal(profits_gross_abs, "profits_gross_abs")?;
    let losses = parse_decimal(losses_gross_abs, "losses_gross_abs")?;

    let factor = ProfitFactor::calculate(profits, losses);
    factor
        .map(|metric| decimal_to_py(py, metric.value))
        .transpose()
}

#[pyfunction]
#[pyo3(signature = (wins, total))]
pub fn calculate_win_rate(py: Python<'_>, wins: f64, total: f64) -> PyResult<Option<PyObject>> {
    let wins_decimal = parse_decimal(wins, "wins")?;
    let total_decimal = parse_decimal(total, "total")?;

    if total_decimal.is_sign_negative() {
        return Err(PyValueError::new_err("total must be non-negative"));
    }

    if wins_decimal.is_sign_negative() {
        return Err(PyValueError::new_err("wins must be non-negative"));
    }

    let rate = WinRate::calculate(wins_decimal, total_decimal);
    rate.map(|metric| decimal_to_py(py, metric.value))
        .transpose()
}

#[pyfunction]
#[pyo3(signature = (mean_return, interval, target_interval=None))]
pub fn calculate_rate_of_return(
    py: Python<'_>,
    mean_return: f64,
    interval: &Bound<'_, PyAny>,
    target_interval: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyMetricWithInterval>> {
    let mean = parse_decimal(mean_return, "mean_return")?;
    let base_choice = parse_interval_choice(interval)?;
    let target_choice = target_interval
        .map(|value| parse_interval_choice(value))
        .transpose()?;

    match base_choice {
        IntervalChoice::Daily => {
            rate_metric(py, RateOfReturn::calculate(mean, Daily), target_choice)
        }
        IntervalChoice::Annual252 => {
            rate_metric(py, RateOfReturn::calculate(mean, Annual252), target_choice)
        }
        IntervalChoice::Annual365 => {
            rate_metric(py, RateOfReturn::calculate(mean, Annual365), target_choice)
        }
        IntervalChoice::Duration(delta) => {
            rate_metric(py, RateOfReturn::calculate(mean, delta), target_choice)
        }
    }
}

fn parse_datetime_point(value: &Bound<'_, PyAny>, index: usize) -> PyResult<DateTime<Utc>> {
    if let Ok(datetime) = value.extract::<DateTime<Utc>>() {
        return Ok(datetime);
    }

    if let Ok(naive) = value.extract::<NaiveDateTime>() {
        return Ok(Utc.from_utc_datetime(&naive));
    }

    Err(PyValueError::new_err(format!(
        "points[{index}].time must be a datetime.datetime instance",
    )))
}

fn parse_numeric_value(value: &Bound<'_, PyAny>, field: &str) -> PyResult<Decimal> {
    if let Ok(number) = value.extract::<f64>() {
        return parse_decimal(number, field);
    }

    let binding = value
        .str()
        .map_err(|_| PyValueError::new_err(format!("{field} must be numeric")))?;
    let value_str = binding
        .to_str()
        .map_err(|_| PyValueError::new_err(format!("{field} must be numeric")))?;

    Decimal::from_str(value_str)
        .map_err(|_| PyValueError::new_err(format!("{field} must be a finite numeric value")))
}

fn parse_equity_points(points: &Bound<'_, PyAny>) -> PyResult<Vec<Timed<Decimal>>> {
    let sequence = points.downcast::<PySequence>().map_err(|_| {
        PyValueError::new_err("points must be an iterable of (datetime, value) pairs")
    })?;

    let length = usize::try_from(sequence.len()?).unwrap_or(0);
    let mut parsed = Vec::with_capacity(length);

    for index in 0..length {
        let item = sequence.get_item(index)?;
        let pair = item.downcast::<PySequence>().map_err(|_| {
            PyValueError::new_err(format!(
                "points[{index}] must be an iterable of length 2 (datetime, value)",
            ))
        })?;

        if pair.len()? != 2 {
            return Err(PyValueError::new_err(format!(
                "points[{index}] must contain exactly two elements (datetime, value)",
            )));
        }

        let time = pair.get_item(0)?;
        let value = pair.get_item(1)?;

        let timestamp = parse_datetime_point(&time, index)?;
        let decimal_value = parse_numeric_value(&value, &format!("points[{index}].value"))?;

        parsed.push(Timed::new(decimal_value, timestamp));
    }

    Ok(parsed)
}

fn build_drawdown_series(points: Vec<Timed<Decimal>>) -> Vec<Drawdown> {
    let mut generator = DrawdownGenerator::default();
    let mut drawdowns = Vec::new();

    for point in points {
        if let Some(drawdown) = generator.update(point) {
            drawdowns.push(drawdown);
        }
    }

    if let Some(drawdown) = generator.generate() {
        drawdowns.push(drawdown);
    }

    drawdowns
}

fn drawdown_series_from_py(points: &Bound<'_, PyAny>) -> PyResult<Vec<Drawdown>> {
    let parsed = parse_equity_points(points)?;
    if parsed.is_empty() {
        return Ok(Vec::new());
    }

    Ok(build_drawdown_series(parsed))
}

#[pyfunction]
#[pyo3(signature = (points))]
pub fn generate_drawdown_series(
    py: Python<'_>,
    points: &Bound<'_, PyAny>,
) -> PyResult<Vec<Py<PyDrawdown>>> {
    let drawdowns = drawdown_series_from_py(points)?;

    drawdowns
        .into_iter()
        .map(|drawdown| PyDrawdown::from_drawdown(py, drawdown))
        .collect::<PyResult<Vec<_>>>()
}

#[pyfunction]
#[pyo3(signature = (points))]
pub fn calculate_max_drawdown(
    py: Python<'_>,
    points: &Bound<'_, PyAny>,
) -> PyResult<Option<Py<PyDrawdown>>> {
    let drawdowns = drawdown_series_from_py(points)?;

    if drawdowns.is_empty() {
        return Ok(None);
    }

    let mut generator: Option<MaxDrawdownGenerator> = None;
    for drawdown in &drawdowns {
        match generator.as_mut() {
            Some(existing) => existing.update(drawdown),
            None => generator = Some(MaxDrawdownGenerator::init(drawdown.clone())),
        }
    }

    match generator.and_then(|generator_state| generator_state.generate()) {
        Some(MaxDrawdown(drawdown)) => PyDrawdown::from_drawdown(py, drawdown).map(Some),
        None => Ok(None),
    }
}

#[pyfunction]
#[pyo3(signature = (points))]
pub fn calculate_mean_drawdown(
    py: Python<'_>,
    points: &Bound<'_, PyAny>,
) -> PyResult<Option<Py<PyMeanDrawdown>>> {
    let drawdowns = drawdown_series_from_py(points)?;

    if drawdowns.is_empty() {
        return Ok(None);
    }

    let mut generator: Option<MeanDrawdownGenerator> = None;
    for drawdown in &drawdowns {
        match generator.as_mut() {
            Some(existing) => existing.update(drawdown),
            None => generator = Some(MeanDrawdownGenerator::init(drawdown.clone())),
        }
    }

    match generator.and_then(|generator_state| generator_state.generate()) {
        Some(mean) => PyMeanDrawdown::from_mean(py, mean).map(Some),
        None => Ok(None),
    }
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
