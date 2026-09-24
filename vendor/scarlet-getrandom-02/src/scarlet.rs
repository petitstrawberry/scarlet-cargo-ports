use core::mem::MaybeUninit;

use crate::Error;
use scarlet_sys::{GET_RANDOM_FLAG_REQUIRE_ENTROPY, Syscall};

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let mut filled = 0;
    while filled < dest.len() {
        // SAFETY: The slice is exclusively borrowed for the duration of the
        // syscall. The kernel writes at most the remaining length to this range.
        let result = unsafe {
            scarlet_sys::syscall3(
                Syscall::GetRandom,
                dest[filled..].as_mut_ptr() as usize,
                dest.len() - filled,
                GET_RANDOM_FLAG_REQUIRE_ENTROPY,
            )
        };
        if result == usize::MAX || result == 0 || result > dest.len() - filled {
            return Err(Error::UNEXPECTED);
        }
        filled += result;
    }
    Ok(())
}
