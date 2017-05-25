#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! # memalloc
//!
//! Memory allocation in stable rust, using generics and the implementation of
//! Vec to handle the actual memory management including alignment.
//!
//! This library do not allow for handling allocation failure, and will simply
//! abort the process on OOM. Unfortunately, this limitation is unavoidable if we want
//! to use only stable APIs.

use std::mem;

/// Returns a pointer to `size` number of elements of type T, or in other
/// words `size` * mem::size_of::<T>() bytes. Memory is aligned by the
/// implementation of Vec<T>.
///
/// On failure, aborts the process.
///
/// Behavior is undefined if the requested size is 0.
#[inline]
pub unsafe fn typed_alloc<T>(size: usize) -> *mut T {
    ptr_from_vec(Vec::with_capacity(size))
}

/// Resizes the allocation referenced by `ptr` to `new_size` number of elements
/// of type T.
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
pub unsafe fn typed_realloc<T>(ptr: *mut T, old_size: usize, new_size: usize) -> *mut T {
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
pub unsafe fn typed_dealloc<T>(ptr: *mut T, old_size: usize) {
    Vec::from_raw_parts(ptr, 0, old_size);
}

// Investigate later, is this safe? Looks like creating a pointer to address
// 1, or returning a pointer to the stack?
// Original below:
//
// A token empty allocation which cannot be read from or written to,
// but which can be used as a placeholder when a 0-sized allocation is
// required.
//pub fn empty() -> *mut u8 {
//  1 as *mut u8
//}

#[inline]
fn ptr_from_vec<T>(mut buf: Vec<T>) -> *mut T {
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);

    ptr
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use {typed_alloc, typed_realloc, typed_dealloc};

    /* Suspicious, TODO: Investigate later.
    //use empty;
    #[test]
    fn test_empty() {
        let ptr: *mut u8 = empty();
        assert!(ptr != ptr::null_mut());
    }
    */

    #[test]
    fn test_allocate() {
        let buffer = unsafe { typed_alloc(8) };

        assert!(buffer != ptr::null_mut());

        unsafe {
            ptr::write(buffer.offset(0), 8);
            ptr::write(buffer.offset(1), 4);
            ptr::write(buffer.offset(3), 5);
            ptr::write(buffer.offset(5), 3);
            ptr::write(buffer.offset(7), 6);

            assert_eq!(ptr::read(buffer.offset(0)), 8);
            assert_eq!(ptr::read(buffer.offset(1)), 4);
            assert_eq!(ptr::read(buffer.offset(3)), 5);
            assert_eq!(ptr::read(buffer.offset(5)), 3);
            assert_eq!(ptr::read(buffer.offset(7)), 6);
        };

        unsafe {
            typed_dealloc(buffer, 8);
        }

        // Try a large buffer
        let buffer = unsafe { typed_alloc(1024 * 1024) };
        assert!(buffer != ptr::null_mut());

        unsafe {
            ptr::write(buffer.offset(1024 * 1024 - 1), 12);
            assert_eq!(ptr::read(buffer.offset(1024 * 1024 - 1)), 12);
        };

        unsafe {
            typed_dealloc(buffer, 1024 * 1024);
        }
    }

    #[test]
    fn test_reallocate() {
        let mut buffer = unsafe { typed_alloc(8) };
        assert!(buffer != ptr::null_mut());

        buffer = unsafe { typed_realloc(buffer, 8, 16) };
        assert!(buffer != ptr::null_mut());

        unsafe {
            // Put some data in the buffer
            ptr::write(buffer.offset(0), 8);
            ptr::write(buffer.offset(1), 4);
            ptr::write(buffer.offset(5), 3);
            ptr::write(buffer.offset(7), 6);
        };

        // Allocate so in-place fails.
        unsafe {
            let _: *mut u8 = typed_alloc(128);
        };

        buffer = unsafe { typed_realloc(buffer, 16, 32) };
        assert!(buffer != ptr::null_mut());

        unsafe {
            // Ensure the data is still there.
            assert_eq!(ptr::read(buffer.offset(0)), 8);
            assert_eq!(ptr::read(buffer.offset(1)), 4);
            assert_eq!(ptr::read(buffer.offset(5)), 3);
            assert_eq!(ptr::read(buffer.offset(7)), 6);
        };
    }
}
