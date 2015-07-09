#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! # memalloc
//!
//! Memory allocation in stable rust, providing a similar interface to `std::rt::heap`,
//! notably these functions align everything according to the alignment of `u8`, rather
//! than using a user-provided alignment.
//!
//! Additionally, they do not allow for handling allocation failure, and will simply
//! abort the process on OOM. Unfortunately, this limitation is unavoidable if we want
//! to use only stable APIs.
//!

use std::mem;

/// Returns a pointer to `size` bytes of memory aligned to `mem::align_of::<u8>()`.
///
/// On failure, aborts the process.
///
/// Behavior is undefined if the requested size is 0.
#[inline]
pub unsafe fn allocate(size: usize) -> *mut u8 {
    ptr_from_vec(Vec::with_capacity(size))
}

/// Resizes the allocation referenced by `ptr` to `new_size` bytes.
///
/// On failure, aborts the process.
///
/// If the allocation was relocated, the memory at the passed-in pointer is
/// undefined after the call.
///
/// Behavior is undefined if the requested `new_size` is 0.
///
/// The `old_size` parameter is the size used to create the allocation
/// referenced by `ptr`, or the `new_size` passed to previous reallocations.
pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if old_size > new_size {
        let mut buf = Vec::from_raw_parts(ptr, new_size, old_size);
        buf.shrink_to_fit();

        ptr_from_vec(buf)
    } else if new_size > old_size {
        let additional = new_size - old_size;

        let mut buf = Vec::from_raw_parts(ptr, 0, old_size);
        buf.reserve_exact(additional);

        ptr_from_vec(buf)
    } else {
        ptr
    }
}

/// Deallocates the memory referenced by `ptr`.
///
/// Behavior is undefined if `ptr` is null.
///
/// The `old_size` parameter is the size used to create the allocation
/// referenced by `ptr`, or the `new_size` passed to the last reallocation.
#[inline]
pub unsafe fn deallocate(ptr: *mut u8, old_size: usize) {
    Vec::from_raw_parts(ptr, 0, old_size);
}

/// A token empty allocation which cannot be read from or written to,
/// but which can be used as a placeholder when a 0-sized allocation is
/// required.
pub fn empty() -> *mut u8 {
    1 as *mut u8
}

#[inline]
fn ptr_from_vec(mut buf: Vec<u8>) -> *mut u8 {
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);

    ptr
}

