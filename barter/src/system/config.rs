/// Configuration module for trading system components.
///
/// Provides data structures for configuring various aspects of a trading system,
/// including instruments and execution components.
use barter_execution::client::mock::MockExecutionConfig;
use barter_instrument::{
    Underlying,
    asset::{Asset, name::AssetNameExchange},
    exchange::ExchangeId,
    instrument::{
        Instrument,
        kind::{
            InstrumentKind, future::FutureContract, option::OptionContract,
            perpetual::PerpetualContract,
        },
        name::{InstrumentNameExchange, InstrumentNameInternal},
        quote::InstrumentQuoteAsset,
        spec::{InstrumentSpec, InstrumentSpecQuantity, OrderQuantityUnits},
    },
};
use derive_more::From;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Risk manager configuration options for the trading system.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RiskConfiguration {
    /// Global risk thresholds applied across the system.
    pub global: Option<RiskLimits>,

    /// Per-instrument risk overrides identified by instrument index.
    pub instruments: Vec<RiskInstrumentLimits>,
}

impl RiskConfiguration {
    /// Retrieve global risk limits if configured.
    pub fn global(&self) -> Option<&RiskLimits> {
        self.global.as_ref()
    }

    /// Update global risk limits.
    pub fn set_global(&mut self, limits: Option<RiskLimits>) -> Result<(), RiskLimitsError> {
        if let Some(ref limits) = limits {
            limits.validate()?;
        }

        self.global = limits;
        Ok(())
    }

    /// Retrieve risk limits for the given instrument index.
    pub fn instrument_limits(&self, index: usize) -> Option<&RiskLimits> {
        self.instruments
            .iter()
            .find(|entry| entry.index == index)
            .map(|entry| &entry.limits)
    }

    /// Update risk limits for the given instrument index.
    pub fn set_instrument_limits(
        &mut self,
        index: usize,
        limits: Option<RiskLimits>,
    ) -> Result<(), RiskLimitsError> {
        if let Some(ref limits) = limits {
            limits.validate()?;
        }

        if let Some(limits) = limits {
            if let Some(existing) = self
                .instruments
                .iter_mut()
                .find(|entry| entry.index == index)
            {
                existing.limits = limits;
            } else {
                self.instruments
                    .push(RiskInstrumentLimits { index, limits });
            }

            self.instruments.sort_by_key(|entry| entry.index);
        } else {
            self.instruments.retain(|entry| entry.index != index);
        }

        Ok(())
    }
}

/// Risk limits associated with a specific instrument.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct RiskInstrumentLimits {
    /// Index into [`SystemConfig::instruments`].
    pub index: usize,

    /// Risk limits to apply to the instrument.
    pub limits: RiskLimits,
}

/// Individual risk limit thresholds.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RiskLimits {
    /// Maximum notional exposure permitted for a single position.
    pub max_position_notional: Option<Decimal>,

    /// Maximum position quantity permitted.
    pub max_position_quantity: Option<Decimal>,

    /// Maximum leverage permitted for new positions.
    pub max_leverage: Option<Decimal>,

    /// Maximum proportion of account equity allocatable to an instrument.
    pub max_exposure_percent: Option<Decimal>,
}

impl RiskLimits {
    fn validate(&self) -> Result<(), RiskLimitsError> {
        fn ensure_positive(
            value: &Option<Decimal>,
            field: &'static str,
        ) -> Result<(), RiskLimitsError> {
            if let Some(value) = value {
                if value <= &Decimal::ZERO {
                    return Err(RiskLimitsError::NonPositive {
                        field,
                        value: *value,
                    });
                }
            }

            Ok(())
        }

        ensure_positive(&self.max_position_notional, "max_position_notional")?;
        ensure_positive(&self.max_position_quantity, "max_position_quantity")?;
        ensure_positive(&self.max_leverage, "max_leverage")?;

        if let Some(exposure) = self.max_exposure_percent {
            if exposure <= Decimal::ZERO || exposure > Decimal::ONE {
                return Err(RiskLimitsError::ExposurePercentOutOfRange { exposure });
            }
        }

        Ok(())
    }
}

/// Errors that can occur while configuring risk limits.
#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum RiskLimitsError {
    /// A value must be strictly greater than zero.
    #[error("risk limit `{field}` must be positive, received {value}")]
    NonPositive { field: &'static str, value: Decimal },

    /// Exposure percentage must lie within the open interval (0, 1].
    #[error("risk limit `max_exposure_percent` must be in (0, 1], received {exposure}")]
    ExposurePercentOutOfRange { exposure: Decimal },

    /// Provided instrument index is out of bounds.
    #[error("instrument index {index} is out of bounds for {total} instrument(s)")]
    InstrumentIndexOutOfBounds { index: usize, total: usize },
}

/// Top-level configuration for a full trading system.
///
/// Contains configuration for all instruments and execution components.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct SystemConfig {
    /// Configurations for all instruments the system will track.
    pub instruments: Vec<InstrumentConfig>,

    /// Configurations for all execution components.
    pub executions: Vec<ExecutionConfig>,

    /// Optional risk manager configuration.
    #[serde(default)]
    pub risk: RiskConfiguration,
}

impl SystemConfig {
    /// Access the configured risk options.
    pub fn risk(&self) -> &RiskConfiguration {
        &self.risk
    }

    /// Mutable access to the configured risk options.
    pub fn risk_mut(&mut self) -> &mut RiskConfiguration {
        &mut self.risk
    }

    /// Update global risk limits with validation.
    pub fn set_global_risk_limits(
        &mut self,
        limits: Option<RiskLimits>,
    ) -> Result<(), RiskLimitsError> {
        self.risk.set_global(limits)
    }

    /// Update per-instrument risk limits using an instrument index.
    pub fn set_instrument_risk_limits(
        &mut self,
        index: usize,
        limits: Option<RiskLimits>,
    ) -> Result<(), RiskLimitsError> {
        if index >= self.instruments.len() {
            return Err(RiskLimitsError::InstrumentIndexOutOfBounds {
                index,
                total: self.instruments.len(),
            });
        }

        self.risk.set_instrument_limits(index, limits)
    }

    /// Retrieve per-instrument risk limits if configured.
    pub fn instrument_risk_limits(&self, index: usize) -> Option<&RiskLimits> {
        self.risk.instrument_limits(index)
    }
}

/// Convenient minimal instrument configuration, used to generate an [`Instrument`] on startup.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct InstrumentConfig {
    /// Exchange identifier where the instrument is traded.
    pub exchange: ExchangeId,

    /// Exchange-specific name for the instrument (e.g., "BTCUSDT").
    pub name_exchange: InstrumentNameExchange,

    /// Underlying asset pair for the instrument.
    pub underlying: Underlying<AssetNameExchange>,

    /// Quote asset for the instrument.
    pub quote: InstrumentQuoteAsset,

    /// Type of the instrument (spot, perpetual, future, option).
    pub kind: InstrumentKind<AssetNameExchange>,

    /// Optional additional specifications for the instrument.
    pub spec: Option<InstrumentSpec<AssetNameExchange>>,
}

/// Configuration for an execution link.
///
/// Represents different types of execution configurations,
/// currently only supporting mock execution for backtesting.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, From)]
#[serde(untagged)]
pub enum ExecutionConfig {
    /// Mock execution configuration for backtesting
    Mock(MockExecutionConfig),
}

impl From<InstrumentConfig> for Instrument<ExchangeId, Asset> {
    fn from(value: InstrumentConfig) -> Self {
        Self {
            exchange: value.exchange,
            name_internal: InstrumentNameInternal::new_from_exchange_underlying(
                value.exchange,
                &value.underlying.base,
                &value.underlying.quote,
            ),
            name_exchange: value.name_exchange,
            underlying: Underlying {
                base: Asset::new_from_exchange(value.underlying.base),
                quote: Asset::new_from_exchange(value.underlying.quote),
            },
            quote: value.quote,
            kind: match value.kind {
                InstrumentKind::Spot => InstrumentKind::Spot,
                InstrumentKind::Perpetual(contract) => {
                    InstrumentKind::Perpetual(PerpetualContract {
                        contract_size: contract.contract_size,
                        settlement_asset: Asset::new_from_exchange(contract.settlement_asset),
                    })
                }
                InstrumentKind::Future(contract) => InstrumentKind::Future(FutureContract {
                    contract_size: contract.contract_size,
                    settlement_asset: Asset::new_from_exchange(contract.settlement_asset),
                    expiry: contract.expiry,
                }),
                InstrumentKind::Option(contract) => InstrumentKind::Option(OptionContract {
                    contract_size: contract.contract_size,
                    settlement_asset: Asset::new_from_exchange(contract.settlement_asset),
                    kind: contract.kind,
                    exercise: contract.exercise,
                    expiry: contract.expiry,
                    strike: contract.strike,
                }),
            },
            spec: value.spec.map(|spec| InstrumentSpec {
                price: spec.price,
                quantity: InstrumentSpecQuantity {
                    unit: match spec.quantity.unit {
                        OrderQuantityUnits::Asset(asset) => {
                            OrderQuantityUnits::Asset(Asset::new_from_exchange(asset))
                        }
                        OrderQuantityUnits::Contract => OrderQuantityUnits::Contract,
                        OrderQuantityUnits::Quote => OrderQuantityUnits::Quote,
                    },
                    min: spec.quantity.min,
                    increment: spec.quantity.increment,
                },
                notional: spec.notional,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_config() -> SystemConfig {
        SystemConfig {
            instruments: vec![InstrumentConfig {
                exchange: ExchangeId::BinanceSpot,
                name_exchange: InstrumentNameExchange::new("BTCUSDT".to_string()),
                underlying: Underlying {
                    base: AssetNameExchange::new("btc".to_string()),
                    quote: AssetNameExchange::new("usdt".to_string()),
                },
                quote: InstrumentQuoteAsset::UnderlyingQuote,
                kind: InstrumentKind::Spot,
                spec: None,
            }],
            executions: Vec::new(),
            risk: RiskConfiguration::default(),
        }
    }

    #[test]
    fn global_limits_validation() {
        let mut config = sample_config();
        config
            .set_global_risk_limits(Some(RiskLimits {
                max_position_notional: Some(dec!(1000)),
                max_position_quantity: None,
                max_leverage: Some(dec!(2)),
                max_exposure_percent: Some(dec!(0.5)),
            }))
            .unwrap();

        assert_eq!(config.risk().global().unwrap().max_leverage, Some(dec!(2)));
    }

    #[test]
    fn rejects_invalid_exposure() {
        let mut config = sample_config();
        let err = config
            .set_global_risk_limits(Some(RiskLimits {
                max_position_notional: None,
                max_position_quantity: None,
                max_leverage: None,
                max_exposure_percent: Some(dec!(1.5)),
            }))
            .unwrap_err();

        assert!(matches!(
            err,
            RiskLimitsError::ExposurePercentOutOfRange { .. }
        ));
    }

    #[test]
    fn instrument_limits_round_trip() {
        let mut config = sample_config();
        config
            .set_instrument_risk_limits(
                0,
                Some(RiskLimits {
                    max_position_notional: Some(dec!(5000)),
                    max_position_quantity: None,
                    max_leverage: None,
                    max_exposure_percent: None,
                }),
            )
            .unwrap();

        let limits = config.instrument_risk_limits(0).unwrap();
        assert_eq!(limits.max_position_notional, Some(dec!(5000)));

        config.set_instrument_risk_limits(0, None).unwrap();
        assert!(config.instrument_risk_limits(0).is_none());
    }

    #[test]
    fn instrument_index_bounds_check() {
        let mut config = sample_config();
        let err = config
            .set_instrument_risk_limits(
                1,
                Some(RiskLimits {
                    max_position_notional: None,
                    max_position_quantity: None,
                    max_leverage: None,
                    max_exposure_percent: None,
                }),
            )
            .unwrap_err();

        assert!(matches!(
            err,
            RiskLimitsError::InstrumentIndexOutOfBounds { .. }
        ));
    }
}
