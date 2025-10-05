#![cfg(feature = "python-tests")]

use barter_python::barter_python;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyModule, PyTuple},
};

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

#[test]
fn start_system_handle_lifecycle() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let system_config_cls = module.getattr("SystemConfig")?;
        let start_system = module.getattr("start_system")?;
        let engine_event_cls = module.getattr("EngineEvent")?;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base_path = std::path::Path::new(&manifest_dir).join("..");
        let config_path = base_path
            .join("barter")
            .join("examples")
            .join("config")
            .join("system_config.json");

        let config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;
        let handle = start_system.call1((config,))?;

        assert!(handle.call_method0("is_running")?.extract::<bool>()?);

        handle.call_method1("set_trading_enabled", (true,))?;
        let trading_disabled = engine_event_cls.call_method1("trading_state", (false,))?;
        handle.call_method1("send_event", (trading_disabled,))?;

        handle.call_method0("shutdown")?;
        assert!(!handle.call_method0("is_running")?.extract::<bool>()?);

        Ok(())
    })
    .unwrap();
}

#[test]
fn engine_event_command_builders() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let order_key_cls = module.getattr("OrderKey")?;
        let order_open_cls = module.getattr("OrderRequestOpen")?;
        let order_cancel_cls = module.getattr("OrderRequestCancel")?;
        let instrument_filter_cls = module.getattr("InstrumentFilter")?;
        let engine_event_cls = module.getattr("EngineEvent")?;

        let key = order_key_cls.call1((0usize, 0usize, "strategy-alpha", Some("cid-open")))?;
        let open = order_open_cls.call1((key.clone(), "buy", 101.25_f64, 0.5_f64))?;
        let open_list = PyList::new_bound(py, &[open]);
        let open_event = engine_event_cls.call_method1("send_open_requests", (open_list,))?;
        assert!(!open_event.call_method0("is_terminal")?.extract::<bool>()?);

        let cancel = order_cancel_cls.call1((key.clone(),))?;
        let cancel_list = PyList::new_bound(py, &[cancel]);
        let cancel_event = engine_event_cls.call_method1("send_cancel_requests", (cancel_list,))?;
        assert!(
            !cancel_event
                .call_method0("is_terminal")?
                .extract::<bool>()?
        );

        let filter = instrument_filter_cls.call_method0("none")?;
        let close_event = engine_event_cls.call_method1("close_positions", (filter.clone(),))?;
        assert!(!close_event.call_method0("is_terminal")?.extract::<bool>()?);

        let cancel_orders_event = engine_event_cls.call_method1("cancel_orders", (filter,))?;
        assert!(
            !cancel_orders_event
                .call_method0("is_terminal")?
                .extract::<bool>()?
        );

        Ok(())
    })
    .unwrap();
}

#[test]
fn system_config_risk_limits_round_trip() {
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

        let config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;

        let risk_limits = config.call_method0("risk_limits")?;
        let risk_dict = risk_limits.downcast::<PyDict>()?;

        let global_limits = risk_dict.get_item("global").unwrap();
        assert!(global_limits.is_none());

        let global = PyDict::new_bound(py);
        global.set_item("max_leverage", 3.5_f64)?;
        global.set_item("max_position_notional", 12_500_f64)?;
        config.call_method1("set_global_risk_limits", (global,))?;

        let per_instrument = PyDict::new_bound(py);
        per_instrument.set_item("max_position_quantity", 2.5_f64)?;
        config.call_method1("set_instrument_risk_limits", (0usize, per_instrument))?;

        let risk_limits = config.call_method0("risk_limits")?;
        let risk_dict = risk_limits.downcast::<PyDict>()?;

        let global_limits = risk_dict.get_item("global").unwrap().unwrap();
        let global_limits = global_limits.downcast::<PyDict>()?;
        let max_leverage_repr: String = global_limits
            .get_item("max_leverage")?
            .unwrap()
            .repr()?
            .extract()?;
        assert!(max_leverage_repr.contains("3.5"));

        let entries = risk_dict.get_item("instruments")?.unwrap();
        let entries = entries.downcast::<PyList>()?;
        let entry = entries
            .iter()
            .find_map(|value| {
                let dict = value.downcast::<PyDict>().ok()?;
                let index_item = dict.get_item("index").ok()?;
                let index: usize = index_item?.extract().ok()?;
                if index == 0 { Some(dict.clone()) } else { None }
            })
            .expect("instrument 0 limits present");

        let instrument_limits = entry.get_item("limits")?.unwrap();
        let instrument_limits = instrument_limits.downcast::<PyDict>()?;
        let qty_repr: String = instrument_limits
            .get_item("max_position_quantity")?
            .unwrap()
            .repr()?
            .extract()?;
        assert!(qty_repr.contains("2.5"));

        config.call_method1("set_instrument_risk_limits", (0usize, py.None()))?;
        let cleared = config.call_method1("get_instrument_risk_limits", (0usize,))?;
        assert!(cleared.is_none());

        config.call_method1("set_global_risk_limits", (py.None(),))?;
        let cleared_global = config.call_method0("risk_limits")?;
        let cleared_global = cleared_global.downcast::<PyDict>()?;
        assert!(cleared_global.get_item("global").unwrap().is_none());

        Ok(())
    })
    .unwrap();
}

#[test]
fn none_one_or_many_python_semantics() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let container_cls = module.getattr("NoneOneOrMany")?;

        let empty = container_cls.call0()?;
        assert_eq!(empty.len()?, 0);
        assert!(empty.getattr("is_none")?.extract::<bool>()?);
        let empty_list: Vec<PyObject> = empty.call_method0("to_list")?.extract()?;
        assert!(empty_list.is_empty());

        let single = container_cls.call1(("value",))?;
        assert_eq!(single.len()?, 1);
        assert!(single.getattr("is_one")?.extract::<bool>()?);
        let single_list: Vec<String> = single.call_method0("to_list")?.extract()?;
        assert_eq!(single_list, vec!["value".to_string()]);

        let sequence = PyList::new_bound(py, &[1, 2, 3]);
        let many = container_cls.call1((sequence,))?;
        assert_eq!(many.len()?, 3);
        assert!(many.getattr("is_many")?.extract::<bool>()?);
        let many_list: Vec<i64> = many.call_method0("to_list")?.extract()?;
        assert_eq!(many_list, vec![1, 2, 3]);

        let repr: String = many.repr()?.extract()?;
        assert!(repr.contains("Many"));

        Ok(())
    })
    .unwrap();
}

#[test]
fn one_or_many_python_semantics() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let container_cls = module.getattr("OneOrMany")?;

        let one = container_cls.call1((42,))?;
        assert_eq!(one.len()?, 1);
        assert!(one.getattr("is_one")?.extract::<bool>()?);
        let one_list: Vec<i64> = one.call_method0("to_list")?.extract()?;
        assert_eq!(one_list, vec![42]);

        let sequence = PyTuple::new_bound(py, &["alpha", "beta"]);
        let many = container_cls.call1((sequence,))?;
        assert_eq!(many.len()?, 2);
        assert!(many.getattr("is_many")?.extract::<bool>()?);
        let many_list: Vec<String> = many.call_method0("to_list")?.extract()?;
        assert_eq!(many_list, vec!["alpha".to_string(), "beta".to_string()]);

        let also_one = container_cls.call1((42,))?;
        assert!(one.eq(&also_one)?);
        assert!(!one.eq(&many)?);

        Ok(())
    })
    .unwrap();
}

#[test]
fn engine_event_serialization_helpers() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let engine_event_cls = module.getattr("EngineEvent")?;
        let json_module = PyModule::import_bound(py, "json")?;

        let dict = PyDict::new_bound(py);
        dict.set_item("Shutdown", PyDict::new_bound(py))?;

        let event = engine_event_cls.call_method1("from_dict", (dict.clone(),))?;
        assert!(event.call_method0("is_terminal")?.extract::<bool>()?);

        let json: String = event.call_method0("to_json")?.extract()?;
        let round_trip = engine_event_cls.call_method1("from_json", (json,))?;
        assert!(round_trip.call_method0("is_terminal")?.extract::<bool>()?);

        let dict_rt = round_trip.call_method0("to_dict")?;
        let flattened: String = json_module
            .getattr("dumps")?
            .call1((dict_rt.clone(),))?
            .extract()?;
        assert!(flattened.contains("Shutdown"));

        Ok(())
    })
    .unwrap();
}

#[test]
fn system_handle_command_helpers() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let system_config_cls = module.getattr("SystemConfig")?;
        let start_system = module.getattr("start_system")?;
        let order_key_cls = module.getattr("OrderKey")?;
        let order_open_cls = module.getattr("OrderRequestOpen")?;
        let order_cancel_cls = module.getattr("OrderRequestCancel")?;
        let instrument_filter_cls = module.getattr("InstrumentFilter")?;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base_path = std::path::Path::new(&manifest_dir).join("..");
        let config_path = base_path
            .join("barter")
            .join("examples")
            .join("config")
            .join("system_config.json");

        let config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;
        let handle = start_system.call1((config,))?;

        assert!(handle.call_method0("is_running")?.extract::<bool>()?);

        let key = order_key_cls.call1((0usize, 0usize, "strategy-beta", Some("cid-beta")))?;
        let open = order_open_cls.call1((key.clone(), "sell", 99.5_f64, 0.25_f64))?;
        let cancel = order_cancel_cls.call1((key,))?;

        let open_list = PyList::new_bound(py, &[open]);
        handle.call_method1("send_open_requests", (open_list,))?;

        let cancel_list = PyList::new_bound(py, &[cancel]);
        handle.call_method1("send_cancel_requests", (cancel_list,))?;

        let filter = instrument_filter_cls.call_method0("none")?;
        handle.call_method1("close_positions", (filter.clone(),))?;
        handle.call_method1("cancel_orders", (filter,))?;

        handle.call_method0("shutdown")?;
        assert!(!handle.call_method0("is_running")?.extract::<bool>()?);

        Ok(())
    })
    .unwrap();
}

#[test]
fn system_handle_shutdown_with_summary() {
    Python::with_gil(|py| -> PyResult<()> {
        let module = PyModule::new_bound(py, "barter_python")?;
        barter_python(py, &module)?;

        let system_config_cls = module.getattr("SystemConfig")?;
        let start_system = module.getattr("start_system")?;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base_path = std::path::Path::new(&manifest_dir).join("..");
        let config_path = base_path
            .join("barter")
            .join("examples")
            .join("config")
            .join("system_config.json");

        let config =
            system_config_cls.call_method1("from_json", (config_path.display().to_string(),))?;
        let handle = start_system.call1((config,))?;

        let summary = handle.call_method1("shutdown_with_summary", (0.02_f64,))?;
        for key in [
            "time_engine_start",
            "time_engine_end",
            "instruments",
            "assets",
        ] {
            assert!(summary.get_item(key).is_ok());
        }

        Ok(())
    })
    .unwrap();
}
