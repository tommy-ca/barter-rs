use crate::{
    PyEngineEvent, PySequence,
    collection::{PyNoneOneOrMany, wrap_none_one_or_many},
    command::{
        DefaultOrderRequestCancel, DefaultOrderRequestOpen, PyInstrumentFilter,
        PyOrderRequestCancel, PyOrderRequestOpen,
    },
    common::{SummaryInterval, parse_initial_balances, parse_summary_interval},
    config::PySystemConfig,
    integration::{PySnapUpdates, PySnapshot},
    summary::{PyTradingSummary, summary_to_py},
};
use barter::engine::{
    EngineOutput,
    action::{ActionOutput, send_requests::SendRequestsOutput},
    error::EngineError,
};
use barter::{
    EngineEvent, Sequence,
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
use barter_execution::order::OrderEvent;
use barter_instrument::{index::IndexedInstruments, instrument::InstrumentIndex};
use barter_integration::{
    channel::{Tx, UnboundedRx},
    collection::none_one_or_many::NoneOneOrMany,
    snapshot::{SnapUpdates, Snapshot},
};
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt, stream};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Serialize;
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

    fn recv_tick_inner(&self, timeout: Option<f64>) -> PyResult<Option<TradingAuditTick>> {
        let runtime = Arc::clone(&self.runtime);
        self.with_receiver(|receiver| Self::blocking_recv(runtime, receiver, timeout))
    }

    fn try_recv_tick_inner(&self) -> PyResult<Option<TradingAuditTick>> {
        self.with_receiver(|receiver| match receiver.rx.try_recv() {
            Ok(tick) => Ok(Some(tick)),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => Ok(None),
        })
    }

    fn blocking_recv(
        runtime: Arc<Runtime>,
        receiver: &mut UnboundedRx<TradingAuditTick>,
        timeout: Option<f64>,
    ) -> PyResult<Option<TradingAuditTick>> {
        if let Some(secs) = timeout {
            if secs.is_sign_negative() {
                return Err(PyValueError::new_err("timeout must be non-negative"));
            }

            if !secs.is_finite() {
                return Err(PyValueError::new_err("timeout must be finite"));
            }

            let duration = Duration::from_secs_f64(secs);
            runtime
                .block_on(async { tokio::time::timeout(duration, receiver.rx.recv()).await })
                .map_err(|_| PyValueError::new_err("timeout elapsed awaiting audit update"))
        } else {
            Ok(runtime.block_on(receiver.rx.recv()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AuditEventKind {
    FeedEnded,
    Process,
}

impl AuditEventKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::FeedEnded => "FeedEnded",
            Self::Process => "Process",
        }
    }
}

#[pyclass(module = "barter_python", name = "AuditContext", unsendable)]
#[derive(Debug, Copy, Clone)]
pub struct PyAuditContext {
    sequence: Sequence,
    time: DateTime<Utc>,
}

impl PyAuditContext {
    fn new(sequence: Sequence, time: DateTime<Utc>) -> Self {
        Self { sequence, time }
    }
}

#[pyclass(module = "barter_python", name = "AuditEvent", unsendable)]
pub struct PyAuditEvent {
    kind: AuditEventKind,
    event_type: Option<&'static str>,
    output_count: usize,
    error_count: usize,
    outputs: PyObject,
    errors: PyObject,
}

impl PyAuditEvent {
    fn new_feed_ended(py: Python<'_>) -> PyResult<Self> {
        let outputs = Py::new(py, PyNoneOneOrMany::empty())?.into_py(py);
        let errors = Py::new(py, PyNoneOneOrMany::empty())?.into_py(py);
        Ok(Self {
            kind: AuditEventKind::FeedEnded,
            event_type: None,
            output_count: 0,
            error_count: 0,
            outputs,
            errors,
        })
    }

    fn new_process(
        event_type: &'static str,
        output_count: usize,
        error_count: usize,
        outputs: PyObject,
        errors: PyObject,
    ) -> Self {
        Self {
            kind: AuditEventKind::Process,
            event_type: Some(event_type),
            output_count,
            error_count,
            outputs,
            errors,
        }
    }

    fn to_py(&self, py: Python<'_>) -> PyResult<Py<PyAuditEvent>> {
        Py::new(
            py,
            Self {
                kind: self.kind,
                event_type: self.event_type,
                output_count: self.output_count,
                error_count: self.error_count,
                outputs: self.outputs.clone_ref(py),
                errors: self.errors.clone_ref(py),
            },
        )
    }
}

#[pyclass(module = "barter_python", name = "AuditTick", unsendable)]
pub struct PyAuditTick {
    context: PyAuditContext,
    event: PyAuditEvent,
}

impl PyAuditTick {
    fn new(context: PyAuditContext, event: PyAuditEvent) -> Self {
        Self { context, event }
    }
}

#[pymethods]
impl PyAuditContext {
    #[getter]
    pub fn sequence(&self) -> PySequence {
        PySequence::from_inner(self.sequence)
    }

    #[getter]
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let context = EngineContext {
            sequence: self.sequence,
            time: self.time,
        };
        Ok(context_to_py(py, &context)?.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "AuditContext(sequence={}, time={})",
            self.sequence.value(),
            self.time.to_rfc3339()
        ))
    }
}

#[pymethods]
impl PyAuditEvent {
    #[getter]
    pub fn kind(&self) -> &'static str {
        self.kind.as_str()
    }

    #[getter]
    pub fn event_type(&self) -> Option<&'static str> {
        self.event_type
    }

    #[getter]
    pub fn output_count(&self) -> usize {
        self.output_count
    }

    #[getter]
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    #[getter]
    pub fn outputs(&self, py: Python<'_>) -> PyObject {
        self.outputs.clone_ref(py)
    }

    #[getter]
    pub fn errors(&self, py: Python<'_>) -> PyObject {
        self.errors.clone_ref(py)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let event = PyDict::new_bound(py);
        event.set_item("kind", self.kind.as_str())?;
        if let Some(event_type) = self.event_type {
            event.set_item("event_type", event_type)?;
        }
        if matches!(self.kind, AuditEventKind::Process) {
            event.set_item("output_count", self.output_count)?;
            event.set_item("error_count", self.error_count)?;
        }
        event.set_item("outputs", self.outputs.clone_ref(py))?;
        event.set_item("errors", self.errors.clone_ref(py))?;
        Ok(event.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        let event_type = self
            .event_type
            .map(|value| value.to_string())
            .unwrap_or_else(|| "None".to_string());
        Ok(format!(
            "AuditEvent(kind={}, event_type={}, outputs={}, errors={})",
            self.kind.as_str(),
            event_type,
            self.output_count,
            self.error_count,
        ))
    }
}

#[pymethods]
impl PyAuditTick {
    #[getter]
    pub fn context(&self) -> PyAuditContext {
        self.context
    }

    #[getter]
    pub fn event(&self, py: Python<'_>) -> PyResult<Py<PyAuditEvent>> {
        self.event.to_py(py)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let root = PyDict::new_bound(py);
        root.set_item("context", self.context.to_dict(py)?)?;
        root.set_item("event", self.event.to_dict(py)?)?;
        Ok(root.into_py(py))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("AuditTick(kind={})", self.event.kind.as_str()))
    }
}

#[pymethods]
impl PyAuditUpdates {
    #[pyo3(signature = (timeout=None))]
    pub fn recv(&self, py: Python<'_>, timeout: Option<f64>) -> PyResult<Option<PyObject>> {
        match self.recv_tick_inner(timeout)? {
            Some(tick) => audit_tick_summary_to_py(py, &tick).map(Some),
            None => Ok(None),
        }
    }

    pub fn try_recv(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        match self.try_recv_tick_inner()? {
            Some(tick) => audit_tick_summary_to_py(py, &tick).map(Some),
            None => Ok(None),
        }
    }

    #[pyo3(signature = (timeout=None))]
    pub fn recv_tick(
        &self,
        py: Python<'_>,
        timeout: Option<f64>,
    ) -> PyResult<Option<Py<PyAuditTick>>> {
        self.recv_tick_inner(timeout)?
            .map(|tick| audit_tick_to_py(py, &tick))
            .transpose()
    }

    pub fn try_recv_tick(&self, py: Python<'_>) -> PyResult<Option<Py<PyAuditTick>>> {
        self.try_recv_tick_inner()?
            .map(|tick| audit_tick_to_py(py, &tick))
            .transpose()
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
#[pyo3(
    signature = (
        config,
        *,
        trading_enabled = true,
        initial_balances = None,
        audit = false,
        engine_feed_mode = None
    )
)]
pub fn start_system(
    py: Python<'_>,
    config: &PySystemConfig,
    trading_enabled: bool,
    initial_balances: Option<PyObject>,
    audit: bool,
    engine_feed_mode: Option<&str>,
) -> PyResult<PySystemHandle> {
    let runtime = Arc::new(
        RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?,
    );

    let seeded_balances = parse_initial_balances(py, initial_balances)?;
    let feed_mode = parse_engine_feed_mode(engine_feed_mode)?;

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
        .engine_feed_mode(feed_mode)
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
#[pyo3(
    signature = (
        config,
        market_data_path,
        risk_free_return = 0.05,
        interval = None,
        initial_balances = None,
        engine_feed_mode = None
    )
)]
pub fn run_historic_backtest(
    py: Python<'_>,
    config: &PySystemConfig,
    market_data_path: &str,
    risk_free_return: f64,
    interval: Option<&str>,
    initial_balances: Option<PyObject>,
    engine_feed_mode: Option<&str>,
) -> PyResult<Py<PyTradingSummary>> {
    let (clock, market_stream) =
        load_historic_clock_and_market_stream(Path::new(market_data_path))?;

    let seeded_balances = parse_initial_balances(py, initial_balances)?;
    let feed_mode = parse_engine_feed_mode(engine_feed_mode)?;

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
        .engine_feed_mode(feed_mode)
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
        PySnapUpdates::__new__(py, py_snapshot.clone_ref(py), py_updates.into_py(py))?;

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
    let py_tick = audit_tick_to_py(py, tick)?;
    py_tick.bind(py).borrow().to_dict(py)
}

fn audit_tick_to_py(py: Python<'_>, tick: &TradingAuditTick) -> PyResult<Py<PyAuditTick>> {
    let context = PyAuditContext::new(tick.context.sequence, tick.context.time);
    let event = match &tick.event {
        EngineAudit::FeedEnded => PyAuditEvent::new_feed_ended(py)?,
        EngineAudit::Process(process) => {
            let outputs = engine_outputs_to_py(py, &process.outputs)?.into_py(py);
            let errors = wrap_none_one_or_many(py, process.errors.clone())?.into_py(py);
            PyAuditEvent::new_process(
                engine_event_kind(&process.event),
                process.outputs.len(),
                process.errors.len(),
                outputs,
                errors,
            )
        }
    };

    Py::new(py, PyAuditTick::new(context, event))
}

fn context_to_py(py: Python<'_>, context: &EngineContext) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);
    let sequence = PySequence::from_inner(context.sequence);
    let sequence = Py::new(py, sequence)?;
    dict.set_item("sequence", sequence)?;
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

fn serialize_to_py_object<T>(py: Python<'_>, value: &T) -> PyResult<PyObject>
where
    T: Serialize,
{
    let serialized =
        serde_json::to_string(value).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let json_module = PyModule::import_bound(py, "json")?;
    let loads = json_module.getattr("loads")?;
    let loaded = loads.call1((serialized.into_py(py),))?;
    Ok(loaded.into_py(py))
}

fn engine_outputs_to_py<OnTradingDisabled, OnDisconnect>(
    py: Python<'_>,
    outputs: &NoneOneOrMany<EngineOutput<OnTradingDisabled, OnDisconnect>>,
) -> PyResult<Py<PyNoneOneOrMany>>
where
    EngineOutput<OnTradingDisabled, OnDisconnect>: Serialize,
    OnTradingDisabled: Serialize,
    OnDisconnect: Serialize,
{
    let wrapper = match outputs {
        NoneOneOrMany::None => PyNoneOneOrMany::empty(),
        NoneOneOrMany::One(output) => {
            let converted = engine_output_to_py(py, output)?;
            PyNoneOneOrMany {
                inner: NoneOneOrMany::One(converted.into_py(py)),
            }
        }
        NoneOneOrMany::Many(values) => {
            let mut converted = Vec::with_capacity(values.len());
            for output in values {
                converted.push(engine_output_to_py(py, output)?.into_py(py));
            }
            PyNoneOneOrMany {
                inner: NoneOneOrMany::Many(converted),
            }
        }
    };

    Py::new(py, wrapper)
}

fn engine_output_to_py<OnTradingDisabled, OnDisconnect>(
    py: Python<'_>,
    output: &EngineOutput<OnTradingDisabled, OnDisconnect>,
) -> PyResult<PyObject>
where
    EngineOutput<OnTradingDisabled, OnDisconnect>: Serialize,
    OnTradingDisabled: Serialize,
    OnDisconnect: Serialize,
{
    match output {
        EngineOutput::Commanded(action) => action_output_to_py(py, action),
        _ => serialize_to_py_object(py, output),
    }
}

fn action_output_to_py(py: Python<'_>, output: &ActionOutput) -> PyResult<PyObject> {
    match output {
        ActionOutput::CancelOrders(result) => {
            send_requests_output_to_py(py, "CancelOrders", result, order_request_cancel_to_py)
        }
        ActionOutput::OpenOrders(result) => {
            send_requests_output_to_py(py, "OpenOrders", result, order_request_open_to_py)
        }
        ActionOutput::ClosePositions(result) => {
            let dict = PyDict::new_bound(py);
            dict.set_item("variant", "ClosePositions")?;
            let cancels = send_requests_output_to_py(
                py,
                "CancelOrders",
                &result.cancels,
                order_request_cancel_to_py,
            )?;
            let opens = send_requests_output_to_py(
                py,
                "OpenOrders",
                &result.opens,
                order_request_open_to_py,
            )?;
            dict.set_item("cancels", cancels)?;
            dict.set_item("opens", opens)?;
            Ok(dict.into_py(py))
        }
        _ => serialize_to_py_object(py, output),
    }
}

fn send_requests_output_to_py<State, F>(
    py: Python<'_>,
    variant: &str,
    output: &SendRequestsOutput<State>,
    converter: F,
) -> PyResult<PyObject>
where
    State: Clone,
    F: Fn(Python<'_>, &OrderEvent<State>) -> PyResult<PyObject>,
{
    let dict = PyDict::new_bound(py);
    dict.set_item("variant", variant)?;

    let sent = order_requests_to_py(py, &output.sent, &converter)?;
    dict.set_item("sent", sent)?;
    dict.set_item("sent_count", output.sent.len())?;

    let errors = order_request_errors_to_py(py, &output.errors, &converter)?;
    dict.set_item("errors", errors)?;
    dict.set_item("error_count", output.errors.len())?;
    dict.set_item("has_errors", !output.errors.is_none())?;

    Ok(dict.into_py(py))
}

fn order_requests_to_py<State, F>(
    py: Python<'_>,
    value: &NoneOneOrMany<OrderEvent<State>>,
    converter: &F,
) -> PyResult<Py<PyNoneOneOrMany>>
where
    State: Clone,
    F: Fn(Python<'_>, &OrderEvent<State>) -> PyResult<PyObject>,
{
    let wrapper = match value {
        NoneOneOrMany::None => PyNoneOneOrMany::empty(),
        NoneOneOrMany::One(event) => {
            let converted = converter(py, event)?;
            PyNoneOneOrMany {
                inner: NoneOneOrMany::One(converted.into_py(py)),
            }
        }
        NoneOneOrMany::Many(events) => {
            let mut converted = Vec::with_capacity(events.len());
            for event in events {
                converted.push(converter(py, event)?.into_py(py));
            }
            PyNoneOneOrMany {
                inner: NoneOneOrMany::Many(converted),
            }
        }
    };

    Py::new(py, wrapper)
}

fn order_request_errors_to_py<State, F>(
    py: Python<'_>,
    value: &NoneOneOrMany<(OrderEvent<State>, EngineError)>,
    converter: &F,
) -> PyResult<Py<PyNoneOneOrMany>>
where
    State: Clone,
    F: Fn(Python<'_>, &OrderEvent<State>) -> PyResult<PyObject>,
{
    let wrapper = match value {
        NoneOneOrMany::None => PyNoneOneOrMany::empty(),
        NoneOneOrMany::One((order, error)) => {
            let entry = order_request_error_entry_to_py(py, order, error, converter)?;
            PyNoneOneOrMany {
                inner: NoneOneOrMany::One(entry.into_py(py)),
            }
        }
        NoneOneOrMany::Many(entries) => {
            let mut converted = Vec::with_capacity(entries.len());
            for (order, error) in entries {
                converted.push(
                    order_request_error_entry_to_py(py, order, error, converter)?.into_py(py),
                );
            }
            PyNoneOneOrMany {
                inner: NoneOneOrMany::Many(converted),
            }
        }
    };

    Py::new(py, wrapper)
}

fn order_request_error_entry_to_py<State, F>(
    py: Python<'_>,
    order: &OrderEvent<State>,
    error: &EngineError,
    converter: &F,
) -> PyResult<Py<PyAny>>
where
    State: Clone,
    F: Fn(Python<'_>, &OrderEvent<State>) -> PyResult<PyObject>,
{
    let dict = PyDict::new_bound(py);
    dict.set_item("order", converter(py, order)?)?;
    dict.set_item("error", engine_error_to_py(py, error)?)?;
    Ok(dict.into_py(py))
}

fn engine_error_to_py(py: Python<'_>, error: &EngineError) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    match error {
        EngineError::Recoverable(inner) => {
            dict.set_item("variant", "Recoverable")?;
            dict.set_item("message", inner.to_string())?;
        }
        EngineError::Unrecoverable(inner) => {
            dict.set_item("variant", "Unrecoverable")?;
            dict.set_item("message", inner.to_string())?;
        }
    }
    Ok(dict.into_py(py))
}

fn order_request_open_to_py(
    py: Python<'_>,
    request: &DefaultOrderRequestOpen,
) -> PyResult<PyObject> {
    let wrapper = PyOrderRequestOpen::from_inner(request.clone());
    Py::new(py, wrapper).map(|value| value.into_py(py))
}

fn order_request_cancel_to_py(
    py: Python<'_>,
    request: &DefaultOrderRequestCancel,
) -> PyResult<PyObject> {
    let wrapper = PyOrderRequestCancel::from_inner(request.clone());
    Py::new(py, wrapper).map(|value| value.into_py(py))
}

fn parse_risk_free_return(value: f64) -> PyResult<Decimal> {
    Decimal::from_f64(value).ok_or_else(|| PyValueError::new_err("risk_free_return must be finite"))
}

fn parse_engine_feed_mode(value: Option<&str>) -> PyResult<EngineFeedMode> {
    match value {
        None => Ok(EngineFeedMode::Stream),
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "stream" => Ok(EngineFeedMode::Stream),
                "iterator" => Ok(EngineFeedMode::Iterator),
                _ => Err(PyValueError::new_err(format!(
                    "engine_feed_mode must be 'stream' or 'iterator', got {raw}",
                ))),
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use barter_execution::order::{
        OrderKey, OrderKind, TimeInForce,
        id::{ClientOrderId, StrategyId},
        request::{OrderRequestOpen, RequestOpen},
    };
    use barter_instrument::{Side, exchange::ExchangeIndex, instrument::InstrumentIndex};
    use pyo3::types::{PyDict, PyList};
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;

    #[test]
    fn action_output_open_orders_converts_requests_to_wrappers() {
        Python::with_gil(|py| {
            let key = OrderKey {
                exchange: ExchangeIndex(1),
                instrument: InstrumentIndex(2),
                strategy: StrategyId::new("strategy-alpha"),
                cid: ClientOrderId::new("cid-1"),
            };

            let state = RequestOpen::new(
                Side::Buy,
                Decimal::from_f64(101.5).unwrap(),
                Decimal::from_f64(0.75).unwrap(),
                OrderKind::Limit,
                TimeInForce::GoodUntilCancelled { post_only: false },
            );

            let request = OrderRequestOpen {
                key: key.clone(),
                state,
            };

            let output =
                SendRequestsOutput::new(NoneOneOrMany::One(request.clone()), NoneOneOrMany::None);

            let py_object = action_output_to_py(py, &ActionOutput::OpenOrders(output))
                .expect("convert action output");
            let dict = py_object
                .downcast_bound::<PyDict>(py)
                .expect("action output to dict");

            let variant_obj = dict
                .get_item("variant")
                .expect("variant lookup failed")
                .expect("variant present");
            let variant: String = variant_obj.extract().expect("variant string");
            assert_eq!(variant, "OpenOrders");

            let sent_obj = dict
                .get_item("sent")
                .expect("sent lookup failed")
                .expect("sent present");
            let sent_list_obj = sent_obj.call_method0("to_list").expect("to_list succeeds");
            let sent_list = sent_list_obj.downcast::<PyList>().expect("list conversion");

            assert_eq!(sent_list.len(), 1);
            let first = sent_list.get_item(0).expect("first item");
            assert!(first.is_instance_of::<PyOrderRequestOpen>());
        });
    }

    #[test]
    fn parse_engine_feed_mode_defaults_to_stream() {
        assert_eq!(
            parse_engine_feed_mode(None).unwrap(),
            EngineFeedMode::Stream
        );
    }

    #[test]
    fn parse_engine_feed_mode_accepts_known_values() {
        assert_eq!(
            parse_engine_feed_mode(Some("iterator")).unwrap(),
            EngineFeedMode::Iterator
        );
        assert_eq!(
            parse_engine_feed_mode(Some(" STREAM ")).unwrap(),
            EngineFeedMode::Stream
        );
        assert_eq!(
            parse_engine_feed_mode(Some("ItErAtOr")).unwrap(),
            EngineFeedMode::Iterator
        );
    }

    #[test]
    fn parse_engine_feed_mode_rejects_unknown_value() {
        let error = parse_engine_feed_mode(Some("warp")).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("engine_feed_mode"));
        assert!(message.contains("warp"));
    }
}
