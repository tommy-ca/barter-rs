use barter::engine::state::instrument::filter::InstrumentFilter;
use barter_execution::order::request::{
    OrderRequestCancel, OrderRequestOpen, RequestCancel, RequestOpen,
};
use barter_execution::order::{
    OrderKey, OrderKind, TimeInForce,
    id::{ClientOrderId, OrderId, StrategyId},
};
use barter_instrument::{
    Side, Underlying, asset::AssetIndex, exchange::ExchangeIndex, instrument::InstrumentIndex,
};
use barter_integration::collection::one_or_many::OneOrMany;
use pyo3::{Py, Python, exceptions::PyValueError, prelude::*};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub type DefaultOrderKey = OrderKey<ExchangeIndex, InstrumentIndex>;
pub type DefaultOrderRequestOpen = OrderRequestOpen<ExchangeIndex, InstrumentIndex>;
pub type DefaultOrderRequestCancel = OrderRequestCancel<ExchangeIndex, InstrumentIndex>;
pub type DefaultInstrumentFilter = InstrumentFilter<ExchangeIndex, AssetIndex, InstrumentIndex>;

#[pyclass(module = "barter_python", name = "OrderKey", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderKey {
    pub(crate) inner: DefaultOrderKey,
}

impl PyOrderKey {
    pub(crate) fn clone_inner(&self) -> DefaultOrderKey {
        self.inner.clone()
    }
}

#[pymethods]
impl PyOrderKey {
    #[new]
    #[pyo3(signature = (exchange, instrument, strategy_id, client_order_id=None))]
    pub fn new(
        exchange: usize,
        instrument: usize,
        strategy_id: &str,
        client_order_id: Option<&str>,
    ) -> Self {
        let exchange = ExchangeIndex(exchange);
        let instrument = InstrumentIndex(instrument);
        let strategy = StrategyId::new(strategy_id);
        let cid = client_order_id
            .map(ClientOrderId::new)
            .unwrap_or_else(ClientOrderId::random);

        Self {
            inner: OrderKey {
                exchange,
                instrument,
                strategy,
                cid,
            },
        }
    }

    #[getter]
    pub fn exchange(&self) -> usize {
        self.inner.exchange.index()
    }

    #[getter]
    pub fn instrument(&self) -> usize {
        self.inner.instrument.index()
    }

    #[getter]
    pub fn strategy_id(&self) -> String {
        self.inner.strategy.0.to_string()
    }

    #[getter]
    pub fn client_order_id(&self) -> String {
        self.inner.cid.0.to_string()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "OrderKey(exchange={}, instrument={}, strategy_id='{}', client_order_id='{}')",
            self.exchange(),
            self.instrument(),
            self.strategy_id(),
            self.client_order_id(),
        ))
    }
}

#[pyclass(module = "barter_python", name = "OrderRequestOpen", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderRequestOpen {
    pub(crate) inner: DefaultOrderRequestOpen,
}

impl PyOrderRequestOpen {
    pub(crate) fn clone_inner(&self) -> DefaultOrderRequestOpen {
        self.inner.clone()
    }
}

#[pymethods]
impl PyOrderRequestOpen {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (key, side, price, quantity, kind="limit", time_in_force=None, post_only=None))]
    pub fn new(
        key: &PyOrderKey,
        side: &str,
        price: f64,
        quantity: f64,
        kind: &str,
        time_in_force: Option<&str>,
        post_only: Option<bool>,
    ) -> PyResult<Self> {
        let side = parse_side(side)?;
        let kind = parse_order_kind(kind)?;
        let price = parse_decimal(price, "price")?;
        let quantity = parse_decimal(quantity, "quantity")?;
        let time_in_force = parse_time_in_force(time_in_force, post_only)?;

        let request = OrderRequestOpen {
            key: key.clone_inner(),
            state: RequestOpen {
                side,
                price,
                quantity,
                kind,
                time_in_force,
            },
        };

        Ok(Self { inner: request })
    }

    #[getter]
    pub fn side(&self) -> &'static str {
        match self.inner.state.side {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }

    #[getter]
    pub fn price(&self) -> String {
        self.inner.state.price.to_string()
    }

    #[getter]
    pub fn quantity(&self) -> String {
        self.inner.state.quantity.to_string()
    }

    #[getter]
    pub fn kind(&self) -> &'static str {
        match self.inner.state.kind {
            OrderKind::Market => "market",
            OrderKind::Limit => "limit",
        }
    }

    #[getter]
    pub fn time_in_force(&self) -> &'static str {
        match self.inner.state.time_in_force {
            TimeInForce::GoodUntilCancelled { .. } => "good_until_cancelled",
            TimeInForce::GoodUntilEndOfDay => "good_until_end_of_day",
            TimeInForce::FillOrKill => "fill_or_kill",
            TimeInForce::ImmediateOrCancel => "immediate_or_cancel",
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "OrderRequestOpen(side='{}', price={}, quantity={}, kind='{}', time_in_force='{}')",
            self.side(),
            self.price(),
            self.quantity(),
            self.kind(),
            self.time_in_force(),
        ))
    }
}

#[pyclass(module = "barter_python", name = "OrderRequestCancel", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderRequestCancel {
    pub(crate) inner: DefaultOrderRequestCancel,
}

impl PyOrderRequestCancel {
    pub(crate) fn clone_inner(&self) -> DefaultOrderRequestCancel {
        self.inner.clone()
    }
}

#[pymethods]
impl PyOrderRequestCancel {
    #[new]
    #[pyo3(signature = (key, order_id=None))]
    pub fn new(key: &PyOrderKey, order_id: Option<&str>) -> PyResult<Self> {
        let id = order_id.map(OrderId::new);
        let request = OrderRequestCancel {
            key: key.clone_inner(),
            state: RequestCancel { id },
        };

        Ok(Self { inner: request })
    }

    #[getter]
    pub fn has_order_id(&self) -> bool {
        self.inner.state.id.is_some()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(match &self.inner.state.id {
            Some(id) => format!("OrderRequestCancel(order_id='{}')", id.0),
            None => "OrderRequestCancel(order_id=None)".to_string(),
        })
    }
}

#[pyclass(module = "barter_python", name = "InstrumentFilter", unsendable)]
#[derive(Debug, Clone)]
pub struct PyInstrumentFilter {
    pub(crate) inner: DefaultInstrumentFilter,
}

impl PyInstrumentFilter {
    pub(crate) fn clone_inner(&self) -> DefaultInstrumentFilter {
        self.inner.clone()
    }
}

#[pymethods]
impl PyInstrumentFilter {
    #[staticmethod]
    pub fn none() -> Self {
        Self {
            inner: InstrumentFilter::None,
        }
    }

    #[staticmethod]
    pub fn exchanges(indices: Vec<usize>) -> PyResult<Self> {
        ensure_non_empty(&indices, "InstrumentFilter.exchanges")?;
        Ok(Self {
            inner: InstrumentFilter::exchanges(indices.into_iter().map(ExchangeIndex)),
        })
    }

    #[staticmethod]
    pub fn instruments(indices: Vec<usize>) -> PyResult<Self> {
        ensure_non_empty(&indices, "InstrumentFilter.instruments")?;
        Ok(Self {
            inner: InstrumentFilter::instruments(indices.into_iter().map(InstrumentIndex)),
        })
    }

    #[staticmethod]
    pub fn underlyings(pairs: Vec<(usize, usize)>) -> PyResult<Self> {
        ensure_non_empty(&pairs, "InstrumentFilter.underlyings")?;

        let underlyings = pairs.into_iter().map(|(base, quote)| Underlying {
            base: AssetIndex(base),
            quote: AssetIndex(quote),
        });

        Ok(Self {
            inner: InstrumentFilter::underlyings(underlyings),
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(match &self.inner {
            InstrumentFilter::None => "InstrumentFilter(None)".to_string(),
            InstrumentFilter::Exchanges(indices) => format!(
                "InstrumentFilter(Exchanges={:?})",
                indices
                    .as_ref()
                    .iter()
                    .map(|idx| idx.index())
                    .collect::<Vec<_>>()
            ),
            InstrumentFilter::Instruments(indices) => format!(
                "InstrumentFilter(Instruments={:?})",
                indices
                    .as_ref()
                    .iter()
                    .map(|idx| idx.index())
                    .collect::<Vec<_>>()
            ),
            InstrumentFilter::Underlyings(underlyings) => format!(
                "InstrumentFilter(Underlyings={:?})",
                underlyings
                    .as_ref()
                    .iter()
                    .map(|u| (u.base.index(), u.quote.index()))
                    .collect::<Vec<_>>()
            ),
        })
    }
}

pub(crate) fn collect_open_requests(
    py: Python<'_>,
    requests: Vec<Py<PyOrderRequestOpen>>,
) -> PyResult<OneOrMany<DefaultOrderRequestOpen>> {
    let inner = requests
        .into_iter()
        .map(|handle| -> PyResult<_> {
            let borrowed = handle.borrow(py);
            Ok(borrowed.clone_inner())
        })
        .collect::<PyResult<Vec<_>>>()?;

    to_one_or_many(inner, "EngineEvent.send_open_requests")
}

pub(crate) fn collect_cancel_requests(
    py: Python<'_>,
    requests: Vec<Py<PyOrderRequestCancel>>,
) -> PyResult<OneOrMany<DefaultOrderRequestCancel>> {
    let inner = requests
        .into_iter()
        .map(|handle| -> PyResult<_> {
            let borrowed = handle.borrow(py);
            Ok(borrowed.clone_inner())
        })
        .collect::<PyResult<Vec<_>>>()?;

    to_one_or_many(inner, "EngineEvent.send_cancel_requests")
}

pub(crate) fn clone_filter(filter: Option<&PyInstrumentFilter>) -> DefaultInstrumentFilter {
    filter
        .map(PyInstrumentFilter::clone_inner)
        .unwrap_or(InstrumentFilter::None)
}

fn parse_side(value: &str) -> PyResult<Side> {
    match value.to_ascii_lowercase().as_str() {
        "buy" | "b" => Ok(Side::Buy),
        "sell" | "s" => Ok(Side::Sell),
        other => Err(PyValueError::new_err(format!("invalid side: {other}"))),
    }
}

fn parse_order_kind(value: &str) -> PyResult<OrderKind> {
    match value.to_ascii_lowercase().as_str() {
        "market" | "mkt" => Ok(OrderKind::Market),
        "limit" | "lmt" => Ok(OrderKind::Limit),
        other => Err(PyValueError::new_err(format!(
            "invalid order kind: {other}"
        ))),
    }
}

fn parse_decimal(value: f64, field: &str) -> PyResult<Decimal> {
    Decimal::from_f64(value)
        .ok_or_else(|| PyValueError::new_err(format!("{field} must be a finite numeric value")))
}

fn parse_time_in_force(value: Option<&str>, post_only: Option<bool>) -> PyResult<TimeInForce> {
    match value.map(|val| val.to_ascii_lowercase()) {
        None => Ok(TimeInForce::GoodUntilCancelled {
            post_only: post_only.unwrap_or(false),
        }),
        Some(ref v)
            if matches!(
                v.as_str(),
                "gtc" | "good_until_cancelled" | "good_til_cancelled" | "good_till_cancelled"
            ) =>
        {
            Ok(TimeInForce::GoodUntilCancelled {
                post_only: post_only.unwrap_or(false),
            })
        }
        Some(ref v)
            if matches!(
                v.as_str(),
                "day" | "good_until_end_of_day" | "good_til_end_of_day" | "gtd"
            ) =>
        {
            ensure_post_only_unused(post_only, v)?;
            Ok(TimeInForce::GoodUntilEndOfDay)
        }
        Some(ref v) if matches!(v.as_str(), "fok" | "fill_or_kill") => {
            ensure_post_only_unused(post_only, v)?;
            Ok(TimeInForce::FillOrKill)
        }
        Some(ref v) if matches!(v.as_str(), "ioc" | "immediate_or_cancel") => {
            ensure_post_only_unused(post_only, v)?;
            Ok(TimeInForce::ImmediateOrCancel)
        }
        Some(v) => Err(PyValueError::new_err(format!("invalid time_in_force: {v}"))),
    }
}

fn ensure_post_only_unused(post_only: Option<bool>, context: &str) -> PyResult<()> {
    if post_only.is_some() {
        Err(PyValueError::new_err(format!(
            "post_only is only supported with good_until_cancelled (got {context})"
        )))
    } else {
        Ok(())
    }
}

fn ensure_non_empty<T>(items: &[T], context: &str) -> PyResult<()> {
    if items.is_empty() {
        Err(PyValueError::new_err(format!(
            "{context} requires at least one entry"
        )))
    } else {
        Ok(())
    }
}

fn to_one_or_many<T>(mut items: Vec<T>, context: &str) -> PyResult<OneOrMany<T>> {
    match items.len() {
        0 => Err(PyValueError::new_err(format!(
            "{context} requires at least one item"
        ))),
        1 => Ok(OneOrMany::One(items.remove(0))),
        _ => Ok(OneOrMany::Many(items)),
    }
}
