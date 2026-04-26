//! macstate-core: macOS system signal collection.
//!
//! Exposes network and power signals as serde-serializable structs.

pub mod network;
pub mod power;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
