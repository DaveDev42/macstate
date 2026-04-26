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

#[cfg(target_os = "macos")]
impl Power {
    pub fn collect() -> Self {
        let (source, battery_percent) = read_power_source();
        let energy_mode = read_energy_mode();
        let low_power_mode = matches!(energy_mode, EnergyMode::Low);
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
            energy_mode: EnergyMode::Automatic,
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

        // CFArrayRef from IOPSCopyPowerSourcesList is +1, but the list itself
        // borrows entries from `snapshot`, so we own and release the array.
        let list_raw = IOPSCopyPowerSourcesList(snapshot.as_ptr());
        let list = match CFOwned::from_create(list_raw) {
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
fn read_energy_mode() -> EnergyMode {
    // `pmset -g` exposes:
    //   - `lowpowermode 1` (Intel + Apple Silicon, Low Power Mode)
    //   - `highpowermode 1` (Apple Silicon Pro/Max, High Power Mode)
    //   - `powermode N` (unified indicator on newer systems: 0=auto, 1=low, 2=high)
    let Ok(out) = std::process::Command::new("pmset").arg("-g").output() else {
        return EnergyMode::Automatic;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut low = false;
    let mut high = false;
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("lowpowermode") {
            if first_field(rest) == Some("1") {
                low = true;
            }
        } else if let Some(rest) = l.strip_prefix("highpowermode") {
            if first_field(rest) == Some("1") {
                high = true;
            }
        } else if let Some(rest) = l.strip_prefix("powermode") {
            match first_field(rest) {
                Some("1") => low = true,
                Some("2") => high = true,
                _ => {}
            }
        }
    }
    match (low, high) {
        (true, _) => EnergyMode::Low,
        (_, true) => EnergyMode::High,
        _ => EnergyMode::Automatic,
    }
}

#[cfg(target_os = "macos")]
fn first_field(s: &str) -> Option<&str> {
    s.split_whitespace().next()
}
