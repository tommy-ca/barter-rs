#![allow(unused_imports)]

use barter_data::{
    streams::builder::dynamic::DynamicStreams,
    subscription::{SubKind, Subscription},
};
use barter_instrument::{
    exchange::ExchangeId,
    instrument::{
        InstrumentIndex,
        market_data::{MarketDataInstrument, kind::MarketDataInstrumentKind},
    },
};
use barter_integration::subscription::SubscriptionId;
use pyo3::prelude::*;

/// Wrapper around [`ExchangeId`] for Python exposure.
#[pyclass(module = "barter_python", name = "ExchangeId", eq)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyExchangeId {
    inner: ExchangeId,
}

#[pymethods]
impl PyExchangeId {
    /// Binance Spot exchange.
    #[classattr]
    const BINANCE_SPOT: Self = Self {
        inner: ExchangeId::BinanceSpot,
    };

    /// Binance Futures USD exchange.
    #[classattr]
    const BINANCE_FUTURES_USD: Self = Self {
        inner: ExchangeId::BinanceFuturesUsd,
    };

    /// Bitfinex exchange.
    #[classattr]
    const BITFINEX: Self = Self {
        inner: ExchangeId::Bitfinex,
    };

    /// BitMEX exchange.
    #[classattr]
    const BITMEX: Self = Self {
        inner: ExchangeId::Bitmex,
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

    /// Coinbase exchange.
    #[classattr]
    const COINBASE: Self = Self {
        inner: ExchangeId::Coinbase,
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

    /// Gate.io Futures BTC exchange.
    #[classattr]
    const GATEIO_FUTURES_BTC: Self = Self {
        inner: ExchangeId::GateioFuturesBtc,
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

    /// Kraken exchange.
    #[classattr]
    const KRAKEN: Self = Self {
        inner: ExchangeId::Kraken,
    };

    /// OKX exchange.
    #[classattr]
    const OKX: Self = Self {
        inner: ExchangeId::Okx,
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
        instrument_kind: Option<&str>,
    ) -> PyResult<Self> {
        let instrument_kind = match instrument_kind {
            Some("spot") | None => MarketDataInstrumentKind::Spot,
            Some("perpetual") => MarketDataInstrumentKind::Perpetual,
            Some(kind) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid instrument_kind '{}'. Currently only 'spot' and 'perpetual' are supported",
                    kind
                )));
            }
        };

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
