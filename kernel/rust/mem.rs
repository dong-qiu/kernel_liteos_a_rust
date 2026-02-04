use core::ffi::c_void;

extern "C" {
    fn KernelMemAlloc(size: u32) -> *mut c_void;
    fn KernelMemFree(ptr: *mut c_void);
}

pub unsafe fn mem_alloc(size: u32) -> *mut c_void {
    KernelMemAlloc(size)
}

pub unsafe fn mem_free(ptr: *mut c_void) {
    KernelMemFree(ptr);
}
