//! Small, checked wrappers over Scarlet Native filesystem operations that
//! Rust's public `std` API does not expose yet.

#[cfg(target_os = "scarlet")]
use std::{
    ffi::CString, fs::File, fs::TryLockError, io, os::fd::AsRawFd, path::Path, thread,
    time::Duration,
};

#[cfg(target_os = "scarlet")]
use scarlet_abi::RawFileMetadata;

#[cfg(target_os = "scarlet")]
fn c_path(path: &Path) -> io::Result<CString> {
    let value = path.to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Scarlet paths require UTF-8")
    })?;
    CString::new(value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains a NUL byte"))
}

/// Query the Native VFS for a file's type, permission flags and stable ID.
#[cfg(target_os = "scarlet")]
pub fn metadata(path: &Path, no_follow: bool) -> io::Result<RawFileMetadata> {
    let path = c_path(path)?;
    let mut metadata = RawFileMetadata::default();
    // SAFETY: The C path is NUL-terminated, and the exclusively borrowed
    // metadata output lives until the synchronous syscall returns.
    let result = unsafe {
        scarlet_sys::syscall3(
            scarlet_sys::Syscall::VfsMetadataWithStatus,
            path.as_ptr() as usize,
            (&raw mut metadata) as usize,
            usize::from(no_follow),
        )
    } as isize;
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            result
                .checked_neg()
                .and_then(|code| i32::try_from(code).ok())
                .unwrap_or(scarlet_abi::ERRNO_EIO),
        ));
    }
    Ok(metadata)
}

/// Create a symbolic link. The target may be relative or missing.
#[cfg(target_os = "scarlet")]
pub fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    let link_path = link;
    let target = c_path(target)?;
    let link = c_path(link)?;
    if metadata(link_path, true).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "link path already exists",
        ));
    }
    // SAFETY: Both C paths are valid and live until the synchronous syscall
    // returns. Scarlet Native VfsCreateSymlink takes (link, target).
    let result = unsafe {
        scarlet_sys::syscall2(
            scarlet_sys::Syscall::VfsCreateSymlink,
            link.as_ptr() as usize,
            target.as_ptr() as usize,
        )
    };
    if result == usize::MAX {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Scarlet VFS symlink creation failed",
        ))
    } else {
        Ok(())
    }
}

/// Return whether a regular file has Scarlet's execute permission bit.
#[cfg(target_os = "scarlet")]
pub fn is_executable(path: &Path) -> bool {
    metadata(path, false).is_ok_and(|meta| {
        meta.file_type == scarlet_abi::FILE_TYPE_REGULAR
            && meta.permissions & scarlet_abi::FILE_PERMISSION_EXECUTE != 0
    })
}

/// Acquire a Native whole-file lock using the open description owned by `file`.
/// The kernel releases it when the last copy of that description closes.
#[cfg(target_os = "scarlet")]
fn file_lock(file: &File, operation: usize) -> io::Result<()> {
    // SAFETY: The live `File` owns the Native handle and FileLock consumes
    // only scalar arguments. The borrowed handle cannot close during this call.
    let result = unsafe {
        scarlet_sys::syscall2(
            scarlet_sys::Syscall::FileLock,
            file.as_raw_fd() as usize,
            operation,
        )
    } as isize;
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            result
                .checked_neg()
                .and_then(|code| i32::try_from(code).ok())
                .unwrap_or(scarlet_abi::ERRNO_EIO),
        ));
    }
    Ok(())
}

#[cfg(target_os = "scarlet")]
pub fn try_file_lock(file: &File, shared: bool) -> Result<(), TryLockError> {
    let operation = if shared { 1 } else { 2 };
    match file_lock(file, operation | 4) {
        Ok(()) => Ok(()),
        Err(error) if error.raw_os_error() == Some(scarlet_abi::ERRNO_EAGAIN) => {
            Err(TryLockError::WouldBlock)
        }
        Err(error) => Err(TryLockError::Error(error)),
    }
}

#[cfg(target_os = "scarlet")]
pub fn lock_file(file: &File, shared: bool) -> io::Result<()> {
    loop {
        match try_file_lock(file, shared) {
            Ok(()) => return Ok(()),
            Err(TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(10)),
            Err(TryLockError::Error(error)) => return Err(error),
        }
    }
}

#[cfg(target_os = "scarlet")]
pub fn unlock_file(file: &File) -> io::Result<()> {
    file_lock(file, 8)
}
