//! Minimal CoreFoundation FFI: only what we need to read IOPS dictionaries.

use std::ffi::{c_char, c_void, CStr};
use std::os::raw::c_long;

pub type CFTypeRef = *const c_void;
pub type CFAllocatorRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFNumberRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFIndex = c_long;
pub type CFNumberType = CFIndex;

pub const kCFNumberSInt32Type: CFNumberType = 3;

// CFStringEncoding
pub const kCFStringEncodingUTF8: u32 = 0x08000100;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub fn CFRelease(cf: CFTypeRef);

    pub fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> *const c_void;

    pub fn CFDictionaryGetValue(
        theDict: CFDictionaryRef,
        key: *const c_void,
    ) -> *const c_void;

    pub fn CFNumberGetValue(
        number: CFNumberRef,
        theType: CFNumberType,
        valuePtr: *mut c_void,
    ) -> bool;

    pub fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cStr: *const c_char,
        encoding: u32,
    ) -> CFStringRef;

    pub fn CFStringGetCStringPtr(s: CFStringRef, encoding: u32) -> *const c_char;
    pub fn CFStringGetCString(
        s: CFStringRef,
        buffer: *mut c_char,
        buffer_size: CFIndex,
        encoding: u32,
    ) -> bool;
    pub fn CFStringGetLength(s: CFStringRef) -> CFIndex;
}

/// Owning wrapper that calls `CFRelease` on drop.
pub struct CFOwned(CFTypeRef);

impl CFOwned {
    /// SAFETY: caller must ensure `ptr` came from a Create/Copy CF function
    /// (i.e. a +1 retain count we now own), or null.
    pub unsafe fn from_create(ptr: CFTypeRef) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> CFTypeRef {
        self.0
    }
}

impl Drop for CFOwned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

/// Build an autoreleased-equivalent CFString from a Rust `&CStr`.
/// Returns an owning wrapper because `CFStringCreateWithCString` returns +1.
pub fn cfstring_from_cstr(s: &CStr) -> Option<CFOwned> {
    unsafe {
        let raw = CFStringCreateWithCString(std::ptr::null(), s.as_ptr(), kCFStringEncodingUTF8);
        CFOwned::from_create(raw as CFTypeRef)
    }
}

/// Read a CFNumber out of a dict by &CStr key. Returns None if missing or
/// not a number.
pub unsafe fn dict_get_i32(dict: CFDictionaryRef, key: &CStr) -> Option<i32> {
    let key_cf = cfstring_from_cstr(key)?;
    let raw = CFDictionaryGetValue(dict, key_cf.as_ptr());
    if raw.is_null() {
        return None;
    }
    let mut out: i32 = 0;
    let ok = CFNumberGetValue(
        raw as CFNumberRef,
        kCFNumberSInt32Type,
        &mut out as *mut i32 as *mut c_void,
    );
    if ok {
        Some(out)
    } else {
        None
    }
}

/// Read a CFString out of a dict by &CStr key.
pub unsafe fn dict_get_string(dict: CFDictionaryRef, key: &CStr) -> Option<String> {
    let key_cf = cfstring_from_cstr(key)?;
    let raw = CFDictionaryGetValue(dict, key_cf.as_ptr());
    if raw.is_null() {
        return None;
    }
    cfstring_to_string(raw as CFStringRef)
}

/// Borrow a sub-dictionary from `dict` by `&str` key. The returned
/// pointer is non-owning (lifetime tied to `dict`).
pub unsafe fn dict_get_dict(dict: CFDictionaryRef, key: &str) -> CFDictionaryRef {
    let Ok(c) = std::ffi::CString::new(key) else {
        return std::ptr::null();
    };
    let Some(key_cf) = cfstring_from_cstr(&c) else {
        return std::ptr::null();
    };
    CFDictionaryGetValue(dict, key_cf.as_ptr()) as CFDictionaryRef
}

pub unsafe fn cfstring_to_string(s: CFStringRef) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let fast = CFStringGetCStringPtr(s, kCFStringEncodingUTF8);
    if !fast.is_null() {
        return Some(CStr::from_ptr(fast).to_string_lossy().into_owned());
    }
    // Fallback: allocate.
    let len = CFStringGetLength(s);
    // Worst-case UTF-8 expansion: 4 bytes per UTF-16 unit, +1 for NUL.
    let cap = (len * 4 + 1) as usize;
    let mut buf = vec![0u8; cap];
    let ok = CFStringGetCString(
        s,
        buf.as_mut_ptr() as *mut c_char,
        cap as CFIndex,
        kCFStringEncodingUTF8,
    );
    if !ok {
        return None;
    }
    let cstr = CStr::from_ptr(buf.as_ptr() as *const c_char);
    Some(cstr.to_string_lossy().into_owned())
}
