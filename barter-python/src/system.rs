use crate::{
    PyEngineEvent,
    command::{PyInstrumentFilter, PyOrderRequestCancel, PyOrderRequestOpen, parse_decimal},
    config::PySystemConfig,
    integration::{PySnapUpdates, PySnapshot},
    summary::{PyTradingSummary, summary_to_py},
};
use barter::{
    EngineEvent,
    engine::{
        Engine, Processor,
        audit::{AuditTick, EngineAudit, context::EngineContext},
        clock::{HistoricalClock, LiveClock},
        execution_tx::MultiExchangeTxMap,
        state::{
            EngineState, global::DefaultGlobalData, instrument::data::DefaultInstrumentMarketData,
            trading::TradingState,
        },
    },
    risk::DefaultRiskManager,
    statistic::time::{Annual252, Annual365, Daily},
    strategy::DefaultStrategy,
    system::{
        System,
        builder::{AuditMode, EngineFeedMode, SystemArgs, SystemBuilder},
        config::ExecutionConfig,
    },
};
use barter_data::{
    event::DataKind,
    streams::{
        consumer::{MarketStreamEvent, MarketStreamResult},
        reconnect::{Event, stream::ReconnectingStream},
    },
};
use barter_execution::balance::Balance;
use barter_instrument::{
    Keyed,
    asset::{ExchangeAsset, name::AssetNameInternal},
    exchange::ExchangeId,
    index::IndexedInstruments,
    instrument::InstrumentIndex,
};
use barter_integration::{
    channel::{Tx, UnboundedRx},
    snapshot::{SnapUpdates, Snapshot},
};
use futures::{Stream, StreamExt, stream};
use pyo3::{Bound, exceptions::PyValueError, prelude::*, types::PyDict};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use tokio::sync::mpsc::error::TryRecvError;
use tracing::{info, warn};

type DefaultEngineState = EngineState<DefaultGlobalData, DefaultInstrumentMarketData>;
type TradingEngine = Engine<
    LiveClock,
    DefaultEngineState,
    MultiExchangeTxMap,
    DefaultStrategy<DefaultEngineState>,
    DefaultRiskManager<DefaultEngineState>,
>;
type RunningSystem = System<TradingEngine, EngineEvent>;
type TradingSnapshotTick = AuditTick<DefaultEngineState, EngineContext>;
type TradingEngineAudit = <TradingEngine as Processor<EngineEvent<DataKind>>>::Audit;
type TradingAuditTick = AuditTick<TradingEngineAudit, EngineContext>;
type TradingAuditSnapUpdates = SnapUpdates<TradingSnapshotTick, UnboundedRx<TradingAuditTick>>;

#[pyclass(module = "barter_python", name = "AuditUpdates", unsendable)]
pub struct PyAuditUpdates {
    runtime: Arc<Runtime>,
    receiver: Mutex<Option<UnboundedRx<TradingAuditTick>>>,
}

impl PyAuditUpdates {
    fn new(runtime: Arc<Runtime>, receiver: UnboundedRx<TradingAuditTick>) -> Self {
        Self {
            runtime,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    fn with_receiver<R, T>(&self, func: R) -> PyResult<T>
    where
        R: FnOnce(&mut UnboundedRx<TradingAuditTick>) -> PyResult<T>,
    {
        let mut guard = self
            .receiver
            .lock()
            .map_err(|_| PyValueError::new_err("audit updates receiver poisoned"))?;

        let receiver = guard
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("audit updates stream exhausted"))?;

        func(receiver)
    }
}

#[pymethods]
impl PyAuditUpdates {
    #[pyo3(signature = (timeout=None))]
    pub fn recv(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Option<PyObject>> {
        self.with_receiver(|receiver| {
            let runtime = Arc::clone(&self.runtime);

            let result = if let Some(secs) = timeout {
                if secs.is_sign_negative() {
                    return Err(PyValueError::new_err("timeout must be non-negative"));
                }

                let timeout_duration = Duration::from_secs_f64(secs);
                let rx = &mut receiver.rx;
                runtime
                    .block_on(async { tokio::time::timeout(timeout_duration, rx.recv()).await })
                    .map_err(|_| PyValueError::new_err("timeout elapsed awaiting audit update"))?
            } else {
                runtime.block_on(receiver.rx.recv())
            };

            match result {
                Some(tick) => audit_tick_summary_to_py(py, &tick).map(Some),
                None => Ok(None),
            }
        })
    }

    pub fn try_recv(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        self.with_receiver(|receiver| match receiver.rx.try_recv() {
            Ok(tick) => audit_tick_summary_to_py(py, &tick).map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Ok(None),
        })
    }

    pub fn is_closed(&self) -> PyResult<bool> {
        let guard = self
            .receiver
            .lock()
            .map_err(|_| PyValueError::new_err("audit updates receiver poisoned"))?;
        Ok(guard
            .as_ref()
            .map(|receiver| receiver.rx.is_closed())
            .unwrap_or(true))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(if self.is_closed()? {
            "AuditUpdates(closed=True)".to_string()
        } else {
            "AuditUpdates(closed=False)".to_string()
        })
    }
}

#[pyclass(module = "barter_python", name = "SystemHandle", unsendable)]
pub struct PySystemHandle {
    runtime: Arc<Runtime>,
    system: Mutex<Option<RunningSystem>>,
}

impl PySystemHandle {
    fn new(runtime: Arc<Runtime>, system: RunningSystem) -> Self {
        Self {
            runtime,
            system: Mutex::new(Some(system)),
        }
    }

    fn lock_system(&self) -> PyResult<MutexGuard<'_, Option<RunningSystem>>> {
        self.system
            .lock()
            .map_err(|_| PyValueError::new_err("system handle poisoned"))
    }

    fn take_system(&self) -> PyResult<RunningSystem> {
        let mut guard = self.lock_system()?;
        guard.take().ok_or_else(Self::system_not_running_err)
    }

    fn system_not_running_err() -> PyErr {
        PyValueError::new_err("system is not running")
    }
}

#[pymethods]
impl PySystemHandle {
    /// Return `True` if the underlying system is still running.
    pub fn is_running(&self) -> PyResult<bool> {
        Ok(self.lock_system()?.is_some())
    }

    /// Send an [`EngineEvent`] to the running system.
    pub fn send_event(&self, event: &PyEngineEvent) -> PyResult<()> {
        let guard = self.lock_system()?;
        let system = guard.as_ref().ok_or_else(Self::system_not_running_err)?;

        system
            .feed_tx
            .send(event.inner.clone())
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Send multiple [`EngineEvent`] values to the system in order.
    #[pyo3(signature = (events))]
    pub fn feed_events(&self, py: Python<'_>, events: Vec<Py<PyEngineEvent>>) -> PyResult<()> {
        for event in events {
            let event_ref = event.borrow(py);
            self.send_event(&event_ref)?;
        }
        Ok(())
    }

    /// Take ownership of the audit snapshot and update stream if audit mode is enabled.
    pub fn take_audit(&self, py: Python<'_>) -> PyResult<Option<Py<PySnapUpdates>>> {
        let mut guard = self.lock_system()?;
        let system = guard.as_mut().ok_or_else(Self::system_not_running_err)?;

        match system.take_audit() {
            Some(updates) => build_py_snapupdates(py, Arc::clone(&self.runtime), updates).map(Some),
            None => Ok(None),
        }
    }

    /// Send open order requests to the engine.
    #[pyo3(signature = (requests))]
    pub fn send_open_requests(
        &self,
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestOpen>>,
    ) -> PyResult<()> {
        let event = PyEngineEvent::send_open_requests(py, requests)?;
        self.send_event(&event)
    }

    /// Send cancel order requests to the engine.
    #[pyo3(signature = (requests))]
    pub fn send_cancel_requests(
        &self,
        py: Python<'_>,
        requests: Vec<Py<PyOrderRequestCancel>>,
    ) -> PyResult<()> {
        let event = PyEngineEvent::send_cancel_requests(py, requests)?;
        self.send_event(&event)
    }

    /// Trigger a close positions command using an optional filter.
    #[pyo3(signature = (filter=None))]
    pub fn close_positions(&self, filter: Option<&PyInstrumentFilter>) -> PyResult<()> {
        let event = PyEngineEvent::close_positions(filter);
        self.send_event(&event)
    }

    /// Trigger a cancel orders command using an optional filter.
    #[pyo3(signature = (filter=None))]
    pub fn cancel_orders(&self, filter: Option<&PyInstrumentFilter>) -> PyResult<()> {
        let event = PyEngineEvent::cancel_orders(filter);
        self.send_event(&event)
    }

    /// Toggle algorithmic trading on or off.
    pub fn set_trading_enabled(&self, enabled: bool) -> PyResult<()> {
        let guard = self.lock_system()?;
        let system = guard.as_ref().ok_or_else(Self::system_not_running_err)?;

        let state = if enabled {
            TradingState::Enabled
        } else {
            TradingState::Disabled
        };
        system.trading_state(state);
        Ok(())
    }

    /// Gracefully shut down the system.
    pub fn shutdown(&self, py: Python<'_>) -> PyResult<()> {
        let system = self.take_system()?;
        let runtime = Arc::clone(&self.runtime);

        match py.allow_threads(|| runtime.block_on(system.shutdown())) {
            Ok((_engine, _audit)) => Ok(()),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }

    /// Abort the system without waiting for a graceful shutdown.
    pub fn abort(&self, py: Python<'_>) -> PyResult<()> {
        let system = self.take_system()?;
        let runtime = Arc::clone(&self.runtime);

        match py.allow_threads(|| runtime.block_on(system.abort())) {
            Ok((_engine, _audit)) => Ok(()),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }

    /// Shut down the system and return a trading summary.
    #[pyo3(signature = (risk_free_return = 0.05, interval = None))]
    pub fn shutdown_with_summary(
        &self,
        py: Python<'_>,
        risk_free_return: f64,
        interval: Option<&str>,
    ) -> PyResult<Py<PyTradingSummary>> {
        let system = self.take_system()?;
        let runtime = Arc::clone(&self.runtime);

        let (engine, _audit) = py
            .allow_threads(|| runtime.block_on(system.shutdown()))
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        let decimal_rfr = parse_risk_free_return(risk_free_return)?;
        let summary_interval = parse_summary_interval(interval)?;
        let mut generator = engine.trading_summary_generator(decimal_rfr);
        match summary_interval {
            SummaryInterval::Daily => summary_to_py(py, generator.generate(Daily)),
            SummaryInterval::Annual252 => summary_to_py(py, generator.generate(Annual252)),
            SummaryInterval::Annual365 => summary_to_py(py, generator.generate(Annual365)),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let running = self.lock_system()?.is_some();
        Ok(format!("SystemHandle(running={running})"))
    }
}

/// Start a live or paper trading system using the provided configuration.
#[pyfunction]
#[pyo3(signature = (config, *, trading_enabled = true, initial_balances = None, audit = false))]
pub fn start_system(
    py: Python<'_>,
    config: &PySystemConfig,
    trading_enabled: bool,
    initial_balances: Option<PyObject>,
    audit: bool,
) -> PyResult<PySystemHandle> {
    let runtime = Arc::new(
        RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?,
    );

    let seeded_balances = parse_initial_balances(py, initial_balances)?;

    let audit_mode = if audit {
        AuditMode::Enabled
    } else {
        AuditMode::Disabled
    };

    let mut config_inner = config.clone_inner();

    // Clear initial balances from executions to allow seeded balances to take precedence
    if !seeded_balances.is_empty() {
        for execution in &mut config_inner.executions {
            let ExecutionConfig::Mock(mock) = execution;
            mock.initial_state.balances.clear();
        }
    }

    let instruments = IndexedInstruments::new(config_inner.instruments.drain(..));
    let market_stream = stream::pending::<MarketStreamEvent<InstrumentIndex, DataKind>>();

    let args = SystemArgs::new(
        &instruments,
        config_inner.executions,
        LiveClock,
        DefaultStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
        DefaultGlobalData,
        |_| DefaultInstrumentMarketData::default(),
    );

    let trading_state = if trading_enabled {
        TradingState::Enabled
    } else {
        TradingState::Disabled
    };

    let system_build = SystemBuilder::new(args)
        .engine_feed_mode(EngineFeedMode::Stream)
        .audit_mode(audit_mode)
        .trading_state(trading_state)
        .balances(seeded_balances)
        .build::<EngineEvent, _>()
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let system = runtime
        .block_on(system_build.init_with_runtime(runtime.handle().clone()))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    Ok(PySystemHandle::new(runtime, system))
}

/// Run a historic backtest using a [`SystemConfig`] and market data events encoded as JSON.
#[pyfunction]
#[pyo3(signature = (config, market_data_path, risk_free_return = 0.05, interval = None, initial_balances = None))]
pub fn run_historic_backtest(
    py: Python<'_>,
    config: &PySystemConfig,
    market_data_path: &str,
    risk_free_return: f64,
    interval: Option<&str>,
    initial_balances: Option<PyObject>,
) -> PyResult<Py<PyTradingSummary>> {
    let (clock, market_stream) =
        load_historic_clock_and_market_stream(Path::new(market_data_path))?;

    let seeded_balances = parse_initial_balances(py, initial_balances)?;

    let mut config_inner = config.clone_inner();

    // Clear initial balances from executions to allow seeded balances to take precedence
    if !seeded_balances.is_empty() {
        for execution in &mut config_inner.executions {
            let ExecutionConfig::Mock(mock) = execution;
            mock.initial_state.balances.clear();
        }
    }
    let instruments = IndexedInstruments::new(config_inner.instruments.drain(..));

    let args = SystemArgs::new(
        &instruments,
        config_inner.executions,
        clock,
        DefaultStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
        DefaultGlobalData,
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
        .balances(seeded_balances)
        .build::<EngineEvent, _>()
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let system = runtime
        .block_on(system_build.init_with_runtime(runtime.handle().clone()))
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let (engine, _audit) = runtime
        .block_on(system.shutdown_after_backtest())
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let decimal_rfr = parse_risk_free_return(risk_free_return)?;
    let summary_interval = parse_summary_interval(interval)?;

    let mut summary = engine.trading_summary_generator(decimal_rfr);
    match summary_interval {
        SummaryInterval::Daily => summary_to_py(py, summary.generate(Daily)),
        SummaryInterval::Annual252 => summary_to_py(py, summary.generate(Annual252)),
        SummaryInterval::Annual365 => summary_to_py(py, summary.generate(Annual365)),
    }
}

fn build_py_snapupdates(
    py: Python<'_>,
    runtime: Arc<Runtime>,
    snap_updates: TradingAuditSnapUpdates,
) -> PyResult<Py<PySnapUpdates>> {
    let TradingAuditSnapUpdates { snapshot, updates } = snap_updates;

    let snapshot_value = snapshot_summary_to_py(py, &snapshot)?;
    let snapshot_inner = Snapshot::new(snapshot_value.clone_ref(py));
    let py_snapshot = Py::new(py, PySnapshot::from_inner(snapshot_inner))?;

    let py_updates = Py::new(py, PyAuditUpdates::new(runtime, updates))?;
    let snap_updates_value =
        PySnapUpdates::__new__(py, py_snapshot.clone_ref(py), py_updates.to_object(py))?;

    Py::new(py, snap_updates_value)
}

fn snapshot_summary_to_py(py: Python<'_>, snapshot: &TradingSnapshotTick) -> PyResult<PyObject> {
    let summary = PyDict::new_bound(py);
    summary.set_item(
        "trading_enabled",
        matches!(snapshot.event.trading, TradingState::Enabled),
    )?;
    summary.set_item("asset_count", snapshot.event.assets.0.len())?;
    summary.set_item("instrument_count", snapshot.event.instruments.0.len())?;

    let root = PyDict::new_bound(py);
    root.set_item("context", context_to_py(py, &snapshot.context)?)?;
    root.set_item("state_summary", summary)?;

    Ok(root.into_py(py))
}

fn audit_tick_summary_to_py(py: Python<'_>, tick: &TradingAuditTick) -> PyResult<PyObject> {
    let event_dict = PyDict::new_bound(py);

    match &tick.event {
        EngineAudit::FeedEnded => {
            event_dict.set_item("kind", "FeedEnded")?;
        }
        EngineAudit::Process(process) => {
            event_dict.set_item("kind", "Process")?;
            event_dict.set_item("event_type", engine_event_kind(&process.event))?;
            event_dict.set_item("output_count", process.outputs.len())?;
            event_dict.set_item("error_count", process.errors.len())?;
        }
    }

    let root = PyDict::new_bound(py);
    root.set_item("context", context_to_py(py, &tick.context)?)?;
    root.set_item("event", event_dict)?;

    Ok(root.into_py(py))
}

fn context_to_py(py: Python<'_>, context: &EngineContext) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("sequence", context.sequence.0)?;
    dict.set_item("time", context.time.to_rfc3339())?;
    Ok(dict.into())
}

fn engine_event_kind(event: &EngineEvent<DataKind>) -> &'static str {
    match event {
        EngineEvent::Shutdown(_) => "Shutdown",
        EngineEvent::Command(_) => "Command",
        EngineEvent::TradingStateUpdate(_) => "TradingStateUpdate",
        EngineEvent::Account(_) => "Account",
        EngineEvent::Market(_) => "Market",
    }
}

fn parse_risk_free_return(value: f64) -> PyResult<Decimal> {
    Decimal::from_f64(value).ok_or_else(|| PyValueError::new_err("risk_free_return must be finite"))
}

fn parse_summary_interval(value: Option<&str>) -> PyResult<SummaryInterval> {
    match value {
        None => Ok(SummaryInterval::Daily),
        Some(raw) => {
            let normalized: String = raw
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();

            match normalized.as_str() {
                "" => Ok(SummaryInterval::Daily),
                "daily" => Ok(SummaryInterval::Daily),
                "annual252" => Ok(SummaryInterval::Annual252),
                "annual365" => Ok(SummaryInterval::Annual365),
                _ => Err(PyValueError::new_err(format!(
                    "invalid interval '{raw}'. valid values are: daily, annual_252, annual_365",
                    raw = raw.trim()
                ))),
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum SummaryInterval {
    Daily,
    Annual252,
    Annual365,
}

fn parse_initial_balances(
    py: Python<'_>,
    values: Option<PyObject>,
) -> PyResult<Vec<Keyed<ExchangeAsset<AssetNameInternal>, Balance>>> {
    let Some(values) = values else {
        return Ok(Vec::new());
    };

    let values = values.bind(py);

    if values.is_none() {
        return Ok(Vec::new());
    }

    let items: Vec<PyObject> = values.extract()?;
    let mut results = Vec::with_capacity(items.len());

    for (index, item) in items.into_iter().enumerate() {
        let mapping = item.bind(py).downcast::<PyDict>().map_err(|_| {
            PyValueError::new_err(format!("initial_balances[{index}] must be a mapping"))
        })?;

        let entry = parse_initial_balance_entry(index, &mapping)?;
        results.push(entry);
    }

    Ok(results)
}

fn parse_initial_balance_entry(
    index: usize,
    mapping: &Bound<'_, PyDict>,
) -> PyResult<Keyed<ExchangeAsset<AssetNameInternal>, Balance>> {
    let exchange_value = mapping.get_item("exchange")?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "initial_balances[{index}] missing 'exchange' field"
        ))
    })?;

    let exchange_label: String = exchange_value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "initial_balances[{index}].exchange must be a string"
        ))
    })?;
    let exchange = parse_exchange_identifier(index, &exchange_label)?;

    let asset_value = mapping.get_item("asset")?.ok_or_else(|| {
        PyValueError::new_err(format!("initial_balances[{index}] missing 'asset' field"))
    })?;

    let asset_label: String = asset_value.extract().map_err(|_| {
        PyValueError::new_err(format!("initial_balances[{index}].asset must be a string"))
    })?;

    let total_value = mapping.get_item("total")?.ok_or_else(|| {
        PyValueError::new_err(format!("initial_balances[{index}] missing 'total' field"))
    })?;

    let total_label = format!("initial_balances[{index}].total");
    let total = parse_decimal(
        total_value
            .extract::<f64>()
            .map_err(|_| PyValueError::new_err(format!("{} must be numeric", total_label)))?,
        &total_label,
    )?;

    let free = match mapping.get_item("free")? {
        Some(value) => {
            let free_label = format!("initial_balances[{index}].free");
            parse_decimal(
                value.extract::<f64>().map_err(|_| {
                    PyValueError::new_err(format!("{} must be numeric", free_label))
                })?,
                &free_label,
            )?
        }
        None => total,
    };

    if free > total {
        return Err(PyValueError::new_err(format!(
            "initial_balances[{index}] free balance cannot exceed total"
        )));
    }

    let asset = AssetNameInternal::from(asset_label.as_str());
    let balance = Balance::new(total, free);

    Ok(Keyed::new(ExchangeAsset::new(exchange, asset), balance))
}

fn parse_exchange_identifier(index: usize, raw: &str) -> PyResult<ExchangeId> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(PyValueError::new_err(format!(
            "initial_balances[{index}].exchange must not be empty"
        )));
    }

    let normalized = normalized.to_ascii_lowercase();
    let quoted = format!("\"{}\"", normalized);

    serde_json::from_str::<ExchangeId>(&quoted).map_err(|_| {
        PyValueError::new_err(format!(
            "initial_balances[{index}].exchange '{raw}' is not a recognised exchange"
        ))
    })
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
