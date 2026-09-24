use core::ffi::{CStr, c_char, c_int};

use crate::Errno;

unsafe extern "C" {
    // Scarlet's C runtime keeps errno in the native thread's TLS header.
    fn __errno_location() -> *mut c_int;
    fn strerror(error: c_int) -> *const c_char;
}

pub const STRERROR_NAME: &str = "strerror";

pub fn with_description<F, T>(err: Errno, callback: F) -> T
where
    F: FnOnce(Result<&str, Errno>) -> T,
{
    // SAFETY: Scarlet libc returns a static NUL-terminated string for every
    // error code, including unknown codes.
    let ptr = unsafe { strerror(err.0) };
    if ptr.is_null() {
        return callback(Err(err));
    }
    // SAFETY: See the guarantee on strerror above.
    let description = unsafe { CStr::from_ptr(ptr) };
    callback(description.to_str().map_err(|_| err))
}

pub fn errno() -> Errno {
    // SAFETY: The CRT initializes this thread's errno slot before Rust code.
    unsafe { Errno(*__errno_location()) }
}

pub fn set_errno(Errno(value): Errno) {
    // SAFETY: The returned slot belongs to the current thread.
    unsafe { *__errno_location() = value };
}
