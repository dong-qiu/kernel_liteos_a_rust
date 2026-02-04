use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::mem::{mem_alloc, mem_free};

pub struct KernelBox<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> KernelBox<T> {
    /// # Safety
    /// Caller must ensure the kernel heap is initialized and allocation is allowed.
    pub unsafe fn new(value: T) -> Option<Self> {
        let size = core::mem::size_of::<T>() as u32;
        let raw = mem_alloc(size) as *mut T;
        if raw.is_null() {
            return None;
        }
        raw.write(value);
        NonNull::new(raw).map(|ptr| KernelBox { ptr, _marker: PhantomData })
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> core::ops::Deref for KernelBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> core::ops::DerefMut for KernelBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for KernelBox<T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.ptr.as_ptr());
            mem_free(self.ptr.as_ptr() as *mut c_void);
        }
    }
}
