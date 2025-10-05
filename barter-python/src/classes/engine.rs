use chrono::{DateTime, Utc};
use pyo3::{prelude::*, pyclass::CompareOp, types::PyModule, Bound, PyObject, exceptions::PyValueError};
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value as JsonValue;

use barter::{
    engine::{command::Command, state::trading::TradingState},
    execution::AccountStreamEvent,
    EngineEvent, Sequence, Timed,
};
use barter_data::{
    books::{Level, OrderBook},
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
    subscription::{
        book::{OrderBookEvent, OrderBookL1},
        candle::Candle,
        liquidation::Liquidation,
        trade::PublicTrade,
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
use barter_integration::Terminal;

use crate::{
    command::{
        collect_cancel_requests, collect_open_requests, clone_filter, parse_decimal, parse_side, PyInstrumentFilter,
        PyOrderRequestCancel, PyOrderRequestOpen, PyOrderSnapshot,
    },
    execution::PyExecutionAssetBalance,
    execution::PyTrade,
};

/// Wrapper around [`EngineEvent`] value for Python.
#[pyclass(module = "barter_python", name = "EngineEvent", unsendable)]
#[derive(Debug, Clone)]
pub struct PyEngineEvent {
    pub(crate) inner: EngineEvent,
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
        let mut json_value: JsonValue = serde_json::from_str(&serialized)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        if let JsonValue::Object(ref mut outer) = json_value {
            let needs_patch = outer
                .get("Shutdown")
                .map(|inner| matches!(inner, JsonValue::Object(map) if map.is_empty()))
                .unwrap_or(false);

            if needs_patch {
                outer.insert("Shutdown".to_string(), JsonValue::Null);
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

    /// Construct an [`EngineEvent::Command`] to cancel all orders.
    #[staticmethod]
    pub fn cancel_all_orders(filter: Option<PyInstrumentFilter>) -> PyResult<Self> {
        let command = Command::CancelOrders(crate::command::clone_filter(filter.as_ref()));
        Ok(Self {
            inner: EngineEvent::Command(command),
        })
    }

    /// Construct an [`EngineEvent::Command`] to close positions using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn close_positions(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::ClosePositions(crate::command::clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Command`] to cancel orders using an optional filter.
    #[staticmethod]
    #[pyo3(signature = (filter=None))]
    pub fn cancel_orders(filter: Option<&PyInstrumentFilter>) -> Self {
        let command = Command::CancelOrders(crate::command::clone_filter(filter));
        Self {
            inner: EngineEvent::Command(command),
        }
    }

    /// Construct an [`EngineEvent::Market`] wrapping a public trade.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, trade_id, price, amount, side, time_exchange=None, time_received=None))]
    pub fn market_trade(
        exchange: &str,
        instrument: usize,
        trade_id: &str,
        price: f64,
        amount: f64,
        side: &str,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let side = parse_side_local(side)?;
        let time_exchange = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange);

        if !price.is_finite() || price <= 0.0 {
            return Err(PyValueError::new_err(
                "price must be a positive, finite numeric value",
            ));
        }

        if !amount.is_finite() || amount <= 0.0 {
            return Err(PyValueError::new_err(
                "amount must be a positive, finite numeric value",
            ));
        }

        let price = crate::command::parse_decimal(price, "price")?;
        let amount = crate::command::parse_decimal(amount, "amount")?;

        let trade = PublicTrade {
            id: trade_id.to_string(),
            price: price.to_f64().unwrap(),
            amount: amount.to_f64().unwrap(),
            side,
        };

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::Trade(trade),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping a candle.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, open, high, low, close, volume, time_exchange=None, time_received=None))]
    pub fn market_candle(
        exchange: &str,
        instrument: usize,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let time_exchange = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange);

        if !open.is_finite() || open <= 0.0 {
            return Err(PyValueError::new_err(
                "open must be a positive, finite numeric value",
            ));
        }

        if !high.is_finite() || high <= 0.0 {
            return Err(PyValueError::new_err(
                "high must be a positive, finite numeric value",
            ));
        }

        if !low.is_finite() || low <= 0.0 {
            return Err(PyValueError::new_err(
                "low must be a positive, finite numeric value",
            ));
        }

        if !close.is_finite() || close <= 0.0 {
            return Err(PyValueError::new_err(
                "close must be a positive, finite numeric value",
            ));
        }

        if !volume.is_finite() || volume < 0.0 {
            return Err(PyValueError::new_err(
                "volume must be a non-negative, finite numeric value",
            ));
        }

        let open = crate::command::parse_decimal(open, "open")?;
        let high = crate::command::parse_decimal(high, "high")?;
        let low = crate::command::parse_decimal(low, "low")?;
        let close = crate::command::parse_decimal(close, "close")?;
        let volume = crate::command::parse_decimal(volume, "volume")?;

        let candle = Candle {
            close_time: time_exchange,
            open: open.to_f64().unwrap(),
            high: high.to_f64().unwrap(),
            low: low.to_f64().unwrap(),
            close: close.to_f64().unwrap(),
            volume: volume.to_f64().unwrap(),
            trade_count: 0, // Default value since not provided
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

    /// Construct an [`EngineEvent::Market`] wrapping a liquidation.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, price, amount, side, time_exchange=None, time_received=None))]
    pub fn market_liquidation(
        exchange: &str,
        instrument: usize,
        price: f64,
        amount: f64,
        side: &str,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);
        let side = parse_side_local(side)?;
        let time_exchange = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange);

        if !price.is_finite() || price <= 0.0 {
            return Err(PyValueError::new_err(
                "price must be a positive, finite numeric value",
            ));
        }

        if !amount.is_finite() || amount <= 0.0 {
            return Err(PyValueError::new_err(
                "amount must be a positive, finite numeric value",
            ));
        }

        let price = crate::command::parse_decimal(price, "price")?;
        let amount = crate::command::parse_decimal(amount, "amount")?;

        let liquidation = Liquidation {
            side,
            price: price.to_f64().unwrap(),
            quantity: amount.to_f64().unwrap(),
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

    /// Construct an [`EngineEvent::Market`] wrapping an order book snapshot.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, sequence, time_engine, bids, asks, time_exchange=None, time_received=None))]
    pub fn market_order_book_snapshot(
        exchange: &str,
        instrument: usize,
        sequence: i64,
        time_engine: Option<DateTime<Utc>>,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);

        let bids_levels: Vec<Level> = bids
            .into_iter()
            .map(|(p, a)| parse_level(p, a))
            .collect::<PyResult<Vec<Level>>>()?;
        let asks_levels: Vec<Level> = asks
            .into_iter()
            .map(|(p, a)| parse_level(p, a))
            .collect::<PyResult<Vec<Level>>>()?;

        let order_book = OrderBook::new(sequence as u64, time_engine, bids_levels, asks_levels);

        let time_exchange = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange);

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::OrderBook(OrderBookEvent::Snapshot(order_book)),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping an order book update.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, sequence, time_engine, bids, asks, time_exchange=None, time_received=None))]
    pub fn market_order_book_update(
        exchange: &str,
        instrument: usize,
        sequence: i64,
        time_engine: Option<DateTime<Utc>>,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);

        let bids_levels: Vec<Level> = bids
            .into_iter()
            .map(|(p, a)| parse_level(p, a))
            .collect::<PyResult<Vec<Level>>>()?;
        let asks_levels: Vec<Level> = asks
            .into_iter()
            .map(|(p, a)| parse_level(p, a))
            .collect::<PyResult<Vec<Level>>>()?;

        let order_book = OrderBook::new(sequence as u64, time_engine, bids_levels, asks_levels);

        let time_exchange = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange);

        let event = MarketEvent {
            time_exchange,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::OrderBook(OrderBookEvent::Update(order_book)),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
        })
    }

    /// Construct an [`EngineEvent::Market`] wrapping an order book L1 snapshot.
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    #[pyo3(signature = (exchange, instrument, bid_price, bid_amount, ask_price, ask_amount, time_exchange=None, time_received=None))]
    pub fn market_order_book_l1(
        exchange: &str,
        instrument: usize,
        bid_price: Option<f64>,
        bid_amount: Option<f64>,
        ask_price: Option<f64>,
        ask_amount: Option<f64>,
        time_exchange: Option<DateTime<Utc>>,
        time_received: Option<DateTime<Utc>>,
    ) -> PyResult<Self> {
        let exchange_id = parse_exchange_id(exchange)?;
        let instrument_index = InstrumentIndex(instrument);

        let time_exchange_unwrapped = time_exchange.unwrap_or(Utc::now());
        let time_received = time_received.unwrap_or(time_exchange_unwrapped);

        let bid = parse_order_book_level(
            bid_price.zip(bid_amount),
            "bid",
        )?;
        let ask = parse_order_book_level(
            ask_price.zip(ask_amount),
            "ask",
        )?;

        let order_book_l1 = OrderBookL1 {
            last_update_time: time_exchange_unwrapped,
            best_bid: bid,
            best_ask: ask,
        };

        let event = MarketEvent {
            time_exchange: time_exchange_unwrapped,
            time_received,
            exchange: exchange_id,
            instrument: instrument_index,
            kind: DataKind::OrderBookL1(order_book_l1),
        };

        Ok(Self {
            inner: EngineEvent::Market(MarketStreamEvent::Item(event)),
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
    serde_json::from_value(JsonValue::String(normalized)).map_err(|_| {
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

            let price = crate::command::parse_decimal(price, &format!("{label} price"))?;
            let amount = crate::command::parse_decimal(amount, &format!("{label} amount"))?;

            Ok(Some(Level::new(price, amount)))
        }
    }
}

fn parse_level(price: f64, amount: f64) -> PyResult<Level> {
    if !price.is_finite() || price <= 0.0 {
        return Err(PyValueError::new_err(
            "price must be a positive, finite numeric value",
        ));
    }

    if !amount.is_finite() || amount < 0.0 {
        return Err(PyValueError::new_err(
            "amount must be a non-negative, finite numeric value",
        ));
    }

    let price = crate::command::parse_decimal(price, "price")?;
    let amount = crate::command::parse_decimal(amount, "amount")?;

    Ok(Level::new(price, amount))
}

fn parse_side_local(value: &str) -> PyResult<barter_instrument::Side> {
    match value.to_lowercase().as_str() {
        "buy" => Ok(barter_instrument::Side::Buy),
        "sell" => Ok(barter_instrument::Side::Sell),
        _ => Err(PyValueError::new_err(format!(
            "invalid side: {value}. expected 'buy' or 'sell'"
        ))),
    }
}