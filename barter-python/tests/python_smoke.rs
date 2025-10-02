#![cfg(feature = "python-tests")]

use barter_python::barter_python;
use pyo3::{prelude::*, types::PyModule};

#[test]
fn shutdown_event_is_terminal() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let shutdown = module.getattr("shutdown_event")?.call0()?;
        let is_terminal: bool = shutdown.call_method0("is_terminal")?.extract()?;
        assert!(is_terminal);

        Ok(())
    })
    .unwrap();
}

#[test]
fn timed_f64_round_trip() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let datetime_mod = PyModule::import_bound(py, "datetime")?;
        let dt = datetime_mod
            .getattr("datetime")?
            .call1((2024, 1, 1, 0, 0, 0))?;

        let timed = module.getattr("timed_f64")?.call1((42.5_f64, dt.clone()))?;
        let value: f64 = timed.getattr("value")?.extract()?;
        let time_repr: String = timed.getattr("time")?.repr()?.extract()?;

        assert_eq!(value, 42.5);
        assert_eq!(time_repr, "datetime.datetime(2024, 1, 1, 0, 0)");

        Ok(())
    })
    .unwrap();
}
