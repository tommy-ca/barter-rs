#![allow(unused_imports)]

use crate::{backtest::market_event_to_py, command::parse_decimal};
use barter_data::{
    event::{DataKind, MarketEvent},
    instrument::InstrumentData,
    streams::{builder::dynamic::DynamicStreams, consumer::MarketStreamResult, reconnect::Event},
    subscription::{
        SubKind, Subscription,
        exchange_supports_instrument_kind as rust_exchange_supports_instrument_kind,
        trade::PublicTrade,
    },
};
use barter_instrument::{
    Keyed, Side,
    exchange::ExchangeId,
    instrument::{
        InstrumentIndex,
        kind::option::{OptionExercise, OptionKind},
        market_data::{
            MarketDataInstrument,
            kind::{MarketDataFutureContract, MarketDataInstrumentKind, MarketDataOptionContract},
        },
    },
};
use barter_integration::subscription::SubscriptionId;
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use pyo3::{
    Bound,
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict, PyModule},
};
#[cfg(feature = "python-tests")]
use serde_json;
#[cfg(feature = "python-tests")]
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use tokio::sync::mpsc::{self, UnboundedReceiver, error::TryRecvError};
use tokio::time::Duration;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vecmap::VecMap;

/// Wrapper around [`ExchangeId`] for Python exposure.
#[pyclass(module = "barter_python", name = "ExchangeId", eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExchangeId {
    inner: ExchangeId,
}

impl PyExchangeId {
    pub(crate) fn as_inner(&self) -> ExchangeId {
        self.inner
    }

    pub(crate) fn from_inner(inner: ExchangeId) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyExchangeId {
    /// Other / unknown exchange.
    #[classattr]
    const OTHER: Self = Self {
        inner: ExchangeId::Other,
    };

    /// Simulated exchange environment.
    #[classattr]
    const SIMULATED: Self = Self {
        inner: ExchangeId::Simulated,
    };

    /// Mock exchange implementation.
    #[classattr]
    const MOCK: Self = Self {
        inner: ExchangeId::Mock,
    };

    /// Binance coin-margined futures exchange.
    #[classattr]
    const BINANCE_FUTURES_COIN: Self = Self {
        inner: ExchangeId::BinanceFuturesCoin,
    };

    /// Binance Spot exchange.
    #[classattr]
    const BINANCE_SPOT: Self = Self {
        inner: ExchangeId::BinanceSpot,
    };

    /// Binance USD-margined futures exchange.
    #[classattr]
    const BINANCE_FUTURES_USD: Self = Self {
        inner: ExchangeId::BinanceFuturesUsd,
    };

    /// Binance Options venue.
    #[classattr]
    const BINANCE_OPTIONS: Self = Self {
        inner: ExchangeId::BinanceOptions,
    };

    /// Binance Portfolio Margin venue.
    #[classattr]
    const BINANCE_PORTFOLIO_MARGIN: Self = Self {
        inner: ExchangeId::BinancePortfolioMargin,
    };

    /// Binance US exchange.
    #[classattr]
    const BINANCE_US: Self = Self {
        inner: ExchangeId::BinanceUs,
    };

    /// Bitazza exchange.
    #[classattr]
    const BITAZZA: Self = Self {
        inner: ExchangeId::Bitazza,
    };

    /// Bitfinex exchange.
    #[classattr]
    const BITFINEX: Self = Self {
        inner: ExchangeId::Bitfinex,
    };

    /// Bitflyer exchange.
    #[classattr]
    const BITFLYER: Self = Self {
        inner: ExchangeId::Bitflyer,
    };

    /// Bitget exchange.
    #[classattr]
    const BITGET: Self = Self {
        inner: ExchangeId::Bitget,
    };

    /// Bitmart spot exchange.
    #[classattr]
    const BITMART: Self = Self {
        inner: ExchangeId::Bitmart,
    };

    /// Bitmart USD-margined futures exchange.
    #[classattr]
    const BITMART_FUTURES_USD: Self = Self {
        inner: ExchangeId::BitmartFuturesUsd,
    };

    /// BitMEX exchange.
    #[classattr]
    const BITMEX: Self = Self {
        inner: ExchangeId::Bitmex,
    };

    /// Bitso exchange.
    #[classattr]
    const BITSO: Self = Self {
        inner: ExchangeId::Bitso,
    };

    /// Bitstamp exchange.
    #[classattr]
    const BITSTAMP: Self = Self {
        inner: ExchangeId::Bitstamp,
    };

    /// Bitvavo exchange.
    #[classattr]
    const BITVAVO: Self = Self {
        inner: ExchangeId::Bitvavo,
    };

    /// Bithumb exchange.
    #[classattr]
    const BITHUMB: Self = Self {
        inner: ExchangeId::Bithumb,
    };

    /// Bybit Spot exchange.
    #[classattr]
    const BYBIT_SPOT: Self = Self {
        inner: ExchangeId::BybitSpot,
    };

    /// Bybit Perpetuals USD exchange.
    #[classattr]
    const BYBIT_PERPETUALS_USD: Self = Self {
        inner: ExchangeId::BybitPerpetualsUsd,
    };

    /// CEX.io exchange.
    #[classattr]
    const CEXIO: Self = Self {
        inner: ExchangeId::Cexio,
    };

    /// Coinbase exchange.
    #[classattr]
    const COINBASE: Self = Self {
        inner: ExchangeId::Coinbase,
    };

    /// Coinbase International exchange.
    #[classattr]
    const COINBASE_INTERNATIONAL: Self = Self {
        inner: ExchangeId::CoinbaseInternational,
    };

    /// Crypto.com exchange.
    #[classattr]
    const CRYPTOCOM: Self = Self {
        inner: ExchangeId::Cryptocom,
    };

    /// Deribit exchange.
    #[classattr]
    const DERIBIT: Self = Self {
        inner: ExchangeId::Deribit,
    };

    /// Gate.io Futures BTC exchange.
    #[classattr]
    const GATEIO_FUTURES_BTC: Self = Self {
        inner: ExchangeId::GateioFuturesBtc,
    };

    /// Gate.io Spot exchange.
    #[classattr]
    const GATEIO_SPOT: Self = Self {
        inner: ExchangeId::GateioSpot,
    };

    /// Gate.io Futures USD exchange.
    #[classattr]
    const GATEIO_FUTURES_USD: Self = Self {
        inner: ExchangeId::GateioFuturesUsd,
    };

    /// Gate.io Perpetuals USD exchange.
    #[classattr]
    const GATEIO_PERPETUALS_USD: Self = Self {
        inner: ExchangeId::GateioPerpetualsUsd,
    };

    /// Gate.io Perpetuals BTC exchange.
    #[classattr]
    const GATEIO_PERPETUALS_BTC: Self = Self {
        inner: ExchangeId::GateioPerpetualsBtc,
    };

    /// Gate.io Options exchange.
    #[classattr]
    const GATEIO_OPTIONS: Self = Self {
        inner: ExchangeId::GateioOptions,
    };

    /// Gemini exchange.
    #[classattr]
    const GEMINI: Self = Self {
        inner: ExchangeId::Gemini,
    };

    /// HitBTC exchange.
    #[classattr]
    const HITBTC: Self = Self {
        inner: ExchangeId::Hitbtc,
    };

    /// Kraken exchange.
    #[classattr]
    const KRAKEN: Self = Self {
        inner: ExchangeId::Kraken,
    };

    /// HTX (Huobi) exchange.
    #[classattr]
    const HTX: Self = Self {
        inner: ExchangeId::Htx,
    };

    /// Kucoin exchange.
    #[classattr]
    const KUCOIN: Self = Self {
        inner: ExchangeId::Kucoin,
    };

    /// Liquid exchange.
    #[classattr]
    const LIQUID: Self = Self {
        inner: ExchangeId::Liquid,
    };

    /// MEXC exchange.
    #[classattr]
    const MEXC: Self = Self {
        inner: ExchangeId::Mexc,
    };

    /// OKX exchange.
    #[classattr]
    const OKX: Self = Self {
        inner: ExchangeId::Okx,
    };

    /// Poloniex exchange.
    #[classattr]
    const POLONIEX: Self = Self {
        inner: ExchangeId::Poloniex,
    };

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("ExchangeId.{:?}", self.inner)
    }
}

/// Wrapper around [`SubKind`] for Python exposure.
#[pyclass(module = "barter_python", name = "SubKind", eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySubKind {
    inner: SubKind,
}

#[pymethods]
impl PySubKind {
    /// Public trades subscription.
    #[classattr]
    const PUBLIC_TRADES: Self = Self {
        inner: SubKind::PublicTrades,
    };

    /// Order book L1 subscription.
    #[classattr]
    const ORDER_BOOKS_L1: Self = Self {
        inner: SubKind::OrderBooksL1,
    };

    /// Order book L2 subscription.
    #[classattr]
    const ORDER_BOOKS_L2: Self = Self {
        inner: SubKind::OrderBooksL2,
    };

    /// Order book L3 subscription.
    #[classattr]
    const ORDER_BOOKS_L3: Self = Self {
        inner: SubKind::OrderBooksL3,
    };

    /// Liquidations subscription.
    #[classattr]
    const LIQUIDATIONS: Self = Self {
        inner: SubKind::Liquidations,
    };

    /// Candles subscription.
    #[classattr]
    const CANDLES: Self = Self {
        inner: SubKind::Candles,
    };

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("SubKind.{:?}", self.inner)
    }
}

/// Wrapper around [`Subscription`] for Python exposure.
#[pyclass(module = "barter_python", name = "Subscription", unsendable)]
#[derive(Debug, Clone)]
pub struct PySubscription {
    inner: Subscription<ExchangeId, MarketDataInstrument, SubKind>,
}

#[pymethods]
impl PySubscription {
    /// Create a new subscription.
    #[new]
    #[pyo3(signature = (exchange, base, quote, kind, instrument_kind=None))]
    fn new(
        exchange: &PyExchangeId,
        base: &str,
        quote: &str,
        kind: &PySubKind,
        instrument_kind: Option<Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let instrument_kind = parse_market_data_instrument_kind(instrument_kind)?;

        let instrument = MarketDataInstrument::from((base, quote, instrument_kind));
        let subscription = Subscription::new(exchange.inner, instrument, kind.inner);

        Ok(Self {
            inner: subscription,
        })
    }

    /// Get the exchange.
    #[getter]
    fn exchange(&self) -> PyExchangeId {
        PyExchangeId {
            inner: self.inner.exchange,
        }
    }

    /// Get the instrument.
    #[getter]
    fn instrument(&self) -> String {
        self.inner.instrument.to_string()
    }

    /// Get the subscription kind.
    #[getter]
    fn kind(&self) -> PySubKind {
        PySubKind {
            inner: self.inner.kind,
        }
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        format!(
            "Subscription(exchange={}, instrument={}, kind={})",
            self.inner.exchange,
            self.instrument(),
            self.inner.kind
        )
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }

    /// Determine whether the exchange supports this instrument kind.
    pub fn is_supported(&self) -> bool {
        rust_exchange_supports_instrument_kind(self.inner.exchange, self.inner.instrument.kind())
    }
}

fn parse_market_data_instrument_kind(
    kind: Option<Bound<'_, PyAny>>,
) -> PyResult<MarketDataInstrumentKind> {
    match kind {
        None => Ok(MarketDataInstrumentKind::Spot),
        Some(value) => {
            if let Ok(kind_str) = value.extract::<&str>() {
                match kind_str.to_ascii_lowercase().as_str() {
                    "spot" => Ok(MarketDataInstrumentKind::Spot),
                    "perpetual" => Ok(MarketDataInstrumentKind::Perpetual),
                    other => Err(PyValueError::new_err(format!(
                        "Invalid instrument_kind '{}'. Use 'spot', 'perpetual', or a mapping with type",
                        other
                    ))),
                }
            } else if let Ok(mapping) = value.downcast::<PyDict>() {
                parse_instrument_kind_mapping(mapping)
            } else {
                Err(PyValueError::new_err(
                    "Invalid instrument_kind. Provide 'spot', 'perpetual', or a mapping with 'type'",
                ))
            }
        }
    }
}

fn parse_instrument_kind_mapping(dict: &Bound<'_, PyDict>) -> PyResult<MarketDataInstrumentKind> {
    let type_binding = get_required(dict, "type")?;
    let kind_value = type_binding
        .extract::<&str>()
        .map_err(|_| PyValueError::new_err("instrument_kind 'type' must be a string"))?;

    match kind_value.to_ascii_lowercase().as_str() {
        "future" => parse_future_kind(dict),
        "option" => parse_option_kind(dict),
        other => Err(PyValueError::new_err(format!(
            "Unsupported instrument_kind type '{}'. Expected 'future' or 'option'",
            other
        ))),
    }
}

fn parse_future_kind(dict: &Bound<'_, PyDict>) -> PyResult<MarketDataInstrumentKind> {
    let expiry_any = get_required(dict, "expiry")?;
    let expiry = parse_datetime_field(expiry_any, "expiry")?;

    Ok(MarketDataInstrumentKind::Future(MarketDataFutureContract {
        expiry,
    }))
}

fn parse_option_kind(dict: &Bound<'_, PyDict>) -> PyResult<MarketDataInstrumentKind> {
    let option_kind = parse_option_kind_field(get_required(dict, "kind")?)?;
    let exercise = parse_option_exercise_field(get_required(dict, "exercise")?)?;
    let expiry = parse_datetime_field(get_required(dict, "expiry")?, "expiry")?;
    let strike_any = get_required(dict, "strike")?;

    let strike_float = strike_any
        .extract::<f64>()
        .or_else(|_| strike_any.extract::<i64>().map(|value| value as f64))
        .map_err(|_| PyValueError::new_err("instrument_kind option 'strike' must be numeric"))?;
    let strike = parse_decimal(strike_float, "strike")?;

    Ok(MarketDataInstrumentKind::Option(MarketDataOptionContract {
        kind: option_kind,
        exercise,
        expiry,
        strike,
    }))
}

fn parse_option_kind_field(value: Bound<'_, PyAny>) -> PyResult<OptionKind> {
    let kind = value
        .extract::<&str>()
        .map_err(|_| PyValueError::new_err("instrument_kind option 'kind' must be a string"))?
        .to_ascii_lowercase();

    match kind.as_str() {
        "call" => Ok(OptionKind::Call),
        "put" => Ok(OptionKind::Put),
        other => Err(PyValueError::new_err(format!(
            "Unsupported option kind '{}'. Expected 'call' or 'put'",
            other
        ))),
    }
}

fn parse_option_exercise_field(value: Bound<'_, PyAny>) -> PyResult<OptionExercise> {
    let exercise = value
        .extract::<&str>()
        .map_err(|_| PyValueError::new_err("instrument_kind option 'exercise' must be a string"))?
        .to_ascii_lowercase();

    match exercise.as_str() {
        "american" => Ok(OptionExercise::American),
        "bermudan" => Ok(OptionExercise::Bermudan),
        "european" => Ok(OptionExercise::European),
        other => Err(PyValueError::new_err(format!(
            "Unsupported option exercise '{}'. Expected 'american', 'bermudan', or 'european'",
            other
        ))),
    }
}

fn parse_datetime_field(value: Bound<'_, PyAny>, field: &str) -> PyResult<DateTime<Utc>> {
    if let Ok(datetime) = value.extract::<DateTime<Utc>>() {
        return Ok(datetime);
    }

    if let Ok(text) = value.extract::<String>() {
        DateTime::parse_from_rfc3339(&text)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|err| PyValueError::new_err(format!("Invalid {} '{}': {}", field, text, err)))
    } else {
        Err(PyValueError::new_err(format!(
            "Invalid {} value. Expected datetime or ISO8601 string",
            field
        )))
    }
}

fn get_required<'py>(dict: &Bound<'py, PyDict>, field: &str) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(field)?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "instrument_kind mapping missing required '{}' field",
            field
        ))
    })
}

/// Wrapper around [`SubscriptionId`] for Python exposure.
#[pyclass(module = "barter_python", name = "SubscriptionId", eq, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySubscriptionId {
    pub(crate) inner: SubscriptionId,
}

#[pymethods]
impl PySubscriptionId {
    /// Create a new [`SubscriptionId`].
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: SubscriptionId::from(id),
        }
    }

    /// Get the string value.
    #[getter]
    fn value(&self) -> &str {
        self.inner.0.as_str()
    }

    /// Return the string representation.
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the debug representation.
    fn __repr__(&self) -> String {
        format!("SubscriptionId('{}')", self.inner)
    }
}

impl PySubscriptionId {
    /// Create a new [`SubscriptionId`] for testing.
    pub fn new_test(id: &str) -> Self {
        Self {
            inner: SubscriptionId::from(id),
        }
    }
}

/// Wrapper around [`DynamicStreams`] for Python exposure.
#[pyclass(module = "barter_python", name = "DynamicStreams", unsendable)]
#[derive(Debug)]
pub struct PyDynamicStreams {
    runtime: Arc<Runtime>,
    inner: Mutex<Option<DynamicStreams<InstrumentIndex>>>,
}

impl PyDynamicStreams {
    pub(crate) fn from_parts(
        runtime: Arc<Runtime>,
        streams: DynamicStreams<InstrumentIndex>,
    ) -> Self {
        Self {
            runtime,
            inner: Mutex::new(Some(streams)),
        }
    }

    fn with_streams<R, T>(&self, func: R) -> PyResult<Option<T>>
    where
        R: FnOnce(&mut DynamicStreams<InstrumentIndex>) -> Option<T>,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| PyValueError::new_err("dynamic streams mutex poisoned"))?;

        Ok(guard.as_mut().and_then(func))
    }

    fn select_stream<Kind, S, F>(&self, extractor: F) -> PyResult<Option<PyMarketStream>>
    where
        Kind: 'static,
        MarketStreamResult<InstrumentIndex, Kind>:
            Into<MarketStreamResult<InstrumentIndex, DataKind>>,
        S: Stream<Item = MarketStreamResult<InstrumentIndex, Kind>> + Send + 'static,
        F: FnOnce(&mut DynamicStreams<InstrumentIndex>) -> Option<S>,
    {
        let runtime = Arc::clone(&self.runtime);
        let stream = self.with_streams(|streams| extractor(streams))?;

        stream
            .map(|stream| {
                let mapped = stream.map(|event| event.into());
                Ok(PyMarketStream::new(runtime, mapped))
            })
            .transpose()
    }
}

#[pymethods]
impl PyDynamicStreams {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = Arc::new(
            RuntimeBuilder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        );

        Ok(Self {
            runtime,
            inner: Mutex::new(Some(DynamicStreams {
                trades: VecMap::default(),
                l1s: VecMap::default(),
                l2s: VecMap::default(),
                liquidations: VecMap::default(),
            })),
        })
    }

    fn select_trades(&self, exchange: &PyExchangeId) -> PyResult<Option<PyMarketStream>> {
        self.select_stream(|streams| streams.select_trades(exchange.inner))
    }

    fn select_all_trades(&self) -> PyResult<PyMarketStream> {
        let runtime = Arc::clone(&self.runtime);
        let stream = self
            .with_streams(|streams| Some(streams.select_all_trades()))?
            .ok_or_else(|| PyValueError::new_err("no trade streams available"))?;
        let mapped = stream.map(|event| event.into());
        Ok(PyMarketStream::new(runtime, mapped))
    }

    fn select_l1s(&self, exchange: &PyExchangeId) -> PyResult<Option<PyMarketStream>> {
        self.select_stream(|streams| streams.select_l1s(exchange.inner))
    }

    fn select_all_l1s(&self) -> PyResult<PyMarketStream> {
        let runtime = Arc::clone(&self.runtime);
        let stream = self
            .with_streams(|streams| Some(streams.select_all_l1s()))?
            .ok_or_else(|| PyValueError::new_err("no order book L1 streams available"))?;
        let mapped = stream.map(|event| event.into());
        Ok(PyMarketStream::new(runtime, mapped))
    }

    fn select_l2s(&self, exchange: &PyExchangeId) -> PyResult<Option<PyMarketStream>> {
        self.select_stream(|streams| streams.select_l2s(exchange.inner))
    }

    fn select_all_l2s(&self) -> PyResult<PyMarketStream> {
        let runtime = Arc::clone(&self.runtime);
        let stream = self
            .with_streams(|streams| Some(streams.select_all_l2s()))?
            .ok_or_else(|| PyValueError::new_err("no order book streams available"))?;
        let mapped = stream.map(|event| event.into());
        Ok(PyMarketStream::new(runtime, mapped))
    }

    fn select_liquidations(&self, exchange: &PyExchangeId) -> PyResult<Option<PyMarketStream>> {
        self.select_stream(|streams| streams.select_liquidations(exchange.inner))
    }

    fn select_all_liquidations(&self) -> PyResult<PyMarketStream> {
        let runtime = Arc::clone(&self.runtime);
        let stream = self
            .with_streams(|streams| Some(streams.select_all_liquidations()))?
            .ok_or_else(|| PyValueError::new_err("no liquidation streams available"))?;
        let mapped = stream.map(|event| event.into());
        Ok(PyMarketStream::new(runtime, mapped))
    }
}

#[pyclass(module = "barter_python", name = "MarketStream", unsendable)]
pub struct PyMarketStream {
    runtime: Arc<Runtime>,
    receiver: Mutex<Option<UnboundedReceiver<MarketStreamResult<InstrumentIndex, DataKind>>>>,
}

impl PyMarketStream {
    fn new(
        runtime: Arc<Runtime>,
        stream: impl Stream<Item = MarketStreamResult<InstrumentIndex, DataKind>> + Send + 'static,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let runtime_clone = Arc::clone(&runtime);

        runtime.spawn(async move {
            futures::pin_mut!(stream);
            while let Some(event) = stream.next().await {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });

        Self {
            runtime: runtime_clone,
            receiver: Mutex::new(Some(rx)),
        }
    }

    fn recv_inner(
        &self,
        timeout: Option<f64>,
    ) -> PyResult<Option<MarketStreamResult<InstrumentIndex, DataKind>>> {
        let mut guard = self
            .receiver
            .lock()
            .map_err(|_| PyValueError::new_err("market stream mutex poisoned"))?;

        let receiver = match guard.as_mut() {
            Some(rx) => rx,
            None => return Ok(None),
        };

        let runtime = Arc::clone(&self.runtime);

        let item = if let Some(secs) = timeout {
            if secs.is_sign_negative() {
                return Err(PyValueError::new_err("timeout must be non-negative"));
            }

            let duration = Duration::from_secs_f64(secs);
            runtime
                .block_on(async { tokio::time::timeout(duration, receiver.recv()).await })
                .map_err(|_| PyValueError::new_err("timeout elapsed awaiting market event"))?
        } else {
            runtime.block_on(receiver.recv())
        };

        if item.is_none() {
            *guard = None;
        }

        Ok(item)
    }
}

#[pymethods]
impl PyMarketStream {
    #[pyo3(signature = (timeout = None))]
    pub fn recv(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Option<PyObject>> {
        let event = self.recv_inner(timeout)?;
        match event {
            Some(event) => market_stream_result_to_py(py, event).map(Some),
            None => Ok(None),
        }
    }

    pub fn try_recv(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let mut guard = self
            .receiver
            .lock()
            .map_err(|_| PyValueError::new_err("market stream mutex poisoned"))?;

        let receiver = match guard.as_mut() {
            Some(rx) => rx,
            None => return Ok(None),
        };

        match receiver.try_recv() {
            Ok(event) => market_stream_result_to_py(py, event).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                *guard = None;
                Ok(None)
            }
        }
    }

    pub fn is_closed(&self) -> PyResult<bool> {
        let guard = self
            .receiver
            .lock()
            .map_err(|_| PyValueError::new_err("market stream mutex poisoned"))?;
        Ok(guard
            .as_ref()
            .map(|receiver| receiver.is_closed())
            .unwrap_or(true))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(if self.is_closed()? {
            "MarketStream(closed=True)".to_string()
        } else {
            "MarketStream(closed=False)".to_string()
        })
    }
}

#[pyfunction]
#[pyo3(signature = (subscriptions))]
pub fn init_dynamic_streams(
    _py: Python<'_>,
    subscriptions: Vec<Vec<PySubscription>>,
) -> PyResult<PyDynamicStreams> {
    let runtime = Arc::new(
        RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?,
    );

    let mut index_map: HashMap<(ExchangeId, MarketDataInstrument), InstrumentIndex> =
        HashMap::new();
    let mut next_index = 0usize;

    let converted: Vec<
        Vec<Subscription<ExchangeId, Keyed<InstrumentIndex, MarketDataInstrument>, SubKind>>,
    > = subscriptions
        .into_iter()
        .map(|batch| {
            batch
                .into_iter()
                .map(|sub| {
                    let inner = sub.inner.clone();
                    let key = (inner.exchange, inner.instrument.clone());
                    let entry = index_map.entry(key).or_insert_with(|| {
                        let current = InstrumentIndex(next_index);
                        next_index += 1;
                        current
                    });

                    let keyed: Keyed<InstrumentIndex, MarketDataInstrument> =
                        Keyed::new(*entry, inner.instrument);

                    Subscription::new(inner.exchange, keyed, inner.kind)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let streams = runtime
        .block_on(DynamicStreams::init(converted))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    Ok(PyDynamicStreams::from_parts(runtime, streams))
}

fn market_stream_result_to_py(
    py: Python<'_>,
    event: MarketStreamResult<InstrumentIndex, DataKind>,
) -> PyResult<PyObject> {
    let data_module = PyModule::import_bound(py, "barter_python.data")?;

    match event {
        Event::Reconnecting(exchange) => {
            let reconnecting = data_module.getattr("MarketStreamReconnecting")?;
            let constructed = reconnecting.call1((exchange.as_str(),))?;
            Ok(constructed.into_py(py))
        }
        Event::Item(result) => match result {
            Ok(event) => {
                let instrument_module = PyModule::import_bound(py, "barter_python.instrument")?;
                let market_event =
                    market_event_to_py(py, &event, &data_module, &instrument_module)?;
                let item_class = data_module.getattr("MarketStreamItem")?;
                let constructed = item_class.call1((market_event,))?;
                Ok(constructed.into_py(py))
            }
            Err(error) => Err(PyValueError::new_err(error.to_string())),
        },
    }
}

#[pyfunction]
#[pyo3(signature = (exchange, instrument_kind))]
pub fn exchange_supports_instrument_kind(
    exchange: &PyExchangeId,
    instrument_kind: Bound<'_, PyAny>,
) -> PyResult<bool> {
    let kind = parse_market_data_instrument_kind(Some(instrument_kind))?;
    Ok(rust_exchange_supports_instrument_kind(
        exchange.inner,
        &kind,
    ))
}

#[cfg(feature = "python-tests")]
fn parse_exchange_id(value: &str) -> PyResult<ExchangeId> {
    let serialized = format!("\"{}\"", value);
    serde_json::from_str(&serialized).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[cfg(feature = "python-tests")]
fn parse_side(value: &str) -> PyResult<Side> {
    match value.to_ascii_lowercase().as_str() {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        other => Err(PyValueError::new_err(format!(
            "invalid side '{}'; expected 'buy' or 'sell'",
            other
        ))),
    }
}

#[cfg(feature = "python-tests")]
fn parse_trade_event(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
) -> PyResult<(ExchangeId, MarketStreamResult<InstrumentIndex, PublicTrade>)> {
    let exchange_str: String = dict
        .get_item("exchange")?
        .ok_or_else(|| PyValueError::new_err("event missing 'exchange'"))?
        .extract()?;
    let exchange = parse_exchange_id(&exchange_str)?;

    let instrument = dict
        .get_item("instrument")?
        .ok_or_else(|| PyValueError::new_err("event missing 'instrument'"))?
        .extract::<usize>()?;

    let time_exchange = dict
        .get_item("time_exchange")?
        .ok_or_else(|| PyValueError::new_err("event missing 'time_exchange'"))?
        .extract::<DateTime<Utc>>()?;
    let time_received = dict
        .get_item("time_received")?
        .ok_or_else(|| PyValueError::new_err("event missing 'time_received'"))?
        .extract::<DateTime<Utc>>()?;

    let trade_value = dict
        .get_item("trade")?
        .ok_or_else(|| PyValueError::new_err("event missing 'trade'"))?;
    let trade_dict = trade_value.downcast::<PyDict>()?;

    let trade = PublicTrade {
        id: trade_dict
            .get_item("id")?
            .ok_or_else(|| PyValueError::new_err("trade missing 'id'"))?
            .extract()?,
        price: trade_dict
            .get_item("price")?
            .ok_or_else(|| PyValueError::new_err("trade missing 'price'"))?
            .extract()?,
        amount: trade_dict
            .get_item("amount")?
            .ok_or_else(|| PyValueError::new_err("trade missing 'amount'"))?
            .extract()?,
        side: parse_side(
            trade_dict
                .get_item("side")?
                .ok_or_else(|| PyValueError::new_err("trade missing 'side'"))?
                .extract::<&str>()?,
        )?,
    };

    Ok((
        exchange,
        Event::Item(Ok(MarketEvent {
            time_exchange,
            time_received,
            exchange,
            instrument: InstrumentIndex(instrument),
            kind: trade,
        })),
    ))
}

#[cfg(feature = "python-tests")]
#[pyfunction]
pub fn _testing_dynamic_trades(
    py: Python<'_>,
    events: Vec<PyObject>,
) -> PyResult<PyDynamicStreams> {
    let mut grouped: BTreeMap<ExchangeId, Vec<MarketStreamResult<InstrumentIndex, PublicTrade>>> =
        BTreeMap::new();

    for obj in events {
        let dict = obj.downcast_bound::<PyDict>(py)?;
        let event_type: String = dict
            .get_item("type")?
            .ok_or_else(|| PyValueError::new_err("event missing 'type'"))?
            .extract()?;

        match event_type.as_str() {
            "item" => {
                let (exchange, event) = parse_trade_event(py, &dict)?;
                grouped.entry(exchange).or_default().push(event);
            }
            "reconnecting" => {
                let exchange_str: String = dict
                    .get_item("exchange")?
                    .ok_or_else(|| PyValueError::new_err("reconnect missing 'exchange'"))?
                    .extract()?;
                let exchange = parse_exchange_id(&exchange_str)?;
                grouped
                    .entry(exchange)
                    .or_default()
                    .push(Event::Reconnecting(exchange));
            }
            "error" => {
                let exchange_str: String = dict
                    .get_item("exchange")?
                    .ok_or_else(|| PyValueError::new_err("error missing 'exchange'"))?
                    .extract()?;
                let exchange = parse_exchange_id(&exchange_str)?;
                let message: String = dict
                    .get_item("message")?
                    .ok_or_else(|| PyValueError::new_err("error missing 'message'"))?
                    .extract()?;
                grouped.entry(exchange).or_default().push(Event::Item(Err(
                    barter_data::error::DataError::Socket(message),
                )));
            }
            other => {
                return Err(PyValueError::new_err(format!(
                    "unsupported event type '{}'",
                    other
                )));
            }
        }
    }

    let mut trade_map: VecMap<ExchangeId, UnboundedReceiverStream<_>> = VecMap::default();

    for (exchange, events) in grouped {
        let (tx, rx) = mpsc::unbounded_channel();
        for event in events {
            tx.send(event).expect("send event");
        }
        drop(tx);
        trade_map.insert(exchange, UnboundedReceiverStream::new(rx));
    }

    let streams = DynamicStreams {
        trades: trade_map,
        l1s: VecMap::default(),
        l2s: VecMap::default(),
        liquidations: VecMap::default(),
    };

    let runtime = Arc::new(
        RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?,
    );

    Ok(PyDynamicStreams::from_parts(runtime, streams))
}

#[cfg(test)]
mod tests {
    use super::*;
    use barter_data::{
        event::MarketEvent,
        streams::{
            builder::dynamic::DynamicStreams, consumer::MarketStreamResult, reconnect::Event,
        },
        subscription::trade::PublicTrade,
    };
    use barter_instrument::{Side, exchange::ExchangeId, instrument::InstrumentIndex};
    use chrono::{TimeZone, Utc};
    use pyo3::{
        Py, PyResult as PyO3Result,
        types::{PyDict, PyList},
    };
    use std::{path::Path, sync::Arc};
    use tokio::{
        runtime::{Builder as RuntimeBuilder, Runtime},
        sync::mpsc::unbounded_channel,
    };
    use vecmap::VecMap;

    fn new_runtime() -> Runtime {
        RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
    }

    fn sample_trade_event() -> MarketStreamResult<InstrumentIndex, PublicTrade> {
        let trade = PublicTrade {
            id: "trade-1".to_string(),
            price: 100.5,
            amount: 0.25,
            side: Side::Buy,
        };

        Event::Item(Ok(MarketEvent {
            time_exchange: Utc.timestamp_opt(1_697_000_000, 0).single().unwrap(),
            time_received: Utc.timestamp_opt(1_697_000_001, 0).single().unwrap(),
            exchange: ExchangeId::BinanceSpot,
            instrument: InstrumentIndex(42),
            kind: trade,
        }))
    }

    fn sample_reconnect_event() -> MarketStreamResult<InstrumentIndex, PublicTrade> {
        Event::Reconnecting(ExchangeId::BinanceSpot)
    }

    fn sample_trade_error() -> MarketStreamResult<InstrumentIndex, PublicTrade> {
        Event::Item(Err(barter_data::error::DataError::SubscriptionsEmpty))
    }

    fn build_dynamic_streams(
        trades: Vec<MarketStreamResult<InstrumentIndex, PublicTrade>>,
    ) -> DynamicStreams<InstrumentIndex> {
        let (tx, rx) = unbounded_channel();
        for event in trades {
            tx.send(event).expect("send event");
        }
        drop(tx);

        let mut trade_map: VecMap<_, _> = VecMap::default();
        trade_map.insert(ExchangeId::BinanceSpot, UnboundedReceiverStream::new(rx));

        DynamicStreams {
            trades: trade_map,
            l1s: VecMap::default(),
            l2s: VecMap::default(),
            liquidations: VecMap::default(),
        }
    }

    fn with_streams<F>(
        trades: Vec<MarketStreamResult<InstrumentIndex, PublicTrade>>,
        func: F,
    ) -> PyO3Result<()>
    where
        F: FnOnce(Python<'_>, &PyDynamicStreams) -> PyO3Result<()>,
    {
        Python::with_gil(|py| {
            let runtime = Arc::new(new_runtime());
            let streams = build_dynamic_streams(trades);
            let py_streams = Py::new(py, PyDynamicStreams::from_parts(runtime, streams))?;
            let module_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("python");
            let sys = PyModule::import_bound(py, "sys")?;
            let sys_path_value = sys.getattr("path")?;
            let sys_path = sys_path_value.downcast::<PyList>()?;
            sys_path.insert(0, module_path.to_str().unwrap())?;
            let borrowed = py_streams.borrow(py);
            func(py, &borrowed)
        })
    }

    #[test]
    fn trade_stream_yields_market_event() {
        let result = with_streams(vec![sample_trade_event()], |py, streams| {
            let exchange = PyExchangeId::from_inner(ExchangeId::BinanceSpot);
            let stream = streams
                .select_trades(&exchange)?
                .expect("trade stream available");

            let wrapper = stream.recv(py, None)?.expect("event");
            let event = wrapper.getattr(py, "event")?;
            let exchange_value: String = event.getattr(py, "exchange")?.extract(py)?;
            assert_eq!(exchange_value, ExchangeId::BinanceSpot.as_str());

            let instrument: usize = event.getattr(py, "instrument")?.extract(py)?;
            assert_eq!(instrument, 42);

            let kind = event.getattr(py, "kind")?;
            let name: String = kind.getattr(py, "kind")?.extract(py)?;
            assert_eq!(name, "trade");

            Ok(())
        });

        result.unwrap();
    }

    #[test]
    fn trade_stream_handles_reconnects() {
        let result = with_streams(vec![sample_reconnect_event()], |py, streams| {
            let exchange = PyExchangeId::from_inner(ExchangeId::BinanceSpot);
            let stream = streams
                .select_trades(&exchange)?
                .expect("trade stream available");

            let reconnect = stream.recv(py, None)?.expect("reconnect event");
            let kind: String = reconnect.getattr(py, "kind")?.extract(py)?;
            assert_eq!(kind, "reconnecting");
            let exchange: String = reconnect.getattr(py, "exchange")?.extract(py)?;
            assert_eq!(exchange, ExchangeId::BinanceSpot.as_str());
            Ok(())
        });

        result.unwrap();
    }

    #[test]
    fn trade_stream_propagates_errors() {
        let result = with_streams(vec![sample_trade_error()], |py, streams| {
            let exchange = PyExchangeId::from_inner(ExchangeId::BinanceSpot);
            let stream = streams
                .select_trades(&exchange)?
                .expect("trade stream available");

            let err = stream.recv(py, None).unwrap_err();
            assert!(err.to_string().contains("subscription"));
            Ok(())
        });

        result.unwrap();
    }
}
