use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Power {
    pub source: Source,
    pub battery_percent: Option<u8>,
    pub low_power_mode: bool,
    pub energy_mode: EnergyMode,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Ac,
    Battery,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EnergyMode {
    Automatic,
    Low,
    High,
}

impl Power {
    pub fn collect() -> Self {
        // TODO: IOKit IOPSCopyPowerSourcesInfo + NSProcessInfo.isLowPowerModeEnabled + pmset parse.
        Self {
            source: Source::Ac,
            battery_percent: None,
            low_power_mode: false,
            energy_mode: EnergyMode::Automatic,
        }
    }
}
