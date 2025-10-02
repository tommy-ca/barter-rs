use crate::config::PySystemConfig;
use barter::{
    EngineEvent,
    engine::{
        clock::HistoricalClock,
        state::{
            global::DefaultGlobalData, instrument::data::DefaultInstrumentMarketData,
            trading::TradingState,
        },
    },
    risk::DefaultRiskManager,
    statistic::{summary::TradingSummary, time::Daily},
    strategy::DefaultStrategy,
    system::builder::{AuditMode, EngineFeedMode, SystemArgs, SystemBuilder},
};
use barter_data::{
    event::DataKind,
    streams::{
        consumer::{MarketStreamEvent, MarketStreamResult},
        reconnect::{Event, stream::ReconnectingStream},
    },
};
use barter_instrument::{index::IndexedInstruments, instrument::InstrumentIndex};
use futures::{Stream, StreamExt};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyModule};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::{fs::File, io::Read, path::Path};
use tokio::runtime::Builder as RuntimeBuilder;
use tracing::{info, warn};

/// Run a historic backtest using a [`SystemConfig`] and market data events encoded as JSON.
#[pyfunction]
#[pyo3(signature = (config, market_data_path, risk_free_return = 0.05))]
pub fn run_historic_backtest(
    py: Python<'_>,
    config: &PySystemConfig,
    market_data_path: &str,
    risk_free_return: f64,
) -> PyResult<PyObject> {
    let (clock, market_stream) =
        load_historic_clock_and_market_stream(Path::new(market_data_path))?;

    let mut config_inner = config.clone_inner();
    let instruments = IndexedInstruments::new(config_inner.instruments.drain(..));

    let args = SystemArgs::new(
        &instruments,
        config_inner.executions,
        clock,
        DefaultStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
        DefaultGlobalData::default(),
        |_| DefaultInstrumentMarketData::default(),
    );

    let runtime = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let system_build = SystemBuilder::new(args)
        .engine_feed_mode(EngineFeedMode::Stream)
        .audit_mode(AuditMode::Disabled)
        .trading_state(TradingState::Enabled)
        .build::<EngineEvent, _>()
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let system = runtime
        .block_on(system_build.init_with_runtime(runtime.handle().clone()))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let (engine, _audit) = runtime
        .block_on(system.shutdown_after_backtest())
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let decimal_rfr = Decimal::from_f64(risk_free_return)
        .ok_or_else(|| PyValueError::new_err("risk_free_return must be finite"))?;

    let mut summary = engine.trading_summary_generator(decimal_rfr);
    let summary = summary.generate(Daily);

    summary_to_py(py, summary)
}

fn load_historic_clock_and_market_stream(
    path: &Path,
) -> PyResult<(
    HistoricalClock,
    impl Stream<Item = MarketStreamEvent<InstrumentIndex, DataKind>> + Send + 'static,
)> {
    let mut file = File::open(path).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let events =
        serde_json::from_str::<Vec<MarketStreamResult<InstrumentIndex, DataKind>>>(&contents)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let time_exchange_first = events
        .iter()
        .find_map(|result| match result {
            Event::Item(Ok(event)) => Some(event.time_exchange),
            _ => None,
        })
        .ok_or_else(|| PyValueError::new_err("market data contains no events"))?;

    let clock = HistoricalClock::new(time_exchange_first);

    let stream = futures::stream::iter(events)
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"))
        .inspect(|event| match event {
            Event::Reconnecting(exchange) => {
                info!(%exchange, "sending historical disconnection to Engine")
            }
            Event::Item(event) => {
                info!(
                    exchange = %event.exchange,
                    instrument = %event.instrument,
                    kind = event.kind.kind_name(),
                    "sending historical event to Engine"
                )
            }
        });

    Ok((clock, stream))
}

fn summary_to_py(py: Python<'_>, summary: TradingSummary<Daily>) -> PyResult<PyObject> {
    let json =
        serde_json::to_string(&summary).map_err(|err| PyValueError::new_err(err.to_string()))?;

    let json_module = PyModule::import_bound(py, "json")?;
    let loads = json_module.getattr("loads")?;
    Ok(loads.call1((json,))?.into_py(py))
}
