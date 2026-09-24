use crate::FileTime;
use std::fs::{self, File, FileTimes};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn to_system_time(value: FileTime) -> io::Result<SystemTime> {
    let seconds = value.unix_seconds();
    let nanos = value.nanoseconds();
    let time = if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::new(seconds as u64, nanos))
    } else if nanos == 0 {
        UNIX_EPOCH.checked_sub(Duration::new(seconds.unsigned_abs(), 0))
    } else {
        UNIX_EPOCH.checked_sub(Duration::new(seconds.unsigned_abs() - 1, 1_000_000_000 - nanos))
    };
    time.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    // Scarlet's metadata record always carries a modification timestamp.
    FileTime::from_system_time(meta.modified().expect("Scarlet file has no modification time"))
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    FileTime::from_system_time(meta.accessed().expect("Scarlet file has no access time"))
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    meta.created().ok().map(FileTime::from_system_time)
}

pub fn set_file_handle_times(
    file: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    let mut times = FileTimes::new();
    if let Some(atime) = atime {
        times = times.set_accessed(to_system_time(atime)?);
    }
    if let Some(mtime) = mtime {
        times = times.set_modified(to_system_time(mtime)?);
    }
    file.set_times(times)
}

pub fn set_file_times(path: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    let file = File::open(path)?;
    set_file_handle_times(&file, Some(atime), Some(mtime))
}

pub fn set_file_mtime(path: &Path, mtime: FileTime) -> io::Result<()> {
    let file = File::open(path)?;
    set_file_handle_times(&file, None, Some(mtime))
}

pub fn set_file_atime(path: &Path, atime: FileTime) -> io::Result<()> {
    let file = File::open(path)?;
    set_file_handle_times(&file, Some(atime), None)
}

pub fn set_symlink_file_times(_path: &Path, _atime: FileTime, _mtime: FileTime) -> io::Result<()> {
    // std has no public no-follow timestamp setter on Scarlet yet.
    Err(io::ErrorKind::Unsupported.into())
}
