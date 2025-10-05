use crate::{
    execution::{
        PyClientOrderId, PyOrderKind, PyStrategyId, PyTimeInForce, coerce_client_order_id,
        coerce_strategy_id,
    },
    instrument::{PyExchangeIndex, PyInstrumentIndex},
};
use barter::engine::state::instrument::filter::InstrumentFilter;
use barter_execution::order::request::{
    OrderRequestCancel, OrderRequestOpen, RequestCancel, RequestOpen,
};
use barter_execution::order::{
    OrderKey, OrderKind, OrderSnapshot, TimeInForce,
    id::{ClientOrderId, OrderId, StrategyId},
    state::{ActiveOrderState, Open, OpenInFlight, OrderState},
};
use barter_instrument::{
    Side, Underlying, asset::AssetIndex, exchange::ExchangeIndex, instrument::InstrumentIndex,
};
use barter_integration::collection::one_or_many::OneOrMany;
use chrono::{DateTime, Utc};
use pyo3::{Bound, Py, PyAny, Python, exceptions::PyValueError, prelude::*};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub type DefaultOrderKey = OrderKey<ExchangeIndex, InstrumentIndex>;
pub type DefaultOrderRequestOpen = OrderRequestOpen<ExchangeIndex, InstrumentIndex>;
pub type DefaultOrderRequestCancel = OrderRequestCancel<ExchangeIndex, InstrumentIndex>;
pub type DefaultInstrumentFilter = InstrumentFilter<ExchangeIndex, AssetIndex, InstrumentIndex>;

#[pyclass(
    module = "barter_python",
    name = "OrderKey",
    unsendable,
    eq,
    hash,
    frozen
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyOrderKey {
    pub(crate) inner: DefaultOrderKey,
}

impl PyOrderKey {
    pub(crate) fn clone_inner(&self) -> DefaultOrderKey {
        self.inner.clone()
    }

    pub(crate) fn from_inner(inner: DefaultOrderKey) -> Self {
        Self { inner }
    }

    pub(crate) fn from_parts(
        exchange: ExchangeIndex,
        instrument: InstrumentIndex,
        strategy: StrategyId,
        cid: ClientOrderId,
    ) -> Self {
        Self {
            inner: OrderKey {
                exchange,
                instrument,
                strategy,
                cid,
            },
        }
    }
}

#[pymethods]
impl PyOrderKey {
    #[new]
    #[pyo3(signature = (exchange, instrument, strategy, cid=None))]
    pub fn new(
        exchange: &Bound<'_, PyAny>,
        instrument: &Bound<'_, PyAny>,
        strategy: &Bound<'_, PyAny>,
        cid: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let exchange_index = parse_exchange_index(exchange)?;
        let instrument_index = parse_instrument_index(instrument)?;
        let strategy_id = coerce_strategy_id(strategy)?;
        let client_order_id = coerce_client_order_id(cid)?;

        Ok(Self::from_parts(
            exchange_index,
            instrument_index,
            strategy_id,
            client_order_id,
        ))
    }

    /// Construct an [`OrderKey`] from wrapper indices.
    #[staticmethod]
    #[pyo3(
        name = "from_indices",
        signature = (exchange, instrument, strategy, cid=None, client_order_id=None)
    )]
    pub fn from_indices(
        exchange: &PyExchangeIndex,
        instrument: &PyInstrumentIndex,
        strategy: &Bound<'_, PyAny>,
        cid: Option<&Bound<'_, PyAny>>,
        client_order_id: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        if cid.is_some() && client_order_id.is_some() {
            return Err(PyValueError::new_err(
                "provide either cid or client_order_id, not both",
            ));
        }

        let cid_arg = client_order_id.or(cid);

        let strategy_id = coerce_strategy_id(strategy)?;
        let client_order_id = coerce_client_order_id(cid_arg)?;

        Ok(Self::from_parts(
            exchange.inner(),
            instrument.inner(),
            strategy_id,
            client_order_id,
        ))
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
    pub fn strategy(&self) -> PyStrategyId {
        PyStrategyId::from_inner(self.inner.strategy.clone())
    }

    #[getter]
    pub fn cid(&self) -> PyClientOrderId {
        PyClientOrderId::from_inner(self.inner.cid.clone())
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
            "OrderKey(exchange={}, instrument={}, strategy='{}', cid='{}')",
            self.exchange(),
            self.instrument(),
            self.strategy_id(),
            self.client_order_id(),
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "{}:{}:{}:{}",
            self.exchange(),
            self.instrument(),
            self.strategy_id(),
            self.client_order_id()
        ))
    }
}

fn parse_exchange_index(value: &Bound<'_, PyAny>) -> PyResult<ExchangeIndex> {
    if let Ok(index) = value.extract::<usize>() {
        return Ok(ExchangeIndex(index));
    }

    if let Ok(wrapper) = value.extract::<Py<PyExchangeIndex>>() {
        return Ok(wrapper.borrow(value.py()).inner());
    }

    Err(PyValueError::new_err(
        "exchange must be an integer or ExchangeIndex",
    ))
}

fn parse_instrument_index(value: &Bound<'_, PyAny>) -> PyResult<InstrumentIndex> {
    if let Ok(index) = value.extract::<usize>() {
        return Ok(InstrumentIndex(index));
    }

    if let Ok(wrapper) = value.extract::<Py<PyInstrumentIndex>>() {
        return Ok(wrapper.borrow(value.py()).inner());
    }

    Err(PyValueError::new_err(
        "instrument must be an integer or InstrumentIndex",
    ))
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

    pub(crate) fn from_inner(inner: DefaultOrderRequestOpen) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyOrderRequestOpen {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(
        signature = (key, side, price, quantity, kind=None, time_in_force=None, post_only=None)
    )]
    pub fn new(
        key: &PyOrderKey,
        side: &str,
        price: f64,
        quantity: f64,
        kind: Option<&Bound<'_, PyAny>>,
        time_in_force: Option<&Bound<'_, PyAny>>,
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

    pub(crate) fn from_inner(inner: DefaultOrderRequestCancel) -> Self {
        Self { inner }
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

#[pyclass(module = "barter_python", name = "OrderSnapshot", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderSnapshot {
    pub(crate) inner: OrderSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex>,
}

impl PyOrderSnapshot {
    pub(crate) fn clone_inner(&self) -> OrderSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex> {
        self.inner.clone()
    }

    pub(crate) fn from_inner(
        inner: OrderSnapshot<ExchangeIndex, AssetIndex, InstrumentIndex>,
    ) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyOrderSnapshot {
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (request, order_id=None, time_exchange=None, filled_quantity=0.0))]
    pub fn from_open_request(
        request: &PyOrderRequestOpen,
        order_id: Option<&str>,
        time_exchange: Option<DateTime<Utc>>,
        filled_quantity: f64,
    ) -> PyResult<Self> {
        let inner = request.clone_inner();

        let mut order = OrderSnapshot {
            key: inner.key,
            side: inner.state.side,
            price: inner.state.price,
            quantity: inner.state.quantity,
            kind: inner.state.kind,
            time_in_force: inner.state.time_in_force,
            state: OrderState::Active(ActiveOrderState::OpenInFlight(OpenInFlight)),
        };

        match (order_id, time_exchange) {
            (Some(id), Some(time)) => {
                let filled = parse_decimal(filled_quantity, "filled_quantity")?;

                if filled.is_sign_negative() {
                    return Err(PyValueError::new_err(
                        "filled_quantity must be a non-negative numeric value",
                    ));
                }

                if filled > order.quantity {
                    return Err(PyValueError::new_err(
                        "filled_quantity cannot exceed order quantity",
                    ));
                }

                let order_id = OrderId::new(id);
                let open = Open::new(order_id, time, filled);
                order.state = OrderState::Active(ActiveOrderState::Open(open));
            }
            (Some(_), None) => {
                return Err(PyValueError::new_err(
                    "time_exchange is required when order_id is provided",
                ));
            }
            (None, Some(_)) => {
                return Err(PyValueError::new_err(
                    "order_id must be provided when time_exchange is set",
                ));
            }
            (None, None) => {
                if filled_quantity != 0.0 {
                    return Err(PyValueError::new_err(
                        "filled_quantity must be zero when order_id is not provided",
                    ));
                }
            }
        }

        Ok(Self { inner: order })
    }

    fn __repr__(&self) -> PyResult<String> {
        let order = &self.inner;
        let side = match order.side {
            Side::Buy => "buy",
            Side::Sell => "sell",
        };

        Ok(format!(
            "OrderSnapshot(exchange={}, instrument={}, strategy='{}', side='{}')",
            order.key.exchange.index(),
            order.key.instrument.index(),
            order.key.strategy.0,
            side,
        ))
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

pub(crate) fn parse_side(value: &str) -> PyResult<Side> {
    match value.to_ascii_lowercase().as_str() {
        "buy" | "b" => Ok(Side::Buy),
        "sell" | "s" => Ok(Side::Sell),
        other => Err(PyValueError::new_err(format!("invalid side: {other}"))),
    }
}

fn parse_order_kind(value: Option<&Bound<'_, PyAny>>) -> PyResult<OrderKind> {
    value
        .map(PyOrderKind::coerce)
        .unwrap_or(Ok(OrderKind::Limit))
}

pub(crate) fn parse_decimal(value: f64, field: &str) -> PyResult<Decimal> {
    Decimal::from_f64(value)
        .ok_or_else(|| PyValueError::new_err(format!("{field} must be a finite numeric value")))
}

fn parse_time_in_force(
    value: Option<&Bound<'_, PyAny>>,
    post_only: Option<bool>,
) -> PyResult<TimeInForce> {
    PyTimeInForce::coerce(value, post_only)
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
