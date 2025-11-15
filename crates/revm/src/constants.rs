//! Tempo EVM specific constants.

/// The gas limit for system transactions at the [`Adagio`](tempo_chainspec::hardfork::TempoHardfork::Adagio) hardfork: 300000000 (30M).
pub const SYSTEM_TX_GAS_LIMIT_ADAGIO: u64 = 30_000_000;

/// The gas limit for system transactions after the [`Moderato`](tempo_chainspec::hardfork::TempoHardfork::Moderato) hardfork: 150000000 (150M).
pub const SYSTEM_TX_GAS_LIMIT_MODERATO: u64 = 150_000_000;
