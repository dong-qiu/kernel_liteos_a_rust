#![no_std]

use core::ffi::c_char;

extern "C" {
    fn HidumperPrintk(msg: *const c_char);
}

pub fn printk(msg: &[u8]) {
    unsafe {
        HidumperPrintk(msg.as_ptr() as *const c_char);
    }
}
