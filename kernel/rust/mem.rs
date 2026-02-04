use core::ffi::c_void;

extern "C" {
    fn KernelMemAlloc(size: u32) -> *mut c_void;
    fn KernelMemFree(ptr: *mut c_void);
}

/// # Safety
/// Caller must ensure `size` is appropriate for the kernel heap and that the
/// returned pointer is checked for null before use.
pub unsafe fn mem_alloc(size: u32) -> *mut c_void {
    KernelMemAlloc(size)
}

/// # Safety
/// Caller must ensure the pointer was allocated by `mem_alloc` and is valid.
pub unsafe fn mem_free(ptr: *mut c_void) {
    KernelMemFree(ptr);
}
