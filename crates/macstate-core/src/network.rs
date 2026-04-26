use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Network {
    pub constrained: bool,
    pub expensive: bool,
    pub interface: Interface,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Interface {
    Wifi,
    Cellular,
    Wired,
    Loopback,
    Other,
}

impl Network {
    pub fn collect() -> Self {
        // TODO: NWPathMonitor via objc2-network.
        // Stub returns safe defaults so the CLI is usable end-to-end.
        Self {
            constrained: false,
            expensive: false,
            interface: Interface::Other,
        }
    }
}
