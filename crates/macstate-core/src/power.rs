use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Power {
    pub source: Source,
    /// Battery charge as an integer percentage (0–100), or `null` when no
    /// battery is present (e.g. Mac mini, iMac).
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 100)))]
    pub battery_percent: Option<u8>,
    /// Whether Low Power Mode is currently active
    /// (`NSProcessInfo.isLowPowerModeEnabled`). This is the runtime state,
    /// not a configured preference.
    pub low_power_mode: bool,
    pub energy_mode: EnergyMode,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
#[cfg_attr(
    feature = "schema",
    schemars(description = "Whether the system is currently drawing from AC or battery (IOPSGetProvidingPowerSourceType).")
)]
pub enum Source {
    Ac,
    Battery,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
#[cfg_attr(
    feature = "schema",
    schemars(description = "Configured energy preference for the current power source, read from IOPM active preferences (the same data source pmset(8) uses). `high` is only available on Apple Silicon Pro/Max. `unknown` is returned when the preference is unreadable or carries an unrecognized value.")
)]
pub enum EnergyMode {
    Automatic,
    Low,
    High,
    /// The IOPM preference was missing or carried a value we don't recognize.
    Unknown,
}

#[cfg(target_os = "macos")]
impl Power {
    pub fn collect() -> Self {
        let (source, battery_percent) = read_power_source();
        let low_power_mode = macstate_sys::objc::is_low_power_mode_enabled();
        let energy_mode = read_energy_mode(source);
        Self {
            source,
            battery_percent,
            low_power_mode,
            energy_mode,
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl Power {
    pub fn collect() -> Self {
        Self {
            source: Source::Ac,
            battery_percent: None,
            low_power_mode: false,
            energy_mode: EnergyMode::Unknown,
        }
    }
}

#[cfg(target_os = "macos")]
fn read_power_source() -> (Source, Option<u8>) {
    use macstate_sys::cf::{
        cfstring_to_string, dict_get_i32, CFArrayGetCount, CFArrayGetValueAtIndex, CFOwned,
    };
    use macstate_sys::iokit::{
        kIOPSACPowerValue, kIOPSCurrentCapacityKey, kIOPSMaxCapacityKey,
        IOPSCopyPowerSourcesInfo, IOPSCopyPowerSourcesList, IOPSGetPowerSourceDescription,
        IOPSGetProvidingPowerSourceType,
    };

    unsafe {
        let snapshot = match CFOwned::from_create(IOPSCopyPowerSourcesInfo()) {
            Some(s) => s,
            None => return (Source::Ac, None),
        };

        let provider = IOPSGetProvidingPowerSourceType(snapshot.as_ptr());
        let source = match cfstring_to_string(provider) {
            Some(s) if s == kIOPSACPowerValue => Source::Ac,
            Some(_) => Source::Battery,
            None => Source::Ac,
        };

        let list = match CFOwned::from_create(IOPSCopyPowerSourcesList(snapshot.as_ptr())) {
            Some(l) => l,
            None => return (source, None),
        };

        let count = CFArrayGetCount(list.as_ptr());
        let mut percent: Option<u8> = None;
        for i in 0..count {
            let ps = CFArrayGetValueAtIndex(list.as_ptr(), i);
            if ps.is_null() {
                continue;
            }
            let desc = IOPSGetPowerSourceDescription(snapshot.as_ptr(), ps);
            if desc.is_null() {
                continue;
            }
            let cur = dict_get_i32(desc, kIOPSCurrentCapacityKey);
            let max = dict_get_i32(desc, kIOPSMaxCapacityKey);
            if let (Some(cur), Some(max)) = (cur, max) {
                if max > 0 {
                    let pct = ((cur as f64 / max as f64) * 100.0).round();
                    percent = Some(pct.clamp(0.0, 100.0) as u8);
                    break;
                }
            }
        }

        (source, percent)
    }
}

#[cfg(target_os = "macos")]
fn read_energy_mode(source: Source) -> EnergyMode {
    use macstate_sys::cf::{dict_get_dict, dict_get_i32, CFOwned};
    use macstate_sys::iokit::{
        kIOPMLowPowerModeKey, kIOPSACPowerValue, kIOPSBatteryPowerValue,
        IOPMCopyActivePMPreferences,
    };

    unsafe {
        let prefs = match CFOwned::from_create(IOPMCopyActivePMPreferences()) {
            Some(p) => p,
            None => return EnergyMode::Unknown,
        };
        let key = match source {
            Source::Ac => kIOPSACPowerValue,
            Source::Battery => kIOPSBatteryPowerValue,
        };
        let sub = dict_get_dict(prefs.as_ptr(), key);
        if sub.is_null() {
            return EnergyMode::Unknown;
        }
        // Despite the key being called `LowPowerMode`, the value is the
        // unified `pmset powermode` indicator: 0=automatic, 1=low, 2=high.
        // The sibling `HighPowerMode` key is unused on current macOS.
        match dict_get_i32(sub, kIOPMLowPowerModeKey) {
            Some(0) => EnergyMode::Automatic,
            Some(1) => EnergyMode::Low,
            Some(2) => EnergyMode::High,
            _ => EnergyMode::Unknown,
        }
    }
}
