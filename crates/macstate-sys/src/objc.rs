//! Tiny Objective-C bridge: just enough to call the few NSProcessInfo
//! methods we need. Built on top of objc2 (which we already pull in via
//! block2), so no extra dependencies.

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};

/// `[NSProcessInfo processInfo].isLowPowerModeEnabled`
///
/// Returns `false` if the class lookup fails (shouldn't happen on macOS).
pub fn is_low_power_mode_enabled() -> bool {
    let Some(cls) = AnyClass::get(c"NSProcessInfo") else {
        return false;
    };
    unsafe {
        // +processInfo returns a shared autoreleased instance.
        let info: Option<Retained<AnyObject>> = msg_send![cls, processInfo];
        let Some(info) = info else { return false };
        let enabled: bool = msg_send![&*info, isLowPowerModeEnabled];
        enabled
    }
}
