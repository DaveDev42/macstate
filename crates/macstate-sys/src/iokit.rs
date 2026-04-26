//! IOKit power-source FFI.

use std::ffi::CStr;

use crate::cf::{CFArrayRef, CFDictionaryRef, CFStringRef, CFTypeRef};

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    pub fn IOPSCopyPowerSourcesInfo() -> CFTypeRef;
    pub fn IOPSCopyPowerSourcesList(blob: CFTypeRef) -> CFArrayRef;
    pub fn IOPSGetPowerSourceDescription(blob: CFTypeRef, ps: CFTypeRef) -> CFDictionaryRef;
    pub fn IOPSGetProvidingPowerSourceType(snapshot: CFTypeRef) -> CFStringRef;

    /// Private but long-stable: returns a dict keyed by `"AC Power"` /
    /// `"Battery Power"`, each mapping to a sub-dict containing
    /// `LowPowerMode` (i32) and, on Apple Silicon Pro/Max, `HighPowerMode`
    /// (i32). This is the same source `pmset(8)` reads — there is no
    /// public C API for these flags.
    pub fn IOPMCopyActivePMPreferences() -> CFDictionaryRef;
}

// String key constants used in power-source dictionaries. These are stable
// across macOS versions and documented in <IOKit/ps/IOPSKeys.h>.
pub const kIOPSCurrentCapacityKey: &CStr = c"Current Capacity";
pub const kIOPSMaxCapacityKey: &CStr = c"Max Capacity";

// Provider type values returned by IOPSGetProvidingPowerSourceType.
// These also match the top-level keys of IOPMCopyActivePMPreferences.
pub const kIOPSACPowerValue: &str = "AC Power";
pub const kIOPSBatteryPowerValue: &str = "Battery Power";

// Sub-dict key inside IOPMCopyActivePMPreferences entries.
// Despite the name, the value is the unified `pmset powermode` indicator:
// 0 = automatic, 1 = low, 2 = high. The sibling `HighPowerMode` key
// exists in the dict but is unused on current macOS.
pub const kIOPMLowPowerModeKey: &CStr = c"LowPowerMode";
