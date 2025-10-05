use crate::{
    command::{DefaultOrderRequestOpen, PyOrderRequestOpen, parse_decimal},
    execution::{PyStrategyId, coerce_client_order_id},
};
use barter_execution::order::{
    OrderKey, OrderKind, TimeInForce,
    id::ClientOrderId,
    request::{OrderRequestOpen, RequestOpen},
};
use barter_instrument::{Side, exchange::ExchangeIndex, instrument::InstrumentIndex};
use pyo3::{PyObject, PyResult, Python, exceptions::PyValueError, prelude::*};

fn ensure_positive(value: &rust_decimal::Decimal, field: &str) -> PyResult<()> {
    if value <= &rust_decimal::Decimal::ZERO {
        Err(PyValueError::new_err(format!("{field} must be positive")))
    } else {
        Ok(())
    }
}

fn closing_side(position_side: Side) -> Side {
    match position_side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    }
}

fn build_close_position_request(
    exchange: ExchangeIndex,
    instrument: InstrumentIndex,
    position_side: Side,
    quantity: f64,
    price: f64,
    strategy_id: &PyStrategyId,
    client_order_id: ClientOrderId,
) -> PyResult<DefaultOrderRequestOpen> {
    let price_decimal = parse_decimal(price, "price")?;
    let quantity_decimal = parse_decimal(quantity, "quantity")?;
    ensure_positive(&quantity_decimal, "quantity")?;

    let request = OrderRequestOpen {
        key: OrderKey {
            exchange,
            instrument,
            strategy: strategy_id.inner(),
            cid: client_order_id,
        },
        state: RequestOpen {
            side: closing_side(position_side),
            price: price_decimal,
            quantity: quantity_decimal,
            kind: OrderKind::Market,
            time_in_force: TimeInForce::ImmediateOrCancel,
        },
    };

    Ok(request)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (exchange, instrument, side, quantity, strategy_id, price, client_order_id=None))]
pub fn build_ioc_market_order_to_close_position(
    py: Python<'_>,
    exchange: usize,
    instrument: usize,
    side: &str,
    quantity: f64,
    strategy_id: &PyStrategyId,
    price: f64,
    client_order_id: Option<PyObject>,
) -> PyResult<PyOrderRequestOpen> {
    let position_side = crate::command::parse_side(side)?;
    let exchange_index = ExchangeIndex(exchange);
    let instrument_index = InstrumentIndex(instrument);

    let fallback_cid = ClientOrderId::new(format!("close-{}", instrument_index.index()));

    let cid = if let Some(object) = client_order_id {
        let bound = object.bind(py);
        coerce_client_order_id(Some(&bound))?
    } else {
        fallback_cid
    };

    let request = build_close_position_request(
        exchange_index,
        instrument_index,
        position_side,
        quantity,
        price,
        strategy_id,
        cid,
    )?;

    Ok(PyOrderRequestOpen { inner: request })
}
