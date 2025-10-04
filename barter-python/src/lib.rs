#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]
#![allow(unsafe_op_in_unsafe_fn, clippy::useless_conversion, clippy::needless_borrow)]

//! Python bindings for the Barter trading engine.

mod analytics;
mod command;
mod config;
mod data;
mod logging;
mod summary;
mod system;

use analytics::{
    calculate_calmar_ratio, calculate_max_drawdown, calculate_mean_drawdown,
    calculate_profit_factor, calculate_rate_of_return, calculate_sharpe_ratio,
    calculate_sortino_ratio, calculate_win_rate, generate_drawdown_series,
};
use barter::engine::{command::Command, state::trading::TradingState};
use barter::execution::AccountStreamEvent;
use barter::{EngineEvent, Timed};
use barter_data::{
    books::Level,
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
    subscription::{
        book::OrderBookL1, candle::Candle, liquidation::Liquidation, trade::PublicTrade,
    },
};
use barter_execution::{
    AccountEvent, AccountEventKind,
    balance::{AssetBalance, Balance},
    order::{
        id::{OrderId, StrategyId},
        request::OrderResponseCancel,
        state::Cancelled,
    },
    trade::{AssetFees, Trade, TradeId},
};
use barter_instrument::{
    asset::{AssetIndex, QuoteAsset},
    exchange::{ExchangeId, ExchangeIndex},
    instrument::InstrumentIndex,
};
use barter_integration::{Terminal, snapshot::Snapshot};
use chrono::{DateTime, Utc};
use command::{
    PyInstrumentFilter, PyOrderKey, PyOrderRequestCancel, PyOrderRequestOpen, PyOrderSnapshot,
    clone_filter, collect_cancel_requests, collect_open_requests, parse_decimal, parse_side,
};
use config::PySystemConfig;
use data::{PyDynamicStreams, PyExchangeId, PySubKind, PySubscription, init_dynamic_streams};
use logging::init_tracing;
use pyo3::{Bound, exceptions::PyValueError, prelude::*, types::PyModule};
use serde_json::Value;
use summary::{
    PyAssetTearSheet, PyBalance, PyDrawdown, PyInstrumentTearSheet, PyMeanDrawdown,
    PyMetricWithInterval, PyTradingSummary,
};
use system::{PySystemHandle, run_historic_backtest, start_system};

/// Wrapper around [`Timed`] with a floating point value for Python exposure.
#[pyclass(module = "barter_python", name = "TimedF64", unsendable)]
#[derive(Debug, Clone)]
pub struct PyTimedF64 {
    inner: Timed<f64>,
}

#[pymethods]
impl PyTimedF64 {
    /// Create a new [`Timed`] value.
    #[new]
    #[pyo3(signature = (value, time))]
    pub fn new(value: f64, time: DateTime<Utc>) -> Self {
        Self {
            inner: Timed { value, time },
        }
    }

    /// Value component of the timed pair.
    #[getter]
    pub fn value(&self) -> f64 {
        self.inner.value
    }

    /// Timestamp component of the timed pair.
    #[getter]
    pub fn time(&self) -> DateTime<Utc> {
        self.inner.time
    }

    /// Return a formatted representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "TimedF64(value={}, time={})",
            self.inner.value, self.inner.time
        ))
    }
}

/// Wrapper around [`EngineEvent`] value for Python.
#[pyclass(module = "barter_python", name = "EngineEvent", unsendable)]
#[derive(Debug, Clone)]
pub struct PyEngineEvent {
    inner: EngineEvent,
}

#[pymethods]
impl PyEngineEvent {
    /// Construct a shutdown [`EngineEvent`].
    #[staticmethod]
    pub fn shutdown() -> Self {
        Self {
            inner: EngineEvent::shutdown(),
        }
    }

    /// Construct an [`EngineEvent`] from a JSON string.
    #[staticmethod]
    pub fn from_json(data: &str) -> PyResult<Self> {
        let inner =
            serde_json::from_str(data).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(Self { inner })
    }

    /// Construct an [`EngineEvent`] from a Python dictionary-like object.
    #[staticmethod]
    pub fn from_dict(py: Python<'_>, value: PyObject) -> PyResult<Self> {
        let json_module = PyModule::import_bound(py, "json")?;
        let dumps = json_module.getattr("dumps")?;
        let serialized: String = dumps.call1((value,))?.extract()?;
        let mut json_value: Value = serde_json::from_str(&serialized)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        if let Value::Object(ref mut outer) = json_value {
            let needs_patch = outer
                .get("Shutdown")
                .map(|inner| matches!(inner, Value::Object(map) if map.is_empty()))
                .unwrap_or(false);

            if needs_patch {
                outer.insert("Shutdown".to_string(), Value::Null);
            }
        }

        let inner = serde_json::from_value(json_value)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner })
    }

    /// Construct a trading state update event.
    #[staticmethod]
    pub fn trading_state(enabled: bool) -> Self {
        let state = if enabled {
            TradingState::Enabled
        } else {
            TradingState::Disabled
        };

        Self {
            inner: EngineEvent::TradingStateUpdate(state),
        }
    }

    /// Construct an [`EngineEvent::Command`] to send open order requests.
    #[staticmethod]
    pub fn send_open_requests(
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestOpen>>,
    ) -> PyResult<Self> {
        let command = Command::SendOpenRequests(collect_open_requests(py, requests)?);
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to send cancel order requests.
    #[staticmethod]
    pub fn send_cancel_requests(
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestCancel>>,
    ) -> PyResult<Self> {
        let command = Command::SendCancelRequests(collect_cancel_requests(py, requests)?);
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to close positions using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn close_positions(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::ClosePositions(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Command`] to cancel orders using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn cancel_orders(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::CancelOrders(clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Market`] wrapping a public trade update.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (exchange, instrument, price, quantity, side, time_exchange, trade_id=None, time_received=None))]
    pub fn market_trade(
        exchange: &str,
        instrument: usize,
        price: f64,
        quantity: f64,
        side: &str,
        time_exchange: DateTime<Utc>,
        trade_id: Option<&str>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        if !price.is_finite() {
            return Err(PyValueError::new_err(
                "price must be a finite numeric value",
            ));
        }

        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(PyValueError::new_err(
                "quantity must be a positive, finite numeric value",
            ));
        }

        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let trade_side = parse_side(side)?;

        let trade = PublicTrade {
            id: trade_id.unwrap_or_default().to_owned(),
            price,
            amount: quantity,
            side: trade_side,
        };

        let event = MarketEvent {
            time_exchange,
            time_received: time_received.unwrap_or(time_exchange),
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::Trade(trade),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping a level one order book snapshot.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (exchange, instrument, last_update_time, best_bid=None, best_ask=None, time_exchange=None, time_received=None))]
    pub fn market_order_book_l1(
        exchange: &str,
        instrument: usize,
        last_update_time: DateTime<Utc>,
        best_bid: Option<(f64, f64)>,
        best_ask: Option<(f64, f64)>,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);

        let best_bid = parse_order_book_level(best_bid, "best bid")?;
        let best_ask = parse_order_book_level(best_ask, "best ask")?;

        let l1 = OrderBookL1 {
            last_update_time,
            best_bid,
            best_ask,
        };

        let time_exchange = time_exchange.unwrap_or(last_update_time);
        let time_received = time_received.unwrap_or(time_exchange);

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::OrderBookL1(l1),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping a candle update.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (exchange, instrument, time_exchange, close_time, open, high, low, close, volume, trade_count, time_received=None))]
    pub fn market_candle(
        exchange: &str,
        instrument: usize,
        time_exchange: DateTime<Utc>,
        close_time: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        trade_count: u64,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        if !volume.is_finite() || volume < 0.0 {
            return Err(PyValueError::new_err(
                "volume must be a non-negative, finite numeric value",
            ));
        }

        if !high.is_finite() || !low.is_finite() || !open.is_finite() || !close.is_finite() {
            return Err(PyValueError::new_err(
                "candle prices must be finite numeric values",
            ));
        }

        if low > high {
            return Err(PyValueError::new_err(
                "low price cannot be greater than high price",
            ));
        }

        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let time_received = time_received.unwrap_or(time_exchange);

        let candle = Candle {
            close_time,
            open,
            high,
            low,
            close,
            volume,
            trade_count,
        };

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::Candle(candle),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping a liquidation event.
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, price, quantity, side, time_exchange, time_received=None))]
    pub fn market_liquidation(
        exchange: &str,
        instrument: usize,
        price: f64,
        quantity: f64,
        side: &str,
        time_exchange: DateTime<Utc>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        if !price.is_finite() || price <= 0.0 {
            return Err(PyValueError::new_err(
                "price must be a positive, finite numeric value",
            ));
        }

        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(PyValueError::new_err(
                "quantity must be a positive, finite numeric value",
            ));
        }

        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let side = parse_side(side)?;
        let time_received = time_received.unwrap_or(time_exchange);

        let liquidation = Liquidation {
            side,
            price,
            quantity,
            time: time_exchange,
        };

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::Liquidation(liquidation),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] signalling the stream is reconnecting.
    #[staticmethod]
    pub fn market_reconnecting(exchange: &str) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Reconnecting(exchange_id)),
        })
    }

    /// Construct an [`EngineEvent::Account`] signalling the account stream is reconnecting.
    #[staticmethod]
    pub fn account_reconnecting(exchange: &str) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Reconnecting(exchange_id)),
        })
    }

    /// Construct an [`EngineEvent::Account`] with a balance snapshot update.
    #[staticmethod]
    #[pyo3(signature = (exchange, asset, total, free, time_exchange))]
    pub fn account_balance_snapshot(
        exchange: usize,
        asset: usize,
        total: f64,
        free: f64,
        time_exchange: DateTime<Utc>,
    ) -> PyResult<Self> {
        if free > total {
            return Err(PyValueError::new_err(
                "free balance cannot exceed total balance",
            ));
        }

        let total_decimal = parse_decimal(total, "total balance")?;
        let free_decimal = parse_decimal(free, "free balance")?;

        let balance = Balance::new(total_decimal, free_decimal);
        let asset_balance = AssetBalance::new(AssetIndex(asset), balance, time_exchange);
        let snapshot = Snapshot::new(asset_balance);

        let event = AccountEvent::new(
            ExchangeIndex(exchange),
            AccountEventKind::BalanceSnapshot(snapshot),
        );

        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Account`] with an order snapshot update.
    #[staticmethod]
    pub fn account_order_snapshot(exchange: usize, snapshot: &PyOrderSnapshot) -> PyResult<Self> {
        let exchange_index = ExchangeIndex(exchange);
        let order_snapshot = snapshot.clone_inner();

        if order_snapshot.key.exchange != exchange_index {
            return Err(PyValueError::new_err(
                "snapshot key exchange does not match provided exchange index",
            ));
        }

        let event = AccountEvent::new(
            exchange_index,
            AccountEventKind::OrderSnapshot(Snapshot::new(order_snapshot)),
        );

        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Account`] reporting a successful order cancellation.
    #[staticmethod]
    #[pyo3(signature = (exchange, request, order_id, time_exchange))]
    pub fn account_order_cancelled(
        exchange: usize,
        request: &PyOrderRequestCancel,
        order_id: &str,
        time_exchange: DateTime<Utc>,
    ) -> PyResult<Self> {
        let exchange_index = ExchangeIndex(exchange);
        let inner = request.clone_inner();

        if inner.key.exchange != exchange_index {
            return Err(PyValueError::new_err(
                "cancel request key exchange does not match provided exchange index",
            ));
        }

        let order_id = OrderId::new(order_id);

        if let Some(existing) = &inner.state.id && existing != &order_id {
            return Err(PyValueError::new_err(
                "order_id does not match the identifier on the cancel request",
            ));
        }

        let cancelled = Cancelled::new(order_id.clone(), time_exchange);
        let response = OrderResponseCancel {
            key: inner.key,
            state: Ok(cancelled),
        };

        let event = AccountEvent::new(exchange_index, AccountEventKind::OrderCancelled(response));

        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Account`] with a trade fill update.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (exchange, instrument, strategy_id, order_id, trade_id, side, price, quantity, time_exchange, fees=None))]
    pub fn account_trade(
        exchange: usize,
        instrument: usize,
        strategy_id: &str,
        order_id: &str,
        trade_id: &str,
        side: &str,
        price: f64,
        quantity: f64,
        time_exchange: DateTime<Utc>,
        fees: Option<f64>,
    ) -> PyResult<Self> {
        if price <= 0.0 || !price.is_finite() {
            return Err(PyValueError::new_err(
                "price must be a positive, finite numeric value",
            ));
        }

        if quantity <= 0.0 || !quantity.is_finite() {
            return Err(PyValueError::new_err(
                "quantity must be a positive, finite numeric value",
            ));
        }

        if let Some(fee_value) = fees
            && (fee_value < 0.0 || !fee_value.is_finite())
        {
            return Err(PyValueError::new_err(
                "fees must be a non-negative, finite numeric value",
            ));
        }

        let side = parse_side(side)?;
        let price_decimal = parse_decimal(price, "price")?;
        let quantity_decimal = parse_decimal(quantity, "quantity")?;
        let fees_decimal = parse_decimal(fees.unwrap_or(0.0), "fees")?;

        let trade = Trade::<QuoteAsset, InstrumentIndex> {
            id: TradeId::new(trade_id),
            order_id: OrderId::new(order_id),
            instrument: InstrumentIndex(instrument),
            strategy: StrategyId::new(strategy_id),
            time_exchange,
            side,
            price: price_decimal,
            quantity: quantity_decimal,
            fees: AssetFees::quote_fees(fees_decimal),
        };

        let event = AccountEvent::new(ExchangeIndex(exchange), AccountEventKind::Trade(trade));

        Ok(Self {
            inner: EngineEvent::Account(AccountStreamEvent::Item(event)),
        })
    }

    /// Check if the underlying event is terminal.
    pub fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
    }

    /// Serialize the [`EngineEvent`] to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Convert the [`EngineEvent`] into a Python dictionary via JSON round-trip.
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json = self.to_json()?;
        let json_module = PyModule::import_bound(py, "json")?;
        let loads = json_module.getattr("loads")?;
        Ok(loads.call1((json,))?.into_py(py))
    }

    /// Debug style string representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("EngineEvent({:?})", self.inner))
    }
}

fn parse_exchange_id(value: &str) -> PyResult<ExchangeId> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PyValueError::new_err("exchange must not be empty"));
    }

    let normalized = trimmed.to_ascii_lowercase();
    serde_json::from_value(Value::String(normalized)).map_err(|_| {
        PyValueError::new_err(format!(
            "unknown exchange identifier: {trimmed}. expected snake_case exchange ids such as 'binance_spot'"
        ))
    })
}

fn parse_order_book_level(value: Option<(f64, f64)>, label: &str) -> PyResult<Option<Level>> {
    match value {
        None => Ok(None),
        Some((price, amount)) => {
            if !price.is_finite() || price <= 0.0 {
                return Err(PyValueError::new_err(format!(
                    "{label} price must be a positive, finite numeric value"
                )));
            }

            if !amount.is_finite() || amount < 0.0 {
                return Err(PyValueError::new_err(format!(
                    "{label} amount must be a non-negative, finite numeric value"
                )));
            }

            let price = parse_decimal(price, &format!("{label} price"))?;
            let amount = parse_decimal(amount, &format!("{label} amount"))?;

            Ok(Some(Level::new(price, amount)))
        }
    }
}

/// Convenience function returning a shutdown [`EngineEvent`].
#[pyfunction]
pub fn shutdown_event() -> PyEngineEvent {
    PyEngineEvent::shutdown()
}

/// Create a [`Timed`] floating point value.
#[pyfunction]
pub fn timed_f64(value: f64, time: DateTime<Utc>) -> PyTimedF64 {
    PyTimedF64::new(value, time)
}

/// Python module definition entry point.
#[pymodule]
pub fn barter_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySystemConfig>()?;
    m.add_class::<PyEngineEvent>()?;
    m.add_class::<PyTimedF64>()?;
    m.add_class::<PySystemHandle>()?;
    m.add_class::<PyOrderKey>()?;
    m.add_class::<PyOrderRequestOpen>()?;
    m.add_class::<PyOrderRequestCancel>()?;
    m.add_class::<PyOrderSnapshot>()?;
    m.add_class::<PyInstrumentFilter>()?;
    m.add_class::<PyTradingSummary>()?;
    m.add_class::<PyInstrumentTearSheet>()?;
    m.add_class::<PyAssetTearSheet>()?;
    m.add_class::<PyMetricWithInterval>()?;
    m.add_class::<PyDrawdown>()?;
    m.add_class::<PyMeanDrawdown>()?;
    m.add_class::<PyBalance>()?;
    m.add_class::<PyExchangeId>()?;
    m.add_class::<PySubKind>()?;
    m.add_class::<PySubscription>()?;
    m.add_class::<PyDynamicStreams>()?;
    m.add_function(wrap_pyfunction!(init_tracing, m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_event, m)?)?;
    m.add_function(wrap_pyfunction!(timed_f64, m)?)?;
    m.add_function(wrap_pyfunction!(run_historic_backtest, m)?)?;
    m.add_function(wrap_pyfunction!(start_system, m)?)?;
    m.add_function(wrap_pyfunction!(init_dynamic_streams, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_sharpe_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_sortino_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_calmar_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_profit_factor, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_win_rate, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_rate_of_return, m)?)?;
    m.add_function(wrap_pyfunction!(generate_drawdown_series, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_max_drawdown, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_mean_drawdown, m)?)?;

    // Expose module level constants.
    let shutdown = PyEngineEvent::shutdown();
    m.add("SHUTDOWN_EVENT", shutdown.into_py(py))?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use barter_data::{event::DataKind, streams::consumer::MarketStreamEvent};
    use barter_execution::order::{
        OrderKind, TimeInForce,
        id::{OrderId, StrategyId},
        state::{ActiveOrderState, OrderState},
    };
    use barter_execution::trade::TradeId;
    use barter_instrument::{Side, exchange::ExchangeId, instrument::InstrumentIndex};
    use chrono::{TimeDelta, TimeZone};
    use pyo3::{Python, types::PyDict};
    use rust_decimal::prelude::ToPrimitive;

    #[test]
    fn engine_event_shutdown_is_terminal() {
        let event = PyEngineEvent {
            inner: EngineEvent::shutdown(),
        };
        assert!(event.inner.is_terminal());
    }

    #[test]
    fn engine_event_trading_state_constructor() {
        let enabled = PyEngineEvent::trading_state(true);
        match enabled.inner {
            EngineEvent::TradingStateUpdate(state) => assert_eq!(state, TradingState::Enabled),
            other => panic!("unexpected event variant: {other:?}"),
        }

        let disabled = PyEngineEvent::trading_state(false);
        match disabled.inner {
            EngineEvent::TradingStateUpdate(state) => assert_eq!(state, TradingState::Disabled),
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn timed_f64_surfaces_value_and_time() {
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let timed = PyTimedF64 {
            inner: Timed { value: 42.5, time },
        };

        assert_eq!(timed.value(), 42.5);
        assert_eq!(timed.time(), time);
    }

    #[test]
    fn engine_event_market_trade_constructor() {
        let time_exchange = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        let time_received = time_exchange + TimeDelta::seconds(1);

        let event = PyEngineEvent::market_trade(
            "binance_spot",
            2,
            101.25,
            0.5,
            "buy",
            time_exchange,
            Some("trade-1"),
            Some(time_received),
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::BinanceSpot);
                assert_eq!(item.instrument, InstrumentIndex(2));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_received);

                match item.kind {
                    DataKind::Trade(trade) => {
                        assert_eq!(trade.id, "trade-1");
                        assert_eq!(trade.price, 101.25);
                        assert_eq!(trade.amount, 0.5);
                        assert_eq!(trade.side, Side::Buy);
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_trade_defaults() {
        let time_exchange = Utc.with_ymd_and_hms(2024, 5, 6, 7, 8, 9).unwrap();

        let event =
            PyEngineEvent::market_trade("mock", 0, 1.25, 3.5, "sell", time_exchange, None, None)
                .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::Mock);
                assert_eq!(item.instrument, InstrumentIndex(0));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::Trade(trade) => {
                        assert!(trade.id.is_empty());
                        assert_eq!(trade.price, 1.25);
                        assert_eq!(trade.amount, 3.5);
                        assert_eq!(trade.side, Side::Sell);
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_order_book_l1_constructor() {
        let last_update = Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap();
        let time_exchange = last_update + TimeDelta::seconds(1);

        let event = PyEngineEvent::market_order_book_l1(
            "binance_spot",
            7,
            last_update,
            Some((100.5, 2.0)),
            Some((101.0, 1.5)),
            Some(time_exchange),
            None,
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::BinanceSpot);
                assert_eq!(item.instrument, InstrumentIndex(7));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::OrderBookL1(book) => {
                        assert_eq!(book.last_update_time, last_update);

                        let best_bid = book.best_bid.expect("best bid expected");
                        assert!((best_bid.price.to_f64().unwrap() - 100.5).abs() < f64::EPSILON);
                        assert!((best_bid.amount.to_f64().unwrap() - 2.0).abs() < f64::EPSILON);

                        let best_ask = book.best_ask.expect("best ask expected");
                        assert!((best_ask.price.to_f64().unwrap() - 101.0).abs() < f64::EPSILON);
                        assert!((best_ask.amount.to_f64().unwrap() - 1.5).abs() < f64::EPSILON);
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_candle_constructor() {
        let time_exchange = Utc.with_ymd_and_hms(2025, 2, 3, 4, 5, 6).unwrap();
        let close_time = time_exchange + TimeDelta::minutes(1);

        let event = PyEngineEvent::market_candle(
            "kraken",
            4,
            time_exchange,
            close_time,
            100.0,
            110.0,
            95.0,
            105.0,
            250.5,
            42,
            None,
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::Kraken);
                assert_eq!(item.instrument, InstrumentIndex(4));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::Candle(candle) => {
                        assert_eq!(candle.close_time, close_time);
                        assert_eq!(candle.open, 100.0);
                        assert_eq!(candle.high, 110.0);
                        assert_eq!(candle.low, 95.0);
                        assert_eq!(candle.close, 105.0);
                        assert_eq!(candle.volume, 250.5);
                        assert_eq!(candle.trade_count, 42);
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_liquidation_constructor() {
        let time_exchange = Utc.with_ymd_and_hms(2025, 3, 4, 5, 6, 7).unwrap();

        let event = PyEngineEvent::market_liquidation(
            "mock",
            2,
            20550.25,
            0.35,
            "sell",
            time_exchange,
            None,
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::Mock);
                assert_eq!(item.instrument, InstrumentIndex(2));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::Liquidation(liquidation) => {
                        assert_eq!(liquidation.side, Side::Sell);
                        assert_eq!(liquidation.price, 20550.25);
                        assert_eq!(liquidation.quantity, 0.35);
                        assert_eq!(liquidation.time, time_exchange);
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_reconnecting_constructor() {
        let event = PyEngineEvent::market_reconnecting("kraken").unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Reconnecting(exchange)) => {
                assert_eq!(exchange, ExchangeId::Kraken);
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_account_reconnecting_constructor() {
        let event = PyEngineEvent::account_reconnecting("binance_spot").unwrap();

        match event.inner {
            EngineEvent::Account(AccountStreamEvent::Reconnecting(exchange)) => {
                assert_eq!(exchange, ExchangeId::BinanceSpot);
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_account_trade_constructor() {
        let time_exchange = Utc.with_ymd_and_hms(2025, 8, 9, 10, 11, 12).unwrap();

        let event = PyEngineEvent::account_trade(
            3,
            4,
            "strategy-123",
            "order-456",
            "trade-789",
            "buy",
            125.25,
            0.75,
            time_exchange,
            Some(0.0015),
        )
        .unwrap();

        match event.inner {
            EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
                assert_eq!(account_event.exchange, ExchangeIndex(3));

                match account_event.kind {
                    AccountEventKind::Trade(trade) => {
                        assert_eq!(trade.instrument, InstrumentIndex(4));
                        assert_eq!(trade.strategy, StrategyId::new("strategy-123"));
                        assert_eq!(trade.order_id, OrderId::new("order-456"));
                        assert_eq!(trade.id, TradeId::new("trade-789"));
                        assert_eq!(trade.side, Side::Buy);
                        assert_eq!(trade.price.to_f64().unwrap(), 125.25);
                        assert_eq!(trade.quantity.to_f64().unwrap(), 0.75);
                        assert_eq!(trade.time_exchange, time_exchange);
                        assert_eq!(trade.fees.fees.to_f64().unwrap(), 0.0015);
                    }
                    other => panic!("unexpected account event kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_account_order_snapshot_open() {
        let key = PyOrderKey::new(1, 2, "strategy-alpha", Some("cid-1"));
        let open_request = PyOrderRequestOpen::new(
            &key,
            "buy",
            105.25,
            0.75,
            "limit",
            Some("good_until_cancelled"),
            Some(true),
        )
        .unwrap();
        let time_exchange = Utc.with_ymd_and_hms(2025, 9, 10, 11, 12, 13).unwrap();

        let snapshot = PyOrderSnapshot::from_open_request(
            &open_request,
            Some("order-789"),
            Some(time_exchange),
            0.25,
        )
        .unwrap();

        let event = PyEngineEvent::account_order_snapshot(1, &snapshot).unwrap();

        match event.inner {
            EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
                assert_eq!(account_event.exchange, ExchangeIndex(1));

                match account_event.kind {
                    AccountEventKind::OrderSnapshot(snapshot) => {
                        let order = snapshot.value();
                        assert_eq!(order.key.exchange, ExchangeIndex(1));
                        assert_eq!(order.key.instrument, InstrumentIndex(2));
                        assert_eq!(order.key.strategy, StrategyId::new("strategy-alpha"));
                        assert_eq!(order.side, Side::Buy);
                        assert_eq!(order.price.to_f64().unwrap(), 105.25);
                        assert_eq!(order.quantity.to_f64().unwrap(), 0.75);
                        assert_eq!(order.kind, OrderKind::Limit);
                        assert_eq!(
                            order.time_in_force,
                            TimeInForce::GoodUntilCancelled { post_only: true }
                        );

                        match &order.state {
                            OrderState::Active(ActiveOrderState::Open(open)) => {
                                assert_eq!(open.id, OrderId::new("order-789"));
                                assert_eq!(open.time_exchange, time_exchange);
                                assert_eq!(open.filled_quantity.to_f64().unwrap(), 0.25);
                            }
                            other => panic!("unexpected order state: {other:?}"),
                        }
                    }
                    other => panic!("unexpected account event kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_account_order_snapshot_open_inflight() {
        let key = PyOrderKey::new(3, 4, "strategy-beta", Some("cid-2"));
        let open_request =
            PyOrderRequestOpen::new(&key, "sell", 250.0, 1.5, "limit", None, None).unwrap();

        let snapshot = PyOrderSnapshot::from_open_request(&open_request, None, None, 0.0).unwrap();

        let event = PyEngineEvent::account_order_snapshot(3, &snapshot).unwrap();

        match event.inner {
            EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
                assert_eq!(account_event.exchange, ExchangeIndex(3));

                match account_event.kind {
                    AccountEventKind::OrderSnapshot(snapshot) => {
                        let order = snapshot.value();
                        assert_eq!(order.key.exchange, ExchangeIndex(3));
                        assert_eq!(order.key.instrument, InstrumentIndex(4));
                        assert_eq!(order.side, Side::Sell);

                        match &order.state {
                            OrderState::Active(ActiveOrderState::OpenInFlight(_)) => {}
                            other => panic!("unexpected order state: {other:?}"),
                        }
                    }
                    other => panic!("unexpected account event kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_account_order_cancelled_success() {
        let key = PyOrderKey::new(2, 5, "strategy-gamma", Some("cid-3"));
        let cancel_request = PyOrderRequestCancel::new(&key, Some("order-456"))
            .expect("cancel request should build");
        let time_exchange = Utc.with_ymd_and_hms(2025, 12, 1, 2, 3, 4).unwrap();

        let event =
            PyEngineEvent::account_order_cancelled(2, &cancel_request, "order-456", time_exchange)
                .unwrap();

        match event.inner {
            EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
                assert_eq!(account_event.exchange, ExchangeIndex(2));

                match account_event.kind {
                    AccountEventKind::OrderCancelled(response) => {
                        assert_eq!(response.key.exchange, ExchangeIndex(2));
                        assert_eq!(response.key.instrument, InstrumentIndex(5));

                        match response.state {
                            Ok(cancelled) => {
                                assert_eq!(cancelled.id, OrderId::new("order-456"));
                                assert_eq!(cancelled.time_exchange, time_exchange);
                            }
                            Err(err) => panic!("unexpected cancellation error: {err:?}"),
                        }
                    }
                    other => panic!("unexpected account event kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_json_roundtrip() {
        let event = PyEngineEvent::trading_state(true);
        let json = event.to_json().unwrap();
        let restored = PyEngineEvent::from_json(&json).unwrap();
        assert_eq!(restored.inner, event.inner);
    }

    #[test]
    fn engine_event_dict_roundtrip() {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("Shutdown", PyDict::new_bound(py)).unwrap();

            let event = PyEngineEvent::from_dict(py, dict.into_py(py)).unwrap();
            assert!(event.inner.is_terminal());

            let object = event.to_dict(py).unwrap();
            let json_module = PyModule::import_bound(py, "json").unwrap();
            let dumps = json_module.getattr("dumps").unwrap();
            let dumped: String = dumps
                .call1((object.clone_ref(py),))
                .unwrap()
                .extract()
                .unwrap();
            assert!(dumped.contains("Shutdown"));
        });
    }
}
