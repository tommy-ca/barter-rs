#![cfg(feature = "python-tests")]

use barter_python::barter_python;
use pyo3::{PyObject, prelude::*, types::PyModule};

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

#[test]
fn system_config_from_json() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let system_config_cls = module.getattr("SystemConfig")?;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let config_path = std::path::Path::new(&manifest_dir)
            .join("..")
            .join("barter")
            .join("examples")
            .join("config")
            .join("system_config.json");

        let system_config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;
        let config_dict = system_config.call_method0("to_dict")?;

        let instruments = config_dict.get_item("instruments").unwrap();
        let instruments: Vec<PyObject> = instruments.extract()?;
        assert!(!instruments.is_empty());

        Ok(())
    })
    .unwrap();
}

#[test]
fn run_historic_backtest_returns_summary() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let system_config_cls = module.getattr("SystemConfig")?;
        let run_backtest = module.getattr("run_historic_backtest")?;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base_path = std::path::Path::new(&manifest_dir).join("..");
        let config_path = base_path
            .join("barter")
            .join("examples")
            .join("config")
            .join("system_config.json");
        let market_path = base_path
            .join("barter")
            .join("examples")
            .join("data")
            .join("binance_spot_market_data_with_disconnect_events.json");

        let system_config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;
        let summary = run_backtest.call1((system_config, market_path.display().to_string()))?;

        let instruments = summary.get_item("instruments").unwrap();
        let instruments: Vec<(String, PyObject)> = instruments.extract()?;
        assert!(!instruments.is_empty());

        Ok(())
    })
    .unwrap();
}
