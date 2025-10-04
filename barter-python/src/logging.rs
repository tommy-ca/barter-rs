use std::env;

use pyo3::{exceptions::PyValueError, prelude::*};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_FILTER: &str = "barter_python=info,barter=warn";

struct AuditSpanFilter;

impl<S> tracing_subscriber::layer::Layer<S> for AuditSpanFilter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn event_enabled(
        &self,
        _: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if let Some(span) = ctx.lookup_current()
            && span.name() == "audit_replica_state_update_span"
        {
            false
        } else {
            true
        }
    }
}

/// Initialise the global tracing subscriber used by the Rust bindings.
///
/// Returns `True` if the subscriber was installed by this call and `False` if a
/// subscriber was already configured.
#[pyfunction]
#[pyo3(signature = (filter = None, ansi = false))]
pub fn init_tracing(filter: Option<&str>, ansi: bool) -> PyResult<bool> {
    let env_filter = match filter {
        Some(spec) => {
            EnvFilter::try_new(spec).map_err(|err| PyValueError::new_err(err.to_string()))?
        }
        None => match env::var("RUST_LOG") {
            Ok(spec) => {
                EnvFilter::try_new(spec).map_err(|err| PyValueError::new_err(err.to_string()))?
            }
            Err(env::VarError::NotPresent) => EnvFilter::new(DEFAULT_FILTER),
            Err(env::VarError::NotUnicode(_)) => {
                return Err(PyValueError::new_err(
                    "RUST_LOG must contain valid UTF-8 characters",
                ));
            }
        },
    };

    let subscriber = fmt().with_env_filter(env_filter).with_ansi(ansi);

    match subscriber.try_init() {
        Ok(()) => Ok(true),
        Err(_already_set) => Ok(false),
    }
}

/// Initialise JSON logging for the Barter trading engine.
///
/// This sets up structured JSON output for logs, which is useful for log aggregation
/// systems and automated processing.
///
/// Returns `True` if the subscriber was installed by this call and `False` if a
/// subscriber was already configured.
#[pyfunction]
pub fn init_json_logging_py() -> PyResult<bool> {
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .with(AuditSpanFilter);

    match subscriber.try_init() {
        Ok(()) => Ok(true),
        Err(_already_set) => Ok(false),
    }
}
