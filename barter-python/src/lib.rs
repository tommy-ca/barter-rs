#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, rust_2024_compatibility)]
#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::useless_conversion,
    clippy::needless_borrow
)]

//! Python bindings for the Barter trading engine.

mod account;
mod analytics;
mod backtest;
mod books;
mod classes;
mod collection;
mod command;
mod common;
mod config;
mod data;
mod error;
mod execution;
mod instrument;
mod integration;
mod logging;
mod metric;
mod risk;
mod strategy;
mod summary;
mod system;

use account::{PyAccountSnapshot, PyInstrumentAccountSnapshot};
use analytics::{
    calculate_calmar_ratio, calculate_max_drawdown, calculate_mean_drawdown,
    calculate_profit_factor, calculate_rate_of_return, calculate_sharpe_ratio,
    calculate_sortino_ratio, calculate_win_rate, generate_drawdown_series, welford_calculate_mean,
    welford_calculate_population_variance, welford_calculate_recurrence_relation_m,
    welford_calculate_sample_variance,
};
use backtest::{PyBacktestArgsConstant, PyBacktestArgsDynamic, PyMarketDataInMemory};
use books::{PyLevel, PyOrderBook, calculate_mid_price, calculate_volume_weighted_mid_price};


use classes::core::{PySequence, PyTimedF64, shutdown_event, timed_f64};
use classes::engine::PyEngineEvent;
use barter_data::{
    books::Level,
};

use barter_instrument::{
    exchange::ExchangeId,
};

use collection::{PyNoneOneOrMany, PyOneOrMany};
use command::{
    PyInstrumentFilter, PyOrderKey, PyOrderRequestCancel, PyOrderRequestOpen, PyOrderSnapshot,
    parse_decimal,
};
use config::{PyExecutionConfig, PyMockExecutionConfig, PySystemConfig};
#[cfg(feature = "python-tests")]
use data::_testing_dynamic_trades;
use data::{
    PyAsyncMarketStream, PyDynamicStreams, PyExchangeId, PyMarketStream, PySubKind, PySubscription, PySubscriptionId,
    exchange_supports_instrument_kind, init_dynamic_streams,
};
use error::{PySocketErrorInfo, SocketError as PySocketErrorExc};
use execution::{
    PyActiveOrderState, PyAssetFees, PyCancelInFlightState, PyCancelledState, PyClientOrderId,
    PyExecutionAssetBalance, PyExecutionBalance, PyExecutionInstrumentMap, PyInactiveOrderState,
    PyMockExecutionClient, PyOpenState, PyOrderError, PyOrderEvent, PyOrderId, PyOrderKind, PyOrderState, PyStrategyId,
    PyTimeInForce, PyTrade, PyTradeId, asset_balance_new, balance_new,
};
use instrument::{
    PyAsset, PyAssetIndex, PyAssetNameExchange, PyAssetNameInternal, PyExchangeIndex,
    PyIndexedInstruments, PyInstrumentIndex, PyInstrumentNameExchange, PyInstrumentNameInternal,
    PyOrderQuantityUnits, PyInstrumentSpec, PyInstrumentSpecNotional, PyInstrumentSpecPrice,
    PyInstrumentSpecQuantity, PyQuoteAsset, PySide,
};
use integration::{PySnapUpdates, PySnapshot};
use logging::{init_json_logging_py, init_tracing};
use metric::{PyField, PyMetric, PyTag, PyValue};
use pyo3::{Bound, exceptions::PyValueError, prelude::*, types::PyModule};
use risk::{
    PyDefaultRiskManager, PyRiskApproved, PyRiskRefused, calculate_abs_percent_difference,
    calculate_delta, calculate_quote_notional,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;
use strategy::build_ioc_market_order_to_close_position;
use summary::{
    PyAssetTearSheet, PyBacktestSummary, PyDrawdown, PyInstrumentTearSheet,
    PyMeanDrawdown, PyMetricWithInterval, PyMultiBacktestSummary, PyTradingSummary,
};
use system::{
    PyAuditContext, PyAuditEvent, PyAuditTick, PyAuditUpdates, PySystemHandle,
    run_historic_backtest, start_system,
};












static EXCHANGE_ID_CACHE: Mutex<Option<HashMap<String, ExchangeId>>> = Mutex::new(None);

fn parse_exchange_id(value: &str) -> PyResult<ExchangeId> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PyValueError::new_err("exchange must not be empty"));
    }

    let normalized = trimmed.to_ascii_lowercase();

    // Check cache first
    if let Some(cache) = EXCHANGE_ID_CACHE.lock().unwrap().as_ref()
        && let Some(&exchange_id) = cache.get(&normalized) {
        return Ok(exchange_id);
    }

    // Parse and cache
    let exchange_id = serde_json::from_value(JsonValue::String(normalized.clone())).map_err(|_| {
        PyValueError::new_err(format!(
            "unknown exchange identifier: {trimmed}. expected snake_case exchange ids such as 'binance_spot'"
        ))
    })?;

    // Initialize cache if needed and insert
    let mut cache = EXCHANGE_ID_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
    cache.as_mut().unwrap().insert(normalized, exchange_id);

    Ok(exchange_id)
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

    let price = parse_decimal(price, "price")?;
    let amount = parse_decimal(amount, "amount")?;

    Ok(Level::new(price, amount))
}



/// Python module definition entry point.
#[pymodule]
pub fn barter_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Create execution submodule
    let execution = PyModule::new_bound(py, "execution")?;
    execution.add_class::<PyClientOrderId>()?;
    execution.add_class::<PyOrderId>()?;
    execution.add_class::<PyStrategyId>()?;
    execution.add_class::<PyTradeId>()?;
    execution.add_class::<PyTrade>()?;
    execution.add_class::<PyAssetFees>()?;
    execution.add_class::<PyExecutionBalance>()?;
    execution.add_class::<PyExecutionAssetBalance>()?;
    execution.add_class::<PyExecutionInstrumentMap>()?;
    execution.add_class::<PyMockExecutionClient>()?;
    execution.add_class::<PyOrderKey>()?;
    execution.add_class::<PyOrderKind>()?;
    execution.add_class::<PyOrderRequestOpen>()?;
    execution.add_class::<PyOrderRequestCancel>()?;
    execution.add_class::<PyOrderSnapshot>()?;
    execution.add_class::<PyOrderEvent>()?;
    execution.add_class::<PyOrderState>()?;
    execution.add_class::<PyActiveOrderState>()?;
    execution.add_class::<PyInactiveOrderState>()?;
    execution.add_class::<PyOpenState>()?;
    execution.add_class::<PyCancelInFlightState>()?;
    execution.add_class::<PyCancelledState>()?;
    execution.add_class::<PyOrderError>()?;
    execution.add_class::<PyInstrumentAccountSnapshot>()?;
    execution.add_class::<PyAccountSnapshot>()?;
    execution.add_class::<PyTimeInForce>()?;
    m.add_submodule(&execution)?;

    m.add_class::<PySystemConfig>()?;
    m.add_class::<PyMockExecutionConfig>()?;
    m.add_class::<PyExecutionConfig>()?;
    m.add_class::<PyEngineEvent>()?;
    m.add_class::<PyTimedF64>()?;
    m.add_class::<PySocketErrorInfo>()?;
    m.add_class::<PySequence>()?;
    m.add_class::<PySystemHandle>()?;
    m.add_class::<PyInstrumentFilter>()?;
    m.add_class::<PyTradingSummary>()?;
    m.add_class::<PyInstrumentTearSheet>()?;
    m.add_class::<PyAssetTearSheet>()?;
    m.add_class::<PyMetricWithInterval>()?;
    m.add_class::<PyDrawdown>()?;
    m.add_class::<PyMeanDrawdown>()?;

     m.add_class::<PyExchangeId>()?;
     m.add_class::<PySubKind>()?;
     m.add_class::<PySubscription>()?;
     m.add_class::<PySubscriptionId>()?;
     m.add_class::<PyDynamicStreams>()?;
     m.add_class::<PyMarketStream>()?;
     m.add_class::<PyAsyncMarketStream>()?;
     m.add_class::<PyAssetNameInternal>()?;
     m.add_class::<PyAssetNameExchange>()?;
     m.add_class::<PyInstrumentNameInternal>()?;
     m.add_class::<PyInstrumentNameExchange>()?;
     m.add_class::<PyAsset>()?;
     m.add_class::<PyAssetIndex>()?;
     m.add_class::<PyQuoteAsset>()?;
     m.add_class::<PyExchangeIndex>()?;
     m.add_class::<PyInstrumentIndex>()?;
     m.add_class::<PyIndexedInstruments>()?;
     m.add_class::<PyOrderQuantityUnits>()?;
     m.add_class::<PyInstrumentSpecPrice>()?;
     m.add_class::<PyInstrumentSpecQuantity>()?;
     m.add_class::<PyInstrumentSpecNotional>()?;
     m.add_class::<PyInstrumentSpec>()?;
     m.add_class::<PySide>()?;
     m.add_class::<PyRiskApproved>()?;
     m.add_class::<PyRiskRefused>()?;
     m.add_class::<PyDefaultRiskManager>()?;
     m.add_class::<PyMetric>()?;
     m.add_class::<PyTag>()?;
     m.add_class::<PyBacktestArgsConstant>()?;
     m.add_class::<PyBacktestArgsDynamic>()?;
     m.add_class::<PyMarketDataInMemory>()?;
     m.add_class::<PyField>()?;
     m.add_class::<PyValue>()?;
     m.add_class::<PyBacktestSummary>()?;
     m.add_class::<PyMultiBacktestSummary>()?;
     m.add_class::<PyLevel>()?;
     m.add_class::<PyOrderBook>()?;
     m.add_class::<PySnapshot>()?;
     m.add_class::<PySnapUpdates>()?;
     m.add_class::<PyAuditContext>()?;
     m.add_class::<PyAuditEvent>()?;
     m.add_class::<PyAuditTick>()?;
     m.add_class::<PyNoneOneOrMany>()?;
     m.add_class::<PyOneOrMany>()?;
     m.add_class::<PyAuditUpdates>()?;
     m.add_class::<PyTradeId>()?;
    m.add_function(wrap_pyfunction!(init_tracing, m)?)?;
    m.add_function(wrap_pyfunction!(init_json_logging_py, m)?)?;
    m.add_function(wrap_pyfunction!(shutdown_event, m)?)?;
    m.add_function(wrap_pyfunction!(timed_f64, m)?)?;
    m.add_function(wrap_pyfunction!(run_historic_backtest, m)?)?;
    m.add_function(wrap_pyfunction!(backtest::backtest, m)?)?;
    m.add_function(wrap_pyfunction!(backtest::run_backtests, m)?)?;
    m.add_function(wrap_pyfunction!(start_system, m)?)?;
    m.add_function(wrap_pyfunction!(init_dynamic_streams, m)?)?;
    m.add_function(wrap_pyfunction!(exchange_supports_instrument_kind, m)?)?;
    #[cfg(feature = "python-tests")]
    m.add_function(wrap_pyfunction!(_testing_dynamic_trades, m)?)?;
    #[cfg(feature = "python-tests")]
    m.add_function(wrap_pyfunction!(error::_testing_raise_socket_error, m)?)?;
    m.add_function(wrap_pyfunction!(balance_new, m)?)?;
    m.add_function(wrap_pyfunction!(asset_balance_new, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_sharpe_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_sortino_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_calmar_ratio, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_profit_factor, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_win_rate, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_rate_of_return, m)?)?;
    m.add_function(wrap_pyfunction!(generate_drawdown_series, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_max_drawdown, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_mean_drawdown, m)?)?;
    m.add_function(wrap_pyfunction!(welford_calculate_mean, m)?)?;
    m.add_function(wrap_pyfunction!(
        welford_calculate_recurrence_relation_m,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(welford_calculate_sample_variance, m)?)?;
    m.add_function(wrap_pyfunction!(welford_calculate_population_variance, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_quote_notional, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_abs_percent_difference, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_delta, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_mid_price, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_volume_weighted_mid_price, m)?)?;
    m.add_function(wrap_pyfunction!(
        build_ioc_market_order_to_close_position,
        m
    )?)?;

    // Expose module level constants.
    let shutdown = PyEngineEvent::shutdown();
    m.add("SHUTDOWN_EVENT", shutdown.into_py(py))?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    let socket_error_type = py.get_type_bound::<PySocketErrorExc>();
    m.add("SocketError", socket_error_type)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use barter::{
        EngineEvent,
        engine::state::trading::TradingState,
        Sequence,
        Timed,
        execution::AccountStreamEvent,
    };
    use barter_data::{
        event::DataKind,
        streams::consumer::MarketStreamEvent,
        subscription::book::OrderBookEvent,
    };
    use barter_execution::{
        order::{
            OrderKind, TimeInForce,
            id::{ClientOrderId, OrderId, StrategyId},
            state::{ActiveOrderState, OrderState},
        },
        trade::TradeId,
        AccountEventKind,
    };
    use barter_instrument::{
        Side,
        exchange::{ExchangeId, ExchangeIndex},
        instrument::InstrumentIndex,
    };
    use barter_integration::{error::SocketError as IntegrationSocketError, Terminal};
    use chrono::{TimeDelta, TimeZone, Utc};
    use pyo3::{
        Python,
        types::{PyDict, PyString},
    };
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
        let timed = PyTimedF64::new(42.5, time);

        assert_eq!(timed.value(), 42.5);
        assert_eq!(timed.time(), time);
    }

    #[test]
    fn sequence_wrapper_advances_like_rust() {
        let mut sequence = PySequence::from_inner(Sequence(10));
        assert_eq!(sequence.value(), 10);

        let previous = sequence.fetch_add();
        assert_eq!(previous.value(), 10);
        assert_eq!(sequence.value(), 11);

        let next_value = sequence.next_value();
        assert_eq!(next_value, 12);
        assert_eq!(sequence.value(), 12);
    }

    #[test]
    fn engine_event_market_trade_constructor() {
        let time_exchange = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        let time_received = time_exchange + TimeDelta::seconds(1);

        let event = PyEngineEvent::market_trade(
            "binance_spot",
            2,
            "trade-1",
            101.25,
            0.5,
            "buy",
            Some(time_exchange),
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
            PyEngineEvent::market_trade("mock", 0, "trade-123", 1.25, 3.5, "sell", Some(time_exchange), None)
                .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::Mock);
                assert_eq!(item.instrument, InstrumentIndex(0));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::Trade(trade) => {
                        assert_eq!(trade.id, "trade-123");
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
            Some(100.5),
            Some(2.0),
            Some(101.0),
            Some(1.5),
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
                        assert_eq!(book.last_update_time, time_exchange);

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

        let event = PyEngineEvent::market_candle(
            "kraken",
            4,
            100.0,
            110.0,
            95.0,
            105.0,
            250.5,
            Some(time_exchange),
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
                        assert_eq!(candle.close_time, time_exchange);
                        assert_eq!(candle.open, 100.0);
                        assert_eq!(candle.high, 110.0);
                        assert_eq!(candle.low, 95.0);
                        assert_eq!(candle.close, 105.0);
                        assert_eq!(candle.volume, 250.5);
                        assert_eq!(candle.trade_count, 0);
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
            Some(time_exchange),
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
    fn engine_event_market_order_book_snapshot_constructor() {
        let time_engine = Utc.with_ymd_and_hms(2025, 4, 5, 6, 7, 8).unwrap();
        let time_exchange = time_engine + TimeDelta::seconds(1);

        let event = PyEngineEvent::market_order_book_snapshot(
            "binance_spot",
            3,
            12345,
            Some(time_engine),
            vec![(100.5, 2.0), (100.0, 1.5)],
            vec![(101.0, 1.0), (101.5, 0.5)],
            Some(time_exchange),
            None,
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::BinanceSpot);
                assert_eq!(item.instrument, InstrumentIndex(3));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::OrderBook(OrderBookEvent::Snapshot(order_book)) => {
                        assert_eq!(order_book.sequence(), 12345);
                        assert_eq!(order_book.time_engine(), Some(time_engine));
                        assert_eq!(order_book.bids().levels().len(), 2);
                        assert_eq!(order_book.asks().levels().len(), 2);
                        // Bids should be sorted descending
                        assert!(
                            order_book.bids().levels()[0].price
                                > order_book.bids().levels()[1].price
                        );
                        // Asks should be sorted ascending
                        assert!(
                            order_book.asks().levels()[0].price
                                < order_book.asks().levels()[1].price
                        );
                    }
                    other => panic!("unexpected market data kind: {other:?}"),
                }
            }
            other => panic!("unexpected event variant: {other:?}"),
        }
    }

    #[test]
    fn engine_event_market_order_book_update_constructor() {
        let time_engine = Utc.with_ymd_and_hms(2025, 4, 5, 6, 7, 8).unwrap();
        let time_exchange = time_engine + TimeDelta::seconds(1);

        let event = PyEngineEvent::market_order_book_update(
            "binance_spot",
            3,
            12346,
            Some(time_engine),
            vec![(100.5, 2.0), (100.0, 1.5)],
            vec![(101.0, 1.0), (101.5, 0.5)],
            Some(time_exchange),
            None,
        )
        .unwrap();

        match event.inner {
            EngineEvent::Market(MarketStreamEvent::Item(item)) => {
                assert_eq!(item.exchange, ExchangeId::BinanceSpot);
                assert_eq!(item.instrument, InstrumentIndex(3));
                assert_eq!(item.time_exchange, time_exchange);
                assert_eq!(item.time_received, time_exchange);

                match item.kind {
                    DataKind::OrderBook(OrderBookEvent::Update(order_book)) => {
                        assert_eq!(order_book.sequence(), 12346);
                        assert_eq!(order_book.time_engine(), Some(time_engine));
                        assert_eq!(order_book.bids().levels().len(), 2);
                        assert_eq!(order_book.asks().levels().len(), 2);
                        // Bids should be sorted descending
                        assert!(
                            order_book.bids().levels()[0].price
                                > order_book.bids().levels()[1].price
                        );
                        // Asks should be sorted ascending
                        assert!(
                            order_book.asks().levels()[0].price
                                < order_book.asks().levels()[1].price
                        );
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

    // #[test]
    // fn engine_event_account_trade_constructor() {
    //     let time_exchange = Utc.with_ymd_and_hms(2025, 8, 9, 10, 11, 12).unwrap();

    //     let event = PyEngineEvent::account_trade(
    //         3,
    //         4,
    //         "strategy-123",
    //         "order-456",
    //         "trade-789",
    //         "buy",
    //         125.25,
    //         0.75,
    //         time_exchange,
    //         Some(0.0015),
    //     )
    //     .unwrap();

    //     match event.inner {
    //         EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
    //             assert_eq!(account_event.exchange, ExchangeIndex(3));

    //             match account_event.kind {
    //                 AccountEventKind::Trade(trade) => {
    //                     assert_eq!(trade.instrument, InstrumentIndex(4));
    //                     assert_eq!(trade.strategy, StrategyId::new("strategy-123"));
    //                     assert_eq!(trade.order_id, OrderId::new("order-456"));
    //                     assert_eq!(trade.id, TradeId::new("trade-789"));
    //                     assert_eq!(trade.side, Side::Buy);
    //                     assert_eq!(trade.price.to_f64().unwrap(), 125.25);
    //                     assert_eq!(trade.quantity.to_f64().unwrap(), 0.75);
    //                     assert_eq!(trade.time_exchange, time_exchange);
    //                     assert_eq!(trade.fees.fees.to_f64().unwrap(), 0.0015);
    //                 }
    //                 other => panic!("unexpected account event kind: {other:?}"),
    //             }
    //         }
    //         other => panic!("unexpected event variant: {other:?}"),
    //     }
    // }

    // #[test]
    // fn engine_event_account_order_snapshot_open() {
    //     let key = PyOrderKey::from_parts(
    //         ExchangeIndex(1),
    //         InstrumentIndex(2),
    //         StrategyId::new("strategy-alpha"),
    //         ClientOrderId::new("cid-1"),
    //     );
    //     let open_request = Python::with_gil(|py| {
    //         let kind = PyString::new_bound(py, "limit").into_any();
    //         let tif = PyString::new_bound(py, "good_until_cancelled").into_any();
    //         PyOrderRequestOpen::new(
    //             &key,
    //             "buy",
    //             105.25,
    //             0.75,
    //             Some(&kind),
    //             Some(&tif),
    //             Some(true),
    //         )
    //     })
    //     .unwrap();
    //     let time_exchange = Utc.with_ymd_and_hms(2025, 9, 10, 11, 12, 13).unwrap();

    //     let snapshot = PyOrderSnapshot::from_open_request(
    //         &open_request,
    //         Some("order-789"),
    //         Some(time_exchange),
    //         0.25,
    //     )
    //     .unwrap();

    //     let event = PyEngineEvent::account_order_snapshot(1, &snapshot).unwrap();

    //     match event.inner {
    //         EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
    //             assert_eq!(account_event.exchange, ExchangeIndex(1));

    //             match account_event.kind {
    //                 AccountEventKind::OrderSnapshot(snapshot) => {
    //                     let order = snapshot.value();
    //                     assert_eq!(order.key.exchange, ExchangeIndex(1));
    //                     assert_eq!(order.key.instrument, InstrumentIndex(2));
    //                     assert_eq!(order.key.strategy, StrategyId::new("strategy-alpha"));
    //                     assert_eq!(order.side, Side::Buy);
    //                     assert_eq!(order.price.to_f64().unwrap(), 105.25);
    //                     assert_eq!(order.quantity.to_f64().unwrap(), 0.75);
    //                     assert_eq!(order.kind, OrderKind::Limit);
    //                     assert_eq!(
    //                         order.time_in_force,
    //                         TimeInForce::GoodUntilCancelled { post_only: true }
    //                     );

    //                     match &order.state {
    //                         OrderState::Active(ActiveOrderState::Open(open)) => {
    //                             assert_eq!(open.id, OrderId::new("order-789"));
    //                             assert_eq!(open.time_exchange, time_exchange);
    //                             assert_eq!(open.filled_quantity.to_f64().unwrap(), 0.25);
    //                         }
    //                         other => panic!("unexpected order state: {other:?}"),
    //                     }
    //                 }
    //                 other => panic!("unexpected account event kind: {other:?}"),
    //             }
    //         }
    //         other => panic!("unexpected event variant: {other:?}"),
    //     }
    // }

    // #[test]
    // fn engine_event_account_order_snapshot_open_inflight() {
    //     let key = PyOrderKey::from_parts(
    //         ExchangeIndex(3),
    //         InstrumentIndex(4),
    //         StrategyId::new("strategy-beta"),
    //         ClientOrderId::new("cid-2"),
    //     );
    //     let open_request = Python::with_gil(|py| {
    //         let kind = PyString::new_bound(py, "limit").into_any();
    //         PyOrderRequestOpen::new(&key, "sell", 250.0, 1.5, Some(&kind), None, None)
    //     })
    //     .unwrap();

    //     let snapshot = PyOrderSnapshot::from_open_request(&open_request, None, None, 0.0).unwrap();

    //     let event = PyEngineEvent::account_order_snapshot(3, &snapshot).unwrap();

    //     match event.inner {
    //         EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
    //             assert_eq!(account_event.exchange, ExchangeIndex(3));

    //             match account_event.kind {
    //                 AccountEventKind::OrderSnapshot(snapshot) => {
    //                     let order = snapshot.value();
    //                     assert_eq!(order.key.exchange, ExchangeIndex(3));
    //                     assert_eq!(order.key.instrument, InstrumentIndex(4));
    //                     assert_eq!(order.side, Side::Sell);

    //                     match &order.state {
    //                         OrderState::Active(ActiveOrderState::OpenInFlight(_)) => {}
    //                         other => panic!("unexpected order state: {other:?}"),
    //                     }
    //                 }
    //                 other => panic!("unexpected account event kind: {other:?}"),
    //             }
    //         }
    //         other => panic!("unexpected event variant: {other:?}"),
    //     }
    // }

    // #[test]
    // fn engine_event_account_order_cancelled_success() {
    //     let key = PyOrderKey::from_parts(
    //         ExchangeIndex(2),
    //         InstrumentIndex(5),
    //         StrategyId::new("strategy-gamma"),
    //         ClientOrderId::new("cid-3"),
    //     );
    //     let cancel_request = PyOrderRequestCancel::new(&key, Some("order-456"))
    //         .expect("cancel request should build");
    //     let time_exchange = Utc.with_ymd_and_hms(2025, 12, 1, 2, 3, 4).unwrap();

    //     let event =
    //         PyEngineEvent::account_order_cancelled(2, &cancel_request, "order-456", time_exchange)
    //             .unwrap();

    //     match event.inner {
    //         EngineEvent::Account(AccountStreamEvent::Item(account_event)) => {
    //             assert_eq!(account_event.exchange, ExchangeIndex(2));

    //             match account_event.kind {
    //                 AccountEventKind::OrderCancelled(response) => {
    //                     assert_eq!(response.key.exchange, ExchangeIndex(2));
    //                     assert_eq!(response.key.instrument, InstrumentIndex(5));

    //                     match response.state {
    //                         Ok(cancelled) => {
    //                             assert_eq!(cancelled.id, OrderId::new("order-456"));
    //                             assert_eq!(cancelled.time_exchange, time_exchange);
    //                             }
    //                         Err(err) => panic!("unexpected cancellation error: {err:?}"),
    //                     }
    //                 }
    //                 other => panic!("unexpected account event kind: {other:?}"),
    //             }
    //         }
    //         other => panic!("unexpected event variant: {other:?}"),
    //     }
    // }

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

    #[test]
    fn subscription_id_constructor_and_accessors() {
        let id = PySubscriptionId::new_test("test-subscription");
        assert_eq!(id.inner.0.as_str(), "test-subscription");
    }

    #[test]
    fn subscription_id_equality_and_hash() {
        let id1 = PySubscriptionId::new_test("same");
        let id2 = PySubscriptionId::new_test("same");
        let id3 = PySubscriptionId::new_test("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn metric_construction_and_accessors() {
        let tag = PyTag::new("key".to_string(), "value".to_string());
        let field = PyField::new("field_key".to_string(), PyValue::float(42.5));
        let metric = PyMetric::new(
            "test_metric".to_string(),
            1234567890,
            vec![tag],
            vec![field],
        )
        .unwrap();

        assert_eq!(metric.name(), "test_metric");
        assert_eq!(metric.time(), 1234567890);
        assert_eq!(metric.tags().len(), 1);
        assert_eq!(metric.fields().len(), 1);
    }

    #[test]
    fn tag_construction_and_accessors() {
        let tag = PyTag::new("test_key".to_string(), "test_value".to_string());

        assert_eq!(tag.key(), "test_key");
        assert_eq!(tag.value(), "test_value");
    }

    #[test]
    fn field_construction_and_accessors() {
        let value = PyValue::int(42);
        let field = PyField::new("test_field".to_string(), value.clone());

        assert_eq!(field.key(), "test_field");
        assert_eq!(field.value(), value);
    }

    #[test]
    fn value_variants() {
        let float_val = PyValue::float(3.14);
        assert!(float_val.is_float());
        assert_eq!(float_val.as_float().unwrap(), 3.14);

        let int_val = PyValue::int(-42);
        assert!(int_val.is_int());
        assert_eq!(int_val.as_int().unwrap(), -42);

        let uint_val = PyValue::uint(42);
        assert!(uint_val.is_uint());
        assert_eq!(uint_val.as_uint().unwrap(), 42);

        let bool_val = PyValue::bool(true);
        assert!(bool_val.is_bool());
        assert_eq!(bool_val.as_bool().unwrap(), true);

        let string_val = PyValue::string("hello".to_string());
        assert!(string_val.is_string());
        assert_eq!(string_val.as_string().unwrap(), "hello");
    }

    #[test]
    fn socket_error_info_exposes_variant_details() {
        let error = IntegrationSocketError::Deserialise {
            error: serde_json::from_str::<serde_json::Value>("not-json").unwrap_err(),
            payload: "not-json".to_string(),
        };

        let info = PySocketErrorInfo::from_socket_error(error);

        Python::with_gil(|py| {
            assert_eq!(info.kind(), "Deserialise");
            assert!(info.message().contains("Deserialising JSON error"));

            let details = info.details(py).expect("details to resolve");
            let details = details.expect("details dictionary");
            let details = details.bind(py);

            let payload_obj = details
                .get_item("payload")
                .unwrap()
                .expect("payload entry");
            let payload: String = payload_obj.extract().expect("payload string");
            assert_eq!(payload, "not-json");

            let error_obj = details
                .get_item("error")
                .unwrap()
                .expect("error entry");
            let error_message: String = error_obj.extract().expect("error string");
            let expected_error = serde_json::from_str::<serde_json::Value>("not-json")
                .unwrap_err()
                .to_string();
            assert_eq!(error_message, expected_error);
        });
    }

    #[test]
    fn socket_error_to_py_err_sets_exception_attributes() {
        let error = IntegrationSocketError::DeserialiseBinary {
            error: serde_json::from_slice::<serde_json::Value>(b"not-json").unwrap_err(),
            payload: vec![1_u8, 2, 3],
        };

        let py_err = crate::error::socket_error_to_py_err(error);

        Python::with_gil(|py| {
            let instance = py_err.into_py(py).into_bound(py);
            let exc_type = py.get_type_bound::<PySocketErrorExc>();
            assert!(instance.is_instance(exc_type.as_any()).unwrap());

            let kind: String = instance.getattr("kind").unwrap().extract().unwrap();
            assert_eq!(kind, "DeserialiseBinary");

            let details_any = instance.getattr("details").unwrap();
            let details = details_any.downcast::<PyDict>().unwrap();
            let payload_obj = details
                .get_item("payload")
                .unwrap()
                .expect("payload entry");
            let payload: Vec<u8> = payload_obj.extract().unwrap();
            assert_eq!(payload, vec![1, 2, 3]);

            let message: String = instance.getattr("message").unwrap().extract().unwrap();
            assert!(message.contains("binary payload"));
        });
    }
}
