//! IOKit power-source FFI.

use std::ffi::CStr;

use crate::cf::{CFArrayRef, CFDictionaryRef, CFStringRef, CFTypeRef};

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    pub fn IOPSCopyPowerSourcesInfo() -> CFTypeRef;
    pub fn IOPSCopyPowerSourcesList(blob: CFTypeRef) -> CFArrayRef;
    pub fn IOPSGetPowerSourceDescription(blob: CFTypeRef, ps: CFTypeRef) -> CFDictionaryRef;
    pub fn IOPSGetProvidingPowerSourceType(snapshot: CFTypeRef) -> CFStringRef;
}

// String key constants used in power-source dictionaries. These are stable
// across macOS versions and documented in <IOKit/ps/IOPSKeys.h>.
pub const kIOPSCurrentCapacityKey: &CStr = c"Current Capacity";
pub const kIOPSMaxCapacityKey: &CStr = c"Max Capacity";

// Provider type values returned by IOPSGetProvidingPowerSourceType.
pub const kIOPSACPowerValue: &str = "AC Power";
pub const kIOPSBatteryPowerValue: &str = "Battery Power";
