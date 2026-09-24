//! Raw Scarlet C ABI declarations. Keep these layouts synchronized with
//! `Scarlet/user/lib/scarlet-libc/include` and its header ABI checks.

pub type size_t = usize;
pub type ssize_t = isize;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type time_t = i64;
pub type suseconds_t = i64;
pub type off_t = i64;
pub type pid_t = crate::c_int;
pub type uid_t = crate::c_uint;
pub type gid_t = crate::c_uint;
pub type mode_t = crate::c_uint;
pub type dev_t = u64;
pub type ino_t = u64;
pub type nlink_t = u64;
pub type socklen_t = crate::c_uint;
pub type sa_family_t = u16;

pub const SEEK_SET: crate::c_int = 0;
pub const SEEK_CUR: crate::c_int = 1;
pub const SEEK_END: crate::c_int = 2;
pub const EIO: crate::c_int = 5;
pub const EINVAL: crate::c_int = 22;
