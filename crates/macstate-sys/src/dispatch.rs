//! Minimal libdispatch FFI (queue + semaphore) used by NWPathMonitor wrapper.

use std::ffi::{c_char, c_void};
use std::ptr;

pub type DispatchQueueT = *mut c_void;
pub type DispatchSemaphoreT = *mut c_void;
pub type DispatchObjectT = *mut c_void;

pub type DispatchTimeT = u64;
pub const DISPATCH_TIME_NOW: DispatchTimeT = 0;
pub const DISPATCH_TIME_FOREVER: DispatchTimeT = u64::MAX;

#[link(name = "System", kind = "framework")]
extern "C" {
    pub fn dispatch_queue_create(label: *const c_char, attr: *const c_void) -> DispatchQueueT;
    pub fn dispatch_release(object: DispatchObjectT);

    pub fn dispatch_semaphore_create(value: isize) -> DispatchSemaphoreT;
    pub fn dispatch_semaphore_wait(dsema: DispatchSemaphoreT, timeout: DispatchTimeT) -> isize;
    pub fn dispatch_semaphore_signal(dsema: DispatchSemaphoreT) -> isize;

    pub fn dispatch_time(when: DispatchTimeT, delta: i64) -> DispatchTimeT;
}

/// Owning wrapper that releases the dispatch object on drop.
pub struct DispatchOwned(DispatchObjectT);

impl DispatchOwned {
    /// SAFETY: caller must ensure `obj` is a +1 dispatch object (returned by a
    /// `*_create` function) or null.
    pub unsafe fn from_create(obj: DispatchObjectT) -> Option<Self> {
        if obj.is_null() {
            None
        } else {
            Some(Self(obj))
        }
    }

    pub fn as_ptr(&self) -> DispatchObjectT {
        self.0
    }
}

impl Drop for DispatchOwned {
    fn drop(&mut self) {
        unsafe { dispatch_release(self.0) }
    }
}

/// Build an absolute `dispatch_time_t` `delta_ns` from now.
pub fn time_after(delta_ns: i64) -> DispatchTimeT {
    unsafe { dispatch_time(DISPATCH_TIME_NOW, delta_ns) }
}

/// Allocate a new private serial queue.
pub fn new_queue(label: &std::ffi::CStr) -> Option<DispatchOwned> {
    unsafe { DispatchOwned::from_create(dispatch_queue_create(label.as_ptr(), ptr::null())) }
}

/// Allocate a new semaphore initialised at 0 (i.e. `wait` blocks until first
/// `signal`).
pub fn new_semaphore() -> Option<DispatchOwned> {
    unsafe { DispatchOwned::from_create(dispatch_semaphore_create(0)) }
}
