use core::ffi::c_char;

extern "C" {
    fn KernelPrintk(msg: *const c_char);
}

pub fn printk(msg: &[u8]) {
    unsafe {
        KernelPrintk(msg.as_ptr() as *const c_char);
    }
}

pub unsafe fn printk_cstr(msg: *const c_char) {
    if !msg.is_null() {
        KernelPrintk(msg);
    }
}
