#![allow(unused_imports)]

use crate::command::parse_decimal;
use barter_data::{
    streams::builder::dynamic::DynamicStreams,
    subscription::{SubKind, Subscription},
};
use barter_instrument::{
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
use pyo3::{Bound, exceptions::PyValueError, prelude::*, types::PyDict};

/// Wrapper around [`ExchangeId`] for Python exposure.
#[pyclass(module = "barter_python", name = "ExchangeId", eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExchangeId {
    inner: ExchangeId,
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
    #[allow(dead_code)]
    inner: Option<DynamicStreams<InstrumentIndex>>,
}

#[pymethods]
impl PyDynamicStreams {
    /// Create a new empty DynamicStreams instance.
    #[new]
    fn new() -> Self {
        Self { inner: None }
    }

    /// Select trades stream for a specific exchange.
    fn select_trades(&mut self, _exchange: &PyExchangeId) -> PyResult<Option<PyObject>> {
        // Placeholder - streams need to be initialized first
        Ok(None)
    }

    /// Select all trades streams.
    fn select_all_trades(&mut self) -> PyResult<PyObject> {
        // Placeholder - streams need to be initialized first
        Ok(pyo3::Python::with_gil(|py| py.None()))
    }
}

/// Initialize market data streams asynchronously.
/// Note: This is a placeholder - actual async implementation needed
#[pyfunction]
#[pyo3(signature = (_subscriptions))]
pub fn init_dynamic_streams(
    _py: Python<'_>,
    _subscriptions: Vec<Vec<PySubscription>>,
) -> PyResult<PyObject> {
    // Placeholder - need proper async handling
    Ok(pyo3::Python::with_gil(|py| py.None()))
}
