//! Synchronous one-shot snapshot of `NWPathMonitor` (Network.framework, 10.14+).
//!
//! `NWPathMonitor` is the modern replacement for `SCNetworkReachability`. The
//! framework is push-based — you register an update handler and it fires on a
//! dispatch queue. For a one-shot CLI we want a single synchronous read, so we
//! wrap the start / wait-on-semaphore / cancel sequence here.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use block2::{Block, RcBlock};

use crate::dispatch::{
    dispatch_semaphore_signal, dispatch_semaphore_wait, new_queue, new_semaphore, time_after,
};

#[link(name = "Network", kind = "framework")]
extern "C" {
    fn nw_path_monitor_create() -> *mut c_void;
    fn nw_path_monitor_set_queue(monitor: *mut c_void, queue: *mut c_void);
    fn nw_path_monitor_set_update_handler(
        monitor: *mut c_void,
        handler: *mut Block<dyn Fn(*mut c_void)>,
    );
    fn nw_path_monitor_start(monitor: *mut c_void);
    fn nw_path_monitor_cancel(monitor: *mut c_void);

    fn nw_path_get_status(path: *mut c_void) -> i32;
    fn nw_path_is_expensive(path: *mut c_void) -> bool;
    fn nw_path_is_constrained(path: *mut c_void) -> bool;
    fn nw_path_uses_interface_type(path: *mut c_void, interface_type: i32) -> bool;

    fn nw_release(obj: *mut c_void);
}

/// `nw_path_status_t`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStatus {
    Invalid = 0,
    Satisfied = 1,
    Unsatisfied = 2,
    SatisfiedWithDifferentInterface = 3,
}

/// `nw_interface_type_t`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Other = 0,
    Wifi = 1,
    Cellular = 2,
    WiredEthernet = 3,
    Loopback = 4,
}

#[derive(Debug, Clone)]
pub struct PathSnapshot {
    pub status: PathStatus,
    pub expensive: bool,
    pub constrained: bool,
    /// First matching interface type the path uses. Probed in priority
    /// wifi → wired → cellular → loopback → other.
    pub interface: Option<InterfaceType>,
}

/// Take a single synchronous snapshot of the current network path.
///
/// Returns `None` if the monitor never fires within `timeout`.
pub fn snapshot(timeout: Duration) -> Option<PathSnapshot> {
    let result: Arc<Mutex<Option<PathSnapshot>>> = Arc::new(Mutex::new(None));
    let fired = Arc::new(AtomicBool::new(false));
    let sem = new_semaphore()?;
    let queue = new_queue(c"dev.macstate.nwpath")?;

    let result_cl = result.clone();
    let fired_cl = fired.clone();
    let sem_ptr_usize = sem.as_ptr() as usize;

    let handler: RcBlock<dyn Fn(*mut c_void)> = RcBlock::new(move |path: *mut c_void| {
        if path.is_null() {
            return;
        }
        if fired_cl.swap(true, Ordering::SeqCst) {
            return;
        }
        let snap = unsafe { read_path(path) };
        if let Ok(mut slot) = result_cl.lock() {
            *slot = Some(snap);
        }
        unsafe {
            dispatch_semaphore_signal(sem_ptr_usize as *mut c_void);
        }
    });

    unsafe {
        let monitor = nw_path_monitor_create();
        if monitor.is_null() {
            return None;
        }
        nw_path_monitor_set_queue(monitor, queue.as_ptr());
        nw_path_monitor_set_update_handler(
            monitor,
            &*handler as *const Block<dyn Fn(*mut c_void)> as *mut _,
        );
        nw_path_monitor_start(monitor);

        let timeout_ns = timeout.as_nanos().min(i64::MAX as u128) as i64;
        let deadline = time_after(timeout_ns);
        let _ = dispatch_semaphore_wait(sem.as_ptr(), deadline);

        nw_path_monitor_cancel(monitor);
        nw_release(monitor);
    }

    result.lock().ok().and_then(|g| g.clone())
}

unsafe fn read_path(path: *mut c_void) -> PathSnapshot {
    let status = match nw_path_get_status(path) {
        0 => PathStatus::Invalid,
        1 => PathStatus::Satisfied,
        2 => PathStatus::Unsatisfied,
        3 => PathStatus::SatisfiedWithDifferentInterface,
        _ => PathStatus::Invalid,
    };
    let expensive = nw_path_is_expensive(path);
    let constrained = nw_path_is_constrained(path);

    let interface = [
        InterfaceType::Wifi,
        InterfaceType::WiredEthernet,
        InterfaceType::Cellular,
        InterfaceType::Loopback,
        InterfaceType::Other,
    ]
    .into_iter()
    .find(|t| nw_path_uses_interface_type(path, *t as i32));

    PathSnapshot {
        status,
        expensive,
        constrained,
        interface,
    }
}
