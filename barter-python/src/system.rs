use crate::{
    PyEngineEvent, PySequence,
    collection::{PyNoneOneOrMany, wrap_none_one_or_many},
    command::{
        DefaultOrderRequestCancel, DefaultOrderRequestOpen, PyInstrumentFilter,
        PyOrderRequestCancel, PyOrderRequestOpen,
    },
    common::{SummaryInterval, parse_initial_balances, parse_summary_interval},
    config::PySystemConfig,
    execution::PyTradeId,
    instrument::{PyInstrumentIndex, PySide},
    integration::{PySnapUpdates, PySnapshot},
    summary::{PyTradingSummary, decimal_to_py, summary_to_py},
};
use barter::engine::{
    EngineOutput,
    action::{ActionOutput, send_requests::SendRequestsOutput},
    error::EngineError,
    state::position::PositionExited,
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
use barter_execution::{order::OrderEvent, trade::TradeId};
use barter_instrument::{
    Side, asset::QuoteAsset, index::IndexedInstruments, instrument::InstrumentIndex,
};
use barter_integration::{
    channel::{Tx, UnboundedRx},
    collection::none_one_or_many::NoneOneOrMany,
    snapshot::{SnapUpdates, Snapshot},
};
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt, stream};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyList},
};
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

#[pyclass(module = "barter_python", name = "PositionExit", unsendable)]
pub struct PyPositionExit {
    instrument: InstrumentIndex,
    side: Side,
    price_entry_average: Decimal,
    quantity_abs_max: Decimal,
    pnl_realised: Decimal,
    fees_enter: Decimal,
    fees_exit: Decimal,
    time_enter: DateTime<Utc>,
    time_exit: DateTime<Utc>,
    trades: Vec<TradeId>,
}

impl PyPositionExit {
    fn from_position(exit: &PositionExited<QuoteAsset, InstrumentIndex>) -> Self {
        Self {
            instrument: exit.instrument,
            side: exit.side,
            price_entry_average: exit.price_entry_average,
            quantity_abs_max: exit.quantity_abs_max,
            pnl_realised: exit.pnl_realised,
            fees_enter: exit.fees_enter.fees,
            fees_exit: exit.fees_exit.fees,
            time_enter: exit.time_enter,
            time_exit: exit.time_exit,
            trades: exit.trades.clone(),
        }
    }

    fn trades_to_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::empty_bound(py);
        for trade in &self.trades {
            let trade_id = PyTradeId::from_inner(trade.clone());
            let trade_value = Py::new(py, trade_id)?;
            list.append(trade_value.into_py(py))?;
        }
        Ok(list.into_py(py))
    }
}

#[pymethods]
impl PyPositionExit {
    #[getter]
    pub fn instrument(&self, py: Python<'_>) -> PyResult<Py<PyInstrumentIndex>> {
        Py::new(py, PyInstrumentIndex::from_inner(self.instrument))
    }

    #[getter]
    pub fn side(&self, py: Python<'_>) -> PyResult<Py<PySide>> {
        Py::new(py, PySide::from_side(self.side))
    }

    #[getter]
    pub fn price_entry_average(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.price_entry_average)
    }

    #[getter]
    pub fn quantity_abs_max(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.quantity_abs_max)
    }

    #[getter]
    pub fn pnl_realised(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.pnl_realised)
    }

    #[getter]
    pub fn fees_enter(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.fees_enter)
    }

    #[getter]
    pub fn fees_exit(&self, py: Python<'_>) -> PyResult<PyObject> {
        decimal_to_py(py, self.fees_exit)
    }

    #[getter]
    pub fn time_enter(&self) -> String {
        self.time_enter.to_rfc3339()
    }

    #[getter]
    pub fn time_exit(&self) -> String {
        self.time_exit.to_rfc3339()
    }

    #[getter]
    pub fn trades(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.trades_to_list(py)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        let instrument = self.instrument(py)?;
        dict.set_item("instrument", instrument.into_py(py))?;
        let side = self.side(py)?;
        dict.set_item("side", side.into_py(py))?;
        dict.set_item(
            "price_entry_average",
            decimal_to_py(py, self.price_entry_average)?,
        )?;
        dict.set_item(
            "quantity_abs_max",
            decimal_to_py(py, self.quantity_abs_max)?,
        )?;
        dict.set_item("pnl_realised", decimal_to_py(py, self.pnl_realised)?)?;
        dict.set_item("fees_enter", decimal_to_py(py, self.fees_enter)?)?;
        dict.set_item("fees_exit", decimal_to_py(py, self.fees_exit)?)?;
        dict.set_item("time_enter", self.time_enter())?;
        dict.set_item("time_exit", self.time_exit())?;
        dict.set_item("trades", self.trades_to_list(py)?)?;
        Ok(dict.into_py(py))
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PositionExit(instrument={}, side={:?}, quantity_abs_max={}, pnl_realised={})",
            self.instrument, self.side, self.quantity_abs_max, self.pnl_realised
        )
    }
}

enum PyEngineOutputInner {
    Commanded { output: Py<PyActionOutput> },
    OnTradingDisabled { payload: PyObject },
    AccountDisconnect { payload: PyObject },
    MarketDisconnect { payload: PyObject },
    PositionExit { output: Py<PyPositionExit> },
    AlgoOrders { payload: PyObject },
}

impl PyEngineOutputInner {
    fn variant(&self) -> &'static str {
        match self {
            Self::Commanded { .. } => "Commanded",
            Self::OnTradingDisabled { .. } => "OnTradingDisabled",
            Self::AccountDisconnect { .. } => "AccountDisconnect",
            Self::MarketDisconnect { .. } => "MarketDisconnect",
            Self::PositionExit { .. } => "PositionExit",
            Self::AlgoOrders { .. } => "AlgoOrders",
        }
    }

    fn commanded(&self, py: Python<'_>) -> Option<Py<PyActionOutput>> {
        match self {
            Self::Commanded { output } => Some(output.clone_ref(py)),
            _ => None,
        }
    }

    fn trading_disabled(&self, py: Python<'_>) -> Option<PyObject> {
        match self {
            Self::OnTradingDisabled { payload } => Some(payload.clone_ref(py)),
            _ => None,
        }
    }

    fn account_disconnect(&self, py: Python<'_>) -> Option<PyObject> {
        match self {
            Self::AccountDisconnect { payload } => Some(payload.clone_ref(py)),
            _ => None,
        }
    }

    fn market_disconnect(&self, py: Python<'_>) -> Option<PyObject> {
        match self {
            Self::MarketDisconnect { payload } => Some(payload.clone_ref(py)),
            _ => None,
        }
    }

    fn position_exit(&self, py: Python<'_>) -> Option<Py<PyPositionExit>> {
        match self {
            Self::PositionExit { output } => Some(output.clone_ref(py)),
            _ => None,
        }
    }

    fn algo_orders(&self, py: Python<'_>) -> Option<PyObject> {
        match self {
            Self::AlgoOrders { payload } => Some(payload.clone_ref(py)),
            _ => None,
        }
    }

    fn has_payload(&self) -> bool {
        match self {
            Self::Commanded { .. }
            | Self::OnTradingDisabled { .. }
            | Self::AccountDisconnect { .. }
            | Self::MarketDisconnect { .. }
            | Self::PositionExit { .. }
            | Self::AlgoOrders { .. } => true,
        }
    }
}

#[pyclass(module = "barter_python", name = "EngineOutput", unsendable)]
pub struct PyEngineOutput {
    inner: PyEngineOutputInner,
}

impl PyEngineOutput {
    fn from_engine_output<OnTradingDisabled, OnDisconnect>(
        py: Python<'_>,
        output: &EngineOutput<OnTradingDisabled, OnDisconnect>,
    ) -> PyResult<Py<PyEngineOutput>>
    where
        EngineOutput<OnTradingDisabled, OnDisconnect>: Serialize,
        OnTradingDisabled: Serialize,
        OnDisconnect: Serialize,
    {
        let inner = match output {
            EngineOutput::Commanded(action) => {
                let output = PyActionOutput::from_action(py, action)?;
                PyEngineOutputInner::Commanded { output }
            }
            EngineOutput::OnTradingDisabled(value) => {
                let payload = serialize_to_py_object(py, value)?;
                PyEngineOutputInner::OnTradingDisabled { payload }
            }
            EngineOutput::AccountDisconnect(value) => {
                let payload = serialize_to_py_object(py, value)?;
                PyEngineOutputInner::AccountDisconnect { payload }
            }
            EngineOutput::MarketDisconnect(value) => {
                let payload = serialize_to_py_object(py, value)?;
                PyEngineOutputInner::MarketDisconnect { payload }
            }
            EngineOutput::PositionExit(value) => {
                let output = Py::new(py, PyPositionExit::from_position(value))?;
                PyEngineOutputInner::PositionExit { output }
            }
            EngineOutput::AlgoOrders(value) => {
                let payload = serialize_to_py_object(py, value)?;
                PyEngineOutputInner::AlgoOrders { payload }
            }
        };

        Py::new(py, PyEngineOutput { inner })
    }
}

#[pymethods]
impl PyEngineOutput {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.inner.variant()
    }

    #[getter]
    pub fn commanded(&self, py: Python<'_>) -> Option<Py<PyActionOutput>> {
        self.inner.commanded(py)
    }

    #[getter]
    pub fn trading_disabled(&self, py: Python<'_>) -> Option<PyObject> {
        self.inner.trading_disabled(py)
    }

    #[getter]
    pub fn account_disconnect(&self, py: Python<'_>) -> Option<PyObject> {
        self.inner.account_disconnect(py)
    }

    #[getter]
    pub fn market_disconnect(&self, py: Python<'_>) -> Option<PyObject> {
        self.inner.market_disconnect(py)
    }

    #[getter]
    pub fn position_exit(&self, py: Python<'_>) -> Option<Py<PyPositionExit>> {
        self.inner.position_exit(py)
    }

    #[getter]
    pub fn algo_orders(&self, py: Python<'_>) -> Option<PyObject> {
        self.inner.algo_orders(py)
    }

    #[getter]
    pub fn other(&self, _py: Python<'_>) -> Option<PyObject> {
        None
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "EngineOutput(variant={}, has_payload={})",
            self.inner.variant(),
            self.inner.has_payload()
        ))
    }
}

#[pyclass(module = "barter_python", name = "SendRequestsOutput", unsendable)]
pub struct PySendRequestsOutput {
    variant: &'static str,
    sent: Py<PyNoneOneOrMany>,
    errors: Py<PyNoneOneOrMany>,
    sent_len: usize,
    error_len: usize,
}

impl PySendRequestsOutput {
    fn new(
        variant: &'static str,
        sent: Py<PyNoneOneOrMany>,
        errors: Py<PyNoneOneOrMany>,
        sent_len: usize,
        error_len: usize,
    ) -> Self {
        Self {
            variant,
            sent,
            errors,
            sent_len,
            error_len,
        }
    }

    fn counts(&self) -> (usize, usize) {
        (self.sent_len, self.error_len)
    }

    fn is_empty_inner(&self) -> bool {
        self.sent_len == 0 && self.error_len == 0
    }
}

#[pymethods]
impl PySendRequestsOutput {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.variant
    }

    #[getter]
    pub fn sent(&self, py: Python<'_>) -> Py<PyNoneOneOrMany> {
        self.sent.clone_ref(py)
    }

    #[getter]
    pub fn errors(&self, py: Python<'_>) -> Py<PyNoneOneOrMany> {
        self.errors.clone_ref(py)
    }

    #[getter]
    pub fn sent_count(&self) -> usize {
        self.sent_len
    }

    #[getter]
    pub fn error_count(&self) -> usize {
        self.error_len
    }

    #[getter]
    pub fn is_empty(&self) -> bool {
        self.is_empty_inner()
    }

    pub fn to_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.sent
            .bind(py)
            .call_method0("to_list")
            .map(|result| result.into_py(py))
    }

    pub fn errors_to_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.errors
            .bind(py)
            .call_method0("to_list")
            .map(|result| result.into_py(py))
    }

    pub fn __len__(&self) -> usize {
        self.sent_len
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.sent
            .bind(py)
            .call_method0("__iter__")
            .map(|result| result.into_py(py))
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "SendRequestsOutput(variant={}, sent={}, errors={})",
            self.variant, self.sent_len, self.error_len
        ))
    }
}

#[pyclass(module = "barter_python", name = "ClosePositionsOutput", unsendable)]
pub struct PyClosePositionsOutput {
    cancels: Py<PySendRequestsOutput>,
    opens: Py<PySendRequestsOutput>,
}

impl PyClosePositionsOutput {
    fn new(cancels: Py<PySendRequestsOutput>, opens: Py<PySendRequestsOutput>) -> Self {
        Self { cancels, opens }
    }

    fn is_empty_inner(&self, py: Python<'_>) -> bool {
        let cancels_empty = {
            let cancels_ref = self.cancels.bind(py).borrow();
            cancels_ref.is_empty_inner()
        };
        let opens_empty = {
            let opens_ref = self.opens.bind(py).borrow();
            opens_ref.is_empty_inner()
        };
        cancels_empty && opens_empty
    }
}

#[pymethods]
impl PyClosePositionsOutput {
    #[getter]
    pub fn cancels(&self, py: Python<'_>) -> Py<PySendRequestsOutput> {
        self.cancels.clone_ref(py)
    }

    #[getter]
    pub fn opens(&self, py: Python<'_>) -> Py<PySendRequestsOutput> {
        self.opens.clone_ref(py)
    }

    #[getter]
    pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.is_empty_inner(py))
    }

    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let (cancel_sent, cancel_errors) = {
            let cancels_ref = self.cancels.bind(py).borrow();
            cancels_ref.counts()
        };
        let (open_sent, open_errors) = {
            let opens_ref = self.opens.bind(py).borrow();
            opens_ref.counts()
        };

        Ok(format!(
            "ClosePositionsOutput(cancels_sent={}, cancels_errors={}, opens_sent={}, opens_errors={})",
            cancel_sent, cancel_errors, open_sent, open_errors
        ))
    }
}

enum PyActionOutputInner {
    CancelOrders {
        output: Py<PySendRequestsOutput>,
    },
    OpenOrders {
        output: Py<PySendRequestsOutput>,
    },
    ClosePositions {
        output: Py<PyClosePositionsOutput>,
    },
    Other {
        original_variant: &'static str,
        payload: Py<PyAny>,
    },
}

impl PyActionOutputInner {
    fn variant(&self) -> &'static str {
        match self {
            Self::CancelOrders { .. } => "CancelOrders",
            Self::OpenOrders { .. } => "OpenOrders",
            Self::ClosePositions { .. } => "ClosePositions",
            Self::Other { .. } => "Other",
        }
    }

    fn original_variant(&self) -> &'static str {
        match self {
            Self::CancelOrders { .. } => "CancelOrders",
            Self::OpenOrders { .. } => "OpenOrders",
            Self::ClosePositions { .. } => "ClosePositions",
            Self::Other {
                original_variant, ..
            } => original_variant,
        }
    }

    fn is_empty(&self, py: Python<'_>) -> bool {
        match self {
            Self::CancelOrders { output } | Self::OpenOrders { output } => {
                output.bind(py).borrow().is_empty_inner()
            }
            Self::ClosePositions { output } => {
                let close_ref = output.bind(py).borrow();
                close_ref.is_empty_inner(py)
            }
            Self::Other { .. } => false,
        }
    }

    fn clone_cancel_orders(&self, py: Python<'_>) -> Option<Py<PySendRequestsOutput>> {
        match self {
            Self::CancelOrders { output } => Some(output.clone_ref(py)),
            _ => None,
        }
    }

    fn clone_open_orders(&self, py: Python<'_>) -> Option<Py<PySendRequestsOutput>> {
        match self {
            Self::OpenOrders { output } => Some(output.clone_ref(py)),
            _ => None,
        }
    }

    fn clone_close_positions(&self, py: Python<'_>) -> Option<Py<PyClosePositionsOutput>> {
        match self {
            Self::ClosePositions { output } => Some(output.clone_ref(py)),
            _ => None,
        }
    }

    fn clone_other(&self, py: Python<'_>) -> Option<PyObject> {
        match self {
            Self::Other { payload, .. } => Some(payload.clone_ref(py).into_py(py)),
            _ => None,
        }
    }
}

#[pyclass(module = "barter_python", name = "ActionOutput", unsendable)]
pub struct PyActionOutput {
    inner: PyActionOutputInner,
}

impl PyActionOutput {
    fn from_action(py: Python<'_>, output: &ActionOutput) -> PyResult<Py<PyActionOutput>> {
        let inner = match output {
            ActionOutput::CancelOrders(result) => {
                let wrapper = send_requests_output_to_py(
                    py,
                    "CancelOrders",
                    result,
                    order_request_cancel_to_py,
                )?;
                PyActionOutputInner::CancelOrders { output: wrapper }
            }
            ActionOutput::OpenOrders(result) => {
                let wrapper =
                    send_requests_output_to_py(py, "OpenOrders", result, order_request_open_to_py)?;
                PyActionOutputInner::OpenOrders { output: wrapper }
            }
            ActionOutput::ClosePositions(result) => {
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
                let close_wrapper = PyClosePositionsOutput::new(cancels, opens);
                let close_wrapper = Py::new(py, close_wrapper)?;
                PyActionOutputInner::ClosePositions {
                    output: close_wrapper,
                }
            }
            ActionOutput::GenerateAlgoOrders(result) => {
                let payload = serialize_to_py_object(py, result)?;
                let dict = PyDict::new_bound(py);
                dict.set_item("variant", "GenerateAlgoOrders")?;
                dict.set_item("payload", payload)?;
                PyActionOutputInner::Other {
                    original_variant: "GenerateAlgoOrders",
                    payload: dict.into_py(py),
                }
            }
        };

        Py::new(py, PyActionOutput { inner })
    }
}

#[pymethods]
impl PyActionOutput {
    #[getter]
    pub fn variant(&self) -> &'static str {
        self.inner.variant()
    }

    #[getter]
    pub fn original_variant(&self) -> &'static str {
        self.inner.original_variant()
    }

    #[getter]
    pub fn cancel_orders(&self, py: Python<'_>) -> Option<Py<PySendRequestsOutput>> {
        self.inner.clone_cancel_orders(py)
    }

    #[getter]
    pub fn open_orders(&self, py: Python<'_>) -> Option<Py<PySendRequestsOutput>> {
        self.inner.clone_open_orders(py)
    }

    #[getter]
    pub fn close_positions(&self, py: Python<'_>) -> Option<Py<PyClosePositionsOutput>> {
        self.inner.clone_close_positions(py)
    }

    #[getter]
    pub fn other(&self, py: Python<'_>) -> Option<PyObject> {
        self.inner.clone_other(py)
    }

    #[getter]
    pub fn is_empty(&self, py: Python<'_>) -> bool {
        self.inner.is_empty(py)
    }

    pub fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "ActionOutput(variant={}, original_variant={}, empty={})",
            self.inner.variant(),
            self.inner.original_variant(),
            self.inner.is_empty(py)
        ))
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
) -> PyResult<Py<PyEngineOutput>>
where
    EngineOutput<OnTradingDisabled, OnDisconnect>: Serialize,
    OnTradingDisabled: Serialize,
    OnDisconnect: Serialize,
{
    PyEngineOutput::from_engine_output(py, output)
}

fn action_output_to_py(py: Python<'_>, output: &ActionOutput) -> PyResult<PyObject> {
    PyActionOutput::from_action(py, output).map(|value| value.into_py(py))
}

fn send_requests_output_to_py<State, F>(
    py: Python<'_>,
    variant: &'static str,
    output: &SendRequestsOutput<State>,
    converter: F,
) -> PyResult<Py<PySendRequestsOutput>>
where
    State: Clone,
    F: Fn(Python<'_>, &OrderEvent<State>) -> PyResult<PyObject>,
{
    let sent = order_requests_to_py(py, &output.sent, &converter)?;
    let errors = order_request_errors_to_py(py, &output.errors, &converter)?;
    let wrapper = PySendRequestsOutput::new(
        variant,
        sent,
        errors,
        output.sent.len(),
        output.errors.len(),
    );
    Py::new(py, wrapper)
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
    use barter::engine::action::{
        generate_algo_orders::GenerateAlgoOrdersOutput, send_requests::SendCancelsAndOpensOutput,
    };
    use barter_execution::{
        order::{
            OrderKey, OrderKind, TimeInForce,
            id::{ClientOrderId, StrategyId},
            request::{OrderRequestCancel, OrderRequestOpen, RequestCancel, RequestOpen},
        },
        trade::AssetFees,
    };
    use barter_instrument::{Side, exchange::ExchangeIndex, instrument::InstrumentIndex};
    use chrono::{TimeZone, Utc};
    use pyo3::types::PyList;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;

    #[test]
    fn action_output_open_orders_exposes_structured_wrapper() {
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
            let action = py_object.bind(py);

            assert_eq!(
                action.get_type().name().expect("action type name"),
                "ActionOutput"
            );

            let variant: String = action
                .getattr("variant")
                .expect("variant attribute")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "OpenOrders");

            let open_wrapper = action
                .getattr("open_orders")
                .expect("open_orders attribute");
            assert_eq!(
                open_wrapper.get_type().name().expect("wrapper type name"),
                "SendRequestsOutput"
            );

            let len: usize = open_wrapper
                .call_method0("__len__")
                .expect("__len__ call")
                .extract()
                .expect("__len__ result");
            assert_eq!(len, 1);

            let sent = open_wrapper.getattr("sent").expect("sent attribute");
            assert_eq!(
                sent.get_type().name().expect("sent type name"),
                "NoneOneOrMany"
            );

            let sent_list_obj = sent.call_method0("to_list").expect("sent to_list");
            let sent_list = sent_list_obj.downcast::<PyList>().expect("sent to PyList");
            assert_eq!(sent_list.len(), 1);
            let first = sent_list.get_item(0).expect("first item");
            assert!(first.is_instance_of::<PyOrderRequestOpen>());

            let errors = open_wrapper.getattr("errors").expect("errors attribute");
            let errors_list_obj = errors.call_method0("to_list").expect("errors to_list");
            let errors_list = errors_list_obj
                .downcast::<PyList>()
                .expect("errors to PyList");
            assert!(errors_list.is_empty());
        });
    }

    #[test]
    fn action_output_close_positions_exposes_nested_wrappers() {
        Python::with_gil(|py| {
            let key = OrderKey {
                exchange: ExchangeIndex(7),
                instrument: InstrumentIndex(3),
                strategy: StrategyId::new("closure"),
                cid: ClientOrderId::new("cid-close"),
            };

            let cancel_request = OrderRequestCancel {
                key: key.clone(),
                state: RequestCancel::new(None),
            };

            let open_request = OrderRequestOpen {
                key: key.clone(),
                state: RequestOpen::new(
                    Side::Sell,
                    Decimal::from_f64(99.95).unwrap(),
                    Decimal::from_f64(0.5).unwrap(),
                    OrderKind::Limit,
                    TimeInForce::GoodUntilCancelled { post_only: true },
                ),
            };

            let cancels = SendRequestsOutput::new(
                NoneOneOrMany::One(cancel_request.clone()),
                NoneOneOrMany::None,
            );
            let opens = SendRequestsOutput::new(
                NoneOneOrMany::One(open_request.clone()),
                NoneOneOrMany::None,
            );

            let combined = SendCancelsAndOpensOutput::new(cancels, opens);
            let py_object = action_output_to_py(py, &ActionOutput::ClosePositions(combined))
                .expect("convert close positions");
            let action = py_object.bind(py);

            assert_eq!(
                action.get_type().name().expect("action type"),
                "ActionOutput"
            );
            let variant: String = action
                .getattr("variant")
                .expect("variant attribute")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "ClosePositions");

            let close_wrapper = action
                .getattr("close_positions")
                .expect("close_positions attribute");
            assert_eq!(
                close_wrapper.get_type().name().expect("close type name"),
                "ClosePositionsOutput"
            );

            let cancels_wrapper = close_wrapper.getattr("cancels").expect("cancels attribute");
            assert_eq!(
                cancels_wrapper
                    .get_type()
                    .name()
                    .expect("cancels type name"),
                "SendRequestsOutput"
            );

            let opens_wrapper = close_wrapper.getattr("opens").expect("opens attribute");
            assert_eq!(
                opens_wrapper.get_type().name().expect("opens type name"),
                "SendRequestsOutput"
            );

            let cancels_len: usize = cancels_wrapper
                .call_method0("__len__")
                .expect("cancels len")
                .extract()
                .expect("cancels usize");
            assert_eq!(cancels_len, 1);

            let opens_len: usize = opens_wrapper
                .call_method0("__len__")
                .expect("opens len")
                .extract()
                .expect("opens usize");
            assert_eq!(opens_len, 1);

            let opens_sent = opens_wrapper.getattr("sent").expect("opens sent");
            let opens_list_obj = opens_sent.call_method0("to_list").expect("opens to_list");
            let opens_list = opens_list_obj.downcast::<PyList>().expect("opens PyList");
            let first_open = opens_list.get_item(0).expect("first open");
            assert!(first_open.is_instance_of::<PyOrderRequestOpen>());

            let cancels_sent = cancels_wrapper.getattr("sent").expect("cancels sent");
            let cancels_list_obj = cancels_sent
                .call_method0("to_list")
                .expect("cancels to_list");
            let cancels_list = cancels_list_obj
                .downcast::<PyList>()
                .expect("cancels PyList");
            let first_cancel = cancels_list.get_item(0).expect("first cancel");
            assert!(first_cancel.is_instance_of::<PyOrderRequestCancel>());
        });
    }

    #[test]
    fn action_output_generate_algo_orders_falls_back_to_other_variant() {
        Python::with_gil(|py| {
            let fallback = GenerateAlgoOrdersOutput::default();
            let py_object = action_output_to_py(py, &ActionOutput::GenerateAlgoOrders(fallback))
                .expect("convert generate algo orders");
            let action = py_object.bind(py);

            let variant: String = action
                .getattr("variant")
                .expect("variant attribute")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "Other");

            let other = action.getattr("other").expect("other attribute");
            assert_eq!(other.get_type().name().expect("other type"), "dict");

            let open_none = action
                .getattr("open_orders")
                .expect("open_orders attribute");
            assert!(open_none.is_none());

            let cancel_none = action
                .getattr("cancel_orders")
                .expect("cancel_orders attribute");
            assert!(cancel_none.is_none());
        });
    }

    #[test]
    fn engine_output_commanded_wraps_action_output() {
        Python::with_gil(|py| {
            let key = OrderKey {
                exchange: ExchangeIndex(2),
                instrument: InstrumentIndex(4),
                strategy: StrategyId::new("engine-commanded"),
                cid: ClientOrderId::new("cid-7"),
            };

            let request = OrderRequestOpen {
                key: key.clone(),
                state: RequestOpen::new(
                    Side::Sell,
                    Decimal::from_f64(120.0).unwrap(),
                    Decimal::from_f64(0.25).unwrap(),
                    OrderKind::Limit,
                    TimeInForce::GoodUntilCancelled { post_only: true },
                ),
            };

            let output =
                SendRequestsOutput::new(NoneOneOrMany::One(request.clone()), NoneOneOrMany::None);

            let engine_output: EngineOutput<(), ()> =
                EngineOutput::Commanded(ActionOutput::OpenOrders(output));
            let py_output = engine_output_to_py(py, &engine_output).expect("convert engine output");
            let wrapper = py_output.bind(py);

            assert_eq!(
                wrapper.get_type().name().expect("wrapper type"),
                "EngineOutput"
            );

            let variant: String = wrapper
                .getattr("variant")
                .expect("variant attribute")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "Commanded");

            let commanded = wrapper.getattr("commanded").expect("commanded attribute");
            assert!(commanded.is_instance_of::<PyActionOutput>());

            let open_orders = commanded.getattr("open_orders").expect("open_orders attr");
            assert!(open_orders.is_instance_of::<PySendRequestsOutput>());
        });
    }

    #[test]
    fn engine_output_position_exit_exposes_structured_payload() {
        Python::with_gil(|py| {
            let exit = PositionExited {
                instrument: InstrumentIndex(11),
                side: Side::Buy,
                price_entry_average: Decimal::from_f64(101.0).unwrap(),
                quantity_abs_max: Decimal::from_f64(1.5).unwrap(),
                pnl_realised: Decimal::from_f64(4.25).unwrap(),
                fees_enter: AssetFees::quote_fees(Decimal::from_f64(0.05).unwrap()),
                fees_exit: AssetFees::quote_fees(Decimal::from_f64(0.08).unwrap()),
                time_enter: Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap(),
                time_exit: Utc.with_ymd_and_hms(2024, 6, 1, 11, 0, 0).unwrap(),
                trades: vec![TradeId::new("t-1"), TradeId::new("t-2")],
            };

            let engine_output = EngineOutput::<(), ()>::PositionExit(exit);
            let py_output = engine_output_to_py(py, &engine_output).expect("convert position exit");
            let wrapper = py_output.bind(py);

            let variant: String = wrapper
                .getattr("variant")
                .expect("variant attribute")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "PositionExit");

            let position_exit = wrapper
                .getattr("position_exit")
                .expect("position_exit attr");
            assert!(position_exit.is_instance_of::<PyPositionExit>());

            let instrument = position_exit
                .getattr("instrument")
                .expect("instrument attr");
            assert!(instrument.is_instance_of::<PyInstrumentIndex>());

            let trades = position_exit.getattr("trades").expect("trades attr");
            let trades_list = trades.downcast::<PyList>().expect("trades list");
            assert_eq!(trades_list.len(), 2);
        });
    }

    #[test]
    fn engine_output_trading_disabled_serializes_payload() {
        Python::with_gil(|py| {
            let engine_output: EngineOutput<String, ()> =
                EngineOutput::OnTradingDisabled(String::from("disabled"));
            let py_output =
                engine_output_to_py(py, &engine_output).expect("convert trading disabled output");
            let wrapper = py_output.bind(py);

            let variant: String = wrapper
                .getattr("variant")
                .expect("variant attr")
                .extract()
                .expect("variant string");
            assert_eq!(variant, "OnTradingDisabled");

            let payload = wrapper
                .getattr("trading_disabled")
                .expect("trading_disabled attr");
            let value: String = payload.extract().expect("payload string");
            assert_eq!(value, "disabled");

            let commanded = wrapper.getattr("commanded").expect("commanded attr");
            assert!(commanded.is_none());
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
