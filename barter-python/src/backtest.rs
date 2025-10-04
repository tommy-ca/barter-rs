use std::{fs::File, io::BufReader, sync::Arc};

use crate::summary::decimal_to_py;
use barter::backtest::market_data::MarketDataInMemory;
use barter_data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
};
use barter_instrument::{Side, exchange::ExchangeId, instrument::InstrumentIndex};
use chrono::{DateTime, Utc};
use pyo3::{
    Bound, PyObject, PyResult, Python,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyModule},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};

#[pyclass(module = "barter_python", name = "MarketDataInMemory", unsendable)]
#[derive(Debug, Clone)]
pub struct PyMarketDataInMemory {
    _inner: MarketDataInMemory<DataKind>,
    events: Arc<Vec<MarketStreamEvent<InstrumentIndex, DataKind>>>,
    time_first_event: DateTime<Utc>,
}

impl PyMarketDataInMemory {
    fn from_events_vec(
        events: Vec<MarketStreamEvent<InstrumentIndex, DataKind>>,
    ) -> PyResult<Self> {
        if events.is_empty() {
            return Err(PyValueError::new_err(
                "market data requires at least one market stream event",
            ));
        }

        let time_first_event = events
            .iter()
            .find_map(|event| match event {
                MarketStreamEvent::Item(item) => Some(item.time_exchange),
                MarketStreamEvent::Reconnecting(_) => None,
            })
            .ok_or_else(|| PyValueError::new_err("market data must contain at least one item"))?;

        let arc = Arc::new(events);
        let inner = MarketDataInMemory::new(Arc::clone(&arc));

        Ok(Self {
            _inner: inner,
            events: arc,
            time_first_event,
        })
    }

    fn data_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
        PyModule::import_bound(py, "barter_python.data")
    }

    fn instrument_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
        PyModule::import_bound(py, "barter_python.instrument")
    }

    fn decimal_class(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        let decimal_module = PyModule::import_bound(py, "decimal")?;
        decimal_module.getattr("Decimal")
    }
}

#[pymethods]
impl PyMarketDataInMemory {
    #[new]
    #[pyo3(signature = (events))]
    pub fn __new__(events: Vec<PyObject>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let data_module = Self::data_module(py)?;
            let instrument_module = Self::instrument_module(py)?;
            let decimal_class = Self::decimal_class(py)?;

            let mut converted = Vec::with_capacity(events.len());
            for obj in events {
                let event = parse_market_event_from_py(
                    py,
                    &obj,
                    &data_module,
                    &instrument_module,
                    &decimal_class,
                )?;
                converted.push(MarketStreamEvent::Item(event));
            }

            Self::from_events_vec(converted)
        })
    }

    #[staticmethod]
    pub fn from_json_file(path: &str) -> PyResult<Self> {
        let file = File::open(path).map_err(|err| PyValueError::new_err(err.to_string()))?;
        let reader = BufReader::new(file);

        let raw_events: Vec<serde_json::Value> = serde_json::from_reader(reader)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        let mut events = Vec::with_capacity(raw_events.len());
        for value in raw_events {
            if let Some(item) = value.get("Item") {
                if let Some(ok) = item.get("Ok") {
                    let event: MarketEvent<InstrumentIndex, DataKind> =
                        serde_json::from_value(ok.clone())
                            .map_err(|err| PyValueError::new_err(err.to_string()))?;
                    events.push(MarketStreamEvent::Item(event));
                } else if let Some(err_value) = item.get("Err") {
                    return Err(PyValueError::new_err(format!(
                        "market data contains error event: {}",
                        err_value
                    )));
                }
            } else if let Some(exchange) = value.get("Reconnecting") {
                let exchange_id: ExchangeId = serde_json::from_value(exchange.clone())
                    .map_err(|err| PyValueError::new_err(err.to_string()))?;
                events.push(MarketStreamEvent::Reconnecting(exchange_id));
            }
        }

        Self::from_events_vec(events)
    }

    #[getter]
    pub fn time_first_event(&self) -> DateTime<Utc> {
        self.time_first_event
    }

    pub fn events(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let data_module = Self::data_module(py)?;
        let instrument_module = Self::instrument_module(py)?;

        let mut output = Vec::new();
        for event in self.events.iter() {
            if let Some(obj) =
                market_stream_event_to_py(py, event, &data_module, &instrument_module)?
            {
                output.push(obj);
            }
        }

        Ok(output)
    }

    pub fn __len__(&self) -> usize {
        self.events.len()
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|_py| {
            let mut reconnecting = 0;
            let mut items = 0;
            for event in self.events.iter() {
                match event {
                    MarketStreamEvent::Reconnecting(_) => reconnecting += 1,
                    MarketStreamEvent::Item(_) => items += 1,
                }
            }

            Ok(format!(
                "MarketDataInMemory(events={}, items={}, reconnecting={})",
                self.events.len(),
                items,
                reconnecting
            ))
        })
    }
}

fn market_stream_event_to_py(
    py: Python<'_>,
    event: &MarketStreamEvent<InstrumentIndex, DataKind>,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<Option<PyObject>> {
    match event {
        MarketStreamEvent::Reconnecting(_) => Ok(None),
        MarketStreamEvent::Item(item) => Ok(Some(market_event_to_py(
            py,
            item,
            data_module,
            instrument_module,
        )?)),
    }
}

fn market_event_to_py(
    py: Python<'_>,
    event: &MarketEvent<InstrumentIndex, DataKind>,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let market_event_class = data_module.getattr("MarketEvent")?;
    let kind = data_kind_to_py(py, &event.kind, data_module, instrument_module)?;
    let exchange = event.exchange.as_str();
    let instrument = event.instrument.index();

    let constructed = market_event_class.call1((
        event.time_exchange,
        event.time_received,
        exchange,
        instrument,
        kind,
    ))?;

    Ok(constructed.into_py(py))
}

fn data_kind_to_py(
    py: Python<'_>,
    kind: &DataKind,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let data_kind_class = data_module.getattr("DataKind")?;

    match kind {
        DataKind::Trade(trade) => {
            let trade_obj = public_trade_to_py(py, trade, data_module, instrument_module)?;
            Ok(data_kind_class
                .call_method1("trade", (trade_obj,))?
                .into_py(py))
        }
        DataKind::OrderBookL1(l1) => {
            let ob = order_book_l1_to_py(py, l1, data_module)?;
            Ok(data_kind_class
                .call_method1("order_book_l1", (ob,))?
                .into_py(py))
        }
        DataKind::OrderBook(event) => {
            let order_book_event_enum = data_module.getattr("OrderBookEvent")?;
            let variant = match event {
                barter_data::subscription::book::OrderBookEvent::Snapshot(_) => {
                    order_book_event_enum.getattr("SNAPSHOT")?
                }
                barter_data::subscription::book::OrderBookEvent::Update(_) => {
                    order_book_event_enum.getattr("UPDATE")?
                }
            };
            Ok(data_kind_class
                .call_method1("order_book", (variant,))?
                .into_py(py))
        }
        DataKind::Candle(candle) => {
            let candle_obj = candle_to_py(py, candle, data_module)?;
            Ok(data_kind_class
                .call_method1("candle", (candle_obj,))?
                .into_py(py))
        }
        DataKind::Liquidation(liquidation) => {
            let liquidation_obj =
                liquidation_to_py(py, liquidation, data_module, instrument_module)?;
            Ok(data_kind_class
                .call_method1("liquidation", (liquidation_obj,))?
                .into_py(py))
        }
    }
}

fn public_trade_to_py(
    py: Python<'_>,
    trade: &barter_data::subscription::trade::PublicTrade,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let public_trade_class = data_module.getattr("PublicTrade")?;
    let side = side_to_py(py, trade.side, instrument_module)?;

    let value = public_trade_class.call1((trade.id.as_str(), trade.price, trade.amount, side))?;
    Ok(value.into_py(py))
}

fn order_book_l1_to_py(
    py: Python<'_>,
    l1: &barter_data::subscription::book::OrderBookL1,
    data_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let order_book_l1_class = data_module.getattr("OrderBookL1")?;
    let level_class = data_module.getattr("Level")?;

    let best_bid = l1
        .best_bid
        .as_ref()
        .map(|level| level_to_py(py, level, &level_class))
        .transpose()?;
    let best_ask = l1
        .best_ask
        .as_ref()
        .map(|level| level_to_py(py, level, &level_class))
        .transpose()?;

    let args = (
        l1.last_update_time,
        best_bid.unwrap_or_else(|| py.None()),
        best_ask.unwrap_or_else(|| py.None()),
    );

    let value = order_book_l1_class.call1(args)?;
    Ok(value.into_py(py))
}

fn level_to_py(
    py: Python<'_>,
    level: &barter_data::books::Level,
    level_class: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let price = decimal_to_py(py, level.price)?;
    let amount = decimal_to_py(py, level.amount)?;
    let level_obj = level_class.call1((price, amount))?;
    Ok(level_obj.into_py(py))
}

fn candle_to_py(
    py: Python<'_>,
    candle: &barter_data::subscription::candle::Candle,
    data_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let candle_class = data_module.getattr("Candle")?;
    let value = candle_class.call1((
        candle.close_time,
        candle.open,
        candle.high,
        candle.low,
        candle.close,
        candle.volume,
        candle.trade_count,
    ))?;
    Ok(value.into_py(py))
}

fn liquidation_to_py(
    py: Python<'_>,
    liquidation: &barter_data::subscription::liquidation::Liquidation,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let liquidation_class = data_module.getattr("Liquidation")?;
    let side = side_to_py(py, liquidation.side, instrument_module)?;
    let value = liquidation_class.call1((
        side,
        liquidation.price,
        liquidation.quantity,
        liquidation.time,
    ))?;
    Ok(value.into_py(py))
}

fn side_to_py(
    py: Python<'_>,
    side: Side,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<PyObject> {
    let side_class = instrument_module.getattr("Side")?;
    let attr = match side {
        Side::Buy => "BUY",
        Side::Sell => "SELL",
    };
    Ok(side_class.getattr(attr)?.into_py(py))
}

fn parse_market_event_from_py(
    py: Python<'_>,
    obj: &PyObject,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
    decimal_class: &Bound<'_, PyAny>,
) -> PyResult<MarketEvent<InstrumentIndex, DataKind>> {
    let market_event_class = data_module.getattr("MarketEvent")?;
    let bound = obj.bind(py);

    if !bound.is_instance(&market_event_class)? {
        return Err(PyValueError::new_err(
            "expected barter_python.data.MarketEvent instance",
        ));
    }

    let time_exchange: DateTime<Utc> = bound.getattr("time_exchange")?.extract()?;
    let time_received: DateTime<Utc> = bound.getattr("time_received")?.extract()?;
    let exchange: String = bound.getattr("exchange")?.extract()?;
    let instrument_index: usize = bound.getattr("instrument")?.extract()?;
    let kind_obj = bound.getattr("kind")?.into_py(py);

    let kind = data_kind_from_py(py, &kind_obj, data_module, instrument_module, decimal_class)?;

    let exchange_id = serde_json::from_str::<ExchangeId>(&format!("\"{}\"", exchange))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    Ok(MarketEvent {
        time_exchange,
        time_received,
        exchange: exchange_id,
        instrument: InstrumentIndex(instrument_index),
        kind,
    })
}

fn data_kind_from_py(
    py: Python<'_>,
    obj: &PyObject,
    data_module: &Bound<'_, PyModule>,
    instrument_module: &Bound<'_, PyModule>,
    decimal_class: &Bound<'_, PyAny>,
) -> PyResult<DataKind> {
    let bound = obj.bind(py);
    let data_kind_class = data_module.getattr("DataKind")?;
    if !bound.is_instance(&data_kind_class)? {
        return Err(PyValueError::new_err(
            "expected barter_python.data.DataKind instance for kind",
        ));
    }

    let kind_name: String = bound.getattr("kind")?.extract()?;
    let data = bound.getattr("data")?;

    match kind_name.as_str() {
        "trade" => {
            let trade_class = data_module.getattr("PublicTrade")?;
            if !data.is_instance(&trade_class)? {
                return Err(PyValueError::new_err("expected PublicTrade for trade kind"));
            }

            let id: String = data.getattr("id")?.extract()?;
            let price: f64 = data.getattr("price")?.extract()?;
            let amount: f64 = data.getattr("amount")?.extract()?;
            let side_obj = data.getattr("side")?.into_py(py);
            let side = side_from_py(py, &side_obj, instrument_module)?;

            Ok(DataKind::Trade(
                barter_data::subscription::trade::PublicTrade {
                    id,
                    price,
                    amount,
                    side,
                },
            ))
        }
        "order_book_l1" => {
            let order_book_l1_class = data_module.getattr("OrderBookL1")?;
            if !data.is_instance(&order_book_l1_class)? {
                return Err(PyValueError::new_err("expected OrderBookL1 for l1 kind"));
            }

            let last_update_time: DateTime<Utc> = data.getattr("last_update_time")?.extract()?;
            let best_bid = level_option_from_py(
                py,
                data.getattr("best_bid")?.into_py(py),
                data_module,
                decimal_class,
            )?;
            let best_ask = level_option_from_py(
                py,
                data.getattr("best_ask")?.into_py(py),
                data_module,
                decimal_class,
            )?;

            Ok(DataKind::OrderBookL1(
                barter_data::subscription::book::OrderBookL1 {
                    last_update_time,
                    best_bid,
                    best_ask,
                },
            ))
        }
        "order_book" => {
            let event = data.extract::<String>()?;
            let variant = match event.to_ascii_uppercase().as_str() {
                "SNAPSHOT" => {
                    let snapshot = barter_data::books::OrderBook::new(
                        0,
                        None,
                        Vec::<barter_data::books::Level>::new(),
                        Vec::<barter_data::books::Level>::new(),
                    );
                    barter_data::subscription::book::OrderBookEvent::Snapshot(snapshot)
                }
                "UPDATE" => {
                    let update = barter_data::books::OrderBook::new(
                        0,
                        None,
                        Vec::<barter_data::books::Level>::new(),
                        Vec::<barter_data::books::Level>::new(),
                    );
                    barter_data::subscription::book::OrderBookEvent::Update(update)
                }
                other => {
                    return Err(PyValueError::new_err(format!(
                        "unsupported OrderBookEvent variant '{}' when round-tripping",
                        other
                    )));
                }
            };
            Ok(DataKind::OrderBook(variant))
        }
        "candle" => {
            let candle_class = data_module.getattr("Candle")?;
            if !data.is_instance(&candle_class)? {
                return Err(PyValueError::new_err("expected Candle for candle kind"));
            }

            Ok(DataKind::Candle(
                barter_data::subscription::candle::Candle {
                    close_time: data.getattr("close_time")?.extract()?,
                    open: data.getattr("open")?.extract()?,
                    high: data.getattr("high")?.extract()?,
                    low: data.getattr("low")?.extract()?,
                    close: data.getattr("close")?.extract()?,
                    volume: data.getattr("volume")?.extract()?,
                    trade_count: data.getattr("trade_count")?.extract()?,
                },
            ))
        }
        "liquidation" => {
            let liquidation_class = data_module.getattr("Liquidation")?;
            if !data.is_instance(&liquidation_class)? {
                return Err(PyValueError::new_err(
                    "expected Liquidation for liquidation kind",
                ));
            }

            let side = side_from_py(py, &data.getattr("side")?.into_py(py), instrument_module)?;
            Ok(DataKind::Liquidation(
                barter_data::subscription::liquidation::Liquidation {
                    side,
                    price: data.getattr("price")?.extract()?,
                    quantity: data.getattr("quantity")?.extract()?,
                    time: data.getattr("time")?.extract()?,
                },
            ))
        }
        other => Err(PyValueError::new_err(format!(
            "unsupported DataKind variant '{}' when round-tripping",
            other
        ))),
    }
}

fn level_option_from_py(
    py: Python<'_>,
    obj: PyObject,
    data_module: &Bound<'_, PyModule>,
    decimal_class: &Bound<'_, PyAny>,
) -> PyResult<Option<barter_data::books::Level>> {
    if obj.is_none(py) {
        return Ok(None);
    }

    let level_class = data_module.getattr("Level")?;
    let bound = obj.bind(py);
    if !bound.is_instance(&level_class)? {
        return Err(PyValueError::new_err("expected Level for order book level"));
    }

    let price = decimal_from_py(py, bound.getattr("price")?.into_py(py), decimal_class)?;
    let amount = decimal_from_py(py, bound.getattr("amount")?.into_py(py), decimal_class)?;

    Ok(Some(barter_data::books::Level::new(price, amount)))
}

fn decimal_from_py(
    py: Python<'_>,
    obj: PyObject,
    decimal_class: &Bound<'_, PyAny>,
) -> PyResult<Decimal> {
    let bound = obj.bind(py);
    if bound.is_instance(decimal_class)? {
        let as_str: String = bound.call_method0("__str__")?.extract()?;
        as_str
            .parse::<Decimal>()
            .map_err(|err| PyValueError::new_err(err.to_string()))
    } else if let Ok(value) = bound.extract::<f64>() {
        Decimal::from_f64(value).ok_or_else(|| {
            PyValueError::new_err("unable to convert float to Decimal without precision loss")
        })
    } else if let Ok(value) = bound.extract::<i64>() {
        Ok(Decimal::from_i64(value).expect("i64 always converts to Decimal"))
    } else {
        Err(PyValueError::new_err(
            "unsupported numeric type for Decimal conversion",
        ))
    }
}

fn side_from_py(
    py: Python<'_>,
    obj: &PyObject,
    instrument_module: &Bound<'_, PyModule>,
) -> PyResult<Side> {
    if let Ok(side) = obj.bind(py).extract::<String>() {
        match side.to_ascii_lowercase().as_str() {
            "buy" => Ok(Side::Buy),
            "sell" => Ok(Side::Sell),
            other => Err(PyValueError::new_err(format!(
                "invalid side value '{}' when round-tripping",
                other
            ))),
        }
    } else {
        let side_class = instrument_module.getattr("Side")?;
        if obj.bind(py).is_instance(&side_class)? {
            let value: String = obj.bind(py).getattr("value")?.extract()?;
            match value.as_str() {
                "buy" => Ok(Side::Buy),
                "sell" => Ok(Side::Sell),
                other => Err(PyValueError::new_err(format!(
                    "invalid side value '{}' when round-tripping",
                    other
                ))),
            }
        } else {
            Err(PyValueError::new_err(
                "expected barter_python.instrument.Side or string for side",
            ))
        }
    }
}
