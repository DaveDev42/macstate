//! macstate-core: macOS system signal collection.
//!
//! Exposes network and power signals as serde-serializable structs.

pub mod network;
pub mod power;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(description = "Snapshot of macOS system signals exposed by `macstate`.")
)]
pub struct State {
    pub network: network::Network,
    pub power: power::Power,
}

impl State {
    pub fn collect() -> Self {
        Self {
            network: network::Network::collect(),
            power: power::Power::collect(),
        }
    }
}
