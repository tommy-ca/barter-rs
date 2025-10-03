use std::env;

use pyo3::{exceptions::PyValueError, prelude::*};
use tracing_subscriber::{EnvFilter, fmt};

const DEFAULT_FILTER: &str = "barter_python=info,barter=warn";

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
