use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Network {
    /// True when the path is on a Low Data Mode network
    /// (`NWPath.isConstrained`).
    pub constrained: bool,
    /// True when the path is on an expensive interface such as cellular or
    /// a personal hotspot (`NWPath.isExpensive`).
    pub expensive: bool,
    pub interface: Interface,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
#[cfg_attr(
    feature = "schema",
    schemars(description = "Primary interface kind reported by NWPathMonitor for the current path. `other` is reported when the path uses none of the well-known kinds (e.g. unknown VPN).")
)]
pub enum Interface {
    Wifi,
    Cellular,
    Wired,
    Loopback,
    Other,
}

#[cfg(target_os = "macos")]
impl Network {
    pub fn collect() -> Self {
        use macstate_sys::nwpath::{snapshot, InterfaceType};
        use std::time::Duration;

        // NWPathMonitor delivers its first update very quickly (<100ms typical),
        // but allow generous slack on cold starts.
        let Some(snap) = snapshot(Duration::from_secs(2)) else {
            return Self {
                constrained: false,
                expensive: false,
                interface: Interface::Other,
            };
        };

        let interface = match snap.interface {
            Some(InterfaceType::Wifi) => Interface::Wifi,
            Some(InterfaceType::Cellular) => Interface::Cellular,
            Some(InterfaceType::WiredEthernet) => Interface::Wired,
            Some(InterfaceType::Loopback) => Interface::Loopback,
            _ => Interface::Other,
        };

        Self {
            constrained: snap.constrained,
            expensive: snap.expensive,
            interface,
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl Network {
    pub fn collect() -> Self {
        Self {
            constrained: false,
            expensive: false,
            interface: Interface::Other,
        }
    }
}
