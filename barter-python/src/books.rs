#![allow(unused_imports)]

use barter_data::books::{
    Asks, Bids, Level, OrderBook, OrderBookSide, mid_price, volume_weighted_mid_price,
};
use pyo3::prelude::*;
use rust_decimal::{Decimal, prelude::FromPrimitive};

/// Wrapper around [`Level`] for Python exposure.
#[pyclass(module = "barter_python", name = "Level", unsendable)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyLevel {
    inner: Level,
}

#[pymethods]
impl PyLevel {
    /// Create a new [`Level`].
    #[new]
    #[pyo3(signature = (price, amount))]
    fn new(price: f64, amount: f64) -> PyResult<Self> {
        if !price.is_finite() || price <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "price must be a positive, finite numeric value",
            ));
        }

        if !amount.is_finite() || amount < 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "amount must be a non-negative, finite numeric value",
            ));
        }

        let price = rust_decimal::Decimal::from_f64(price).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("price must be a finite numeric value")
        })?;
        let amount = rust_decimal::Decimal::from_f64(amount).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("amount must be a finite numeric value")
        })?;

        Ok(Self {
            inner: Level::new(price, amount),
        })
    }

    /// Get the price.
    #[getter]
    fn price(&self) -> String {
        self.inner.price.to_string()
    }

    /// Get the amount.
    #[getter]
    fn amount(&self) -> String {
        self.inner.amount.to_string()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Level(price={}, amount={})",
            self.inner.price, self.inner.amount
        ))
    }
}

/// Wrapper around [`OrderBook`] for Python exposure.
#[pyclass(module = "barter_python", name = "OrderBook", unsendable)]
#[derive(Debug, Clone)]
pub struct PyOrderBook {
    inner: OrderBook,
}

#[pymethods]
impl PyOrderBook {
    /// Create a new [`OrderBook`].
    #[new]
    #[pyo3(signature = (sequence, bids, asks, time_engine=None))]
    fn new(
        sequence: i64,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        time_engine: Option<chrono::DateTime<chrono::Utc>>,
    ) -> PyResult<Self> {
        let bids_levels: Vec<Level> = bids
            .into_iter()
            .map(|(p, a)| {
                if !p.is_finite() || p <= 0.0 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "bid price must be positive and finite",
                    ));
                }
                if !a.is_finite() || a < 0.0 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "bid amount must be non-negative and finite",
                    ));
                }
                let price = rust_decimal::Decimal::from_f64(p).ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("bid price must be finite")
                })?;
                let amount = rust_decimal::Decimal::from_f64(a).ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("bid amount must be finite")
                })?;
                Ok(Level::new(price, amount))
            })
            .collect::<PyResult<Vec<_>>>()?;

        let asks_levels: Vec<Level> = asks
            .into_iter()
            .map(|(p, a)| {
                if !p.is_finite() || p <= 0.0 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ask price must be positive and finite",
                    ));
                }
                if !a.is_finite() || a < 0.0 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "ask amount must be non-negative and finite",
                    ));
                }
                let price = rust_decimal::Decimal::from_f64(p).ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("ask price must be finite")
                })?;
                let amount = rust_decimal::Decimal::from_f64(a).ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("ask amount must be finite")
                })?;
                Ok(Level::new(price, amount))
            })
            .collect::<PyResult<Vec<_>>>()?;

        Ok(Self {
            inner: OrderBook::new(sequence as u64, time_engine, bids_levels, asks_levels),
        })
    }

    /// Get the sequence number.
    #[getter]
    fn sequence(&self) -> u64 {
        self.inner.sequence()
    }

    /// Get the time engine.
    #[getter]
    fn time_engine(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.inner.time_engine()
    }

    /// Get the bids as list of (price, amount) tuples.
    fn bids(&self) -> Vec<(String, String)> {
        self.inner
            .bids()
            .levels()
            .iter()
            .map(|l| (l.price.to_string(), l.amount.to_string()))
            .collect()
    }

    /// Get the asks as list of (price, amount) tuples.
    fn asks(&self) -> Vec<(String, String)> {
        self.inner
            .asks()
            .levels()
            .iter()
            .map(|l| (l.price.to_string(), l.amount.to_string()))
            .collect()
    }

    /// Calculate the mid-price.
    fn mid_price(&self) -> Option<String> {
        self.inner.mid_price().map(|p| p.to_string())
    }

    /// Calculate the volume weighted mid-price.
    fn volume_weighted_mid_price(&self) -> Option<String> {
        self.inner.volume_weighed_mid_price().map(|p| p.to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "OrderBook(sequence={}, bids={}, asks={})",
            self.inner.sequence(),
            self.inner.bids().levels().len(),
            self.inner.asks().levels().len()
        ))
    }
}

/// Calculate the mid-price from best bid and ask prices.
#[pyfunction]
pub fn calculate_mid_price(best_bid_price: f64, best_ask_price: f64) -> PyResult<String> {
    let bid = rust_decimal::Decimal::from_f64(best_bid_price)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_bid_price must be finite"))?;
    let ask = rust_decimal::Decimal::from_f64(best_ask_price)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_ask_price must be finite"))?;

    Ok(mid_price(bid, ask).to_string())
}

/// Calculate the volume weighted mid-price from best bid and ask levels.
#[pyfunction]
pub fn calculate_volume_weighted_mid_price(
    best_bid_price: f64,
    best_bid_amount: f64,
    best_ask_price: f64,
    best_ask_amount: f64,
) -> PyResult<String> {
    let bid_price = rust_decimal::Decimal::from_f64(best_bid_price)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_bid_price must be finite"))?;
    let bid_amount = rust_decimal::Decimal::from_f64(best_bid_amount)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_bid_amount must be finite"))?;
    let ask_price = rust_decimal::Decimal::from_f64(best_ask_price)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_ask_price must be finite"))?;
    let ask_amount = rust_decimal::Decimal::from_f64(best_ask_amount)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("best_ask_amount must be finite"))?;

    let bid_level = Level::new(bid_price, bid_amount);
    let ask_level = Level::new(ask_price, ask_amount);

    Ok(volume_weighted_mid_price(bid_level, ask_level).to_string())
}
