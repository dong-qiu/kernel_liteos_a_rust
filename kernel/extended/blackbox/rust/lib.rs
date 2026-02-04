#![no_std]

use core::ffi::c_void;

const LOG_FLAG: &[u8; 7] = b"GOODLOG";

#[repr(C)]
struct ErrorInfo {
    event: [u8; 32],
    module: [u8; 32],
    error_desc: [u8; 512],
}

#[repr(C)]
struct FaultLogInfo {
    flag: [u8; 8],
    len: i32,
    info: ErrorInfo,
}

#[no_mangle]
pub extern "C" fn BlackboxGetLastLogInfoRust(log_buf: *const c_void, info: *mut c_void) -> i32 {
    if log_buf.is_null() || info.is_null() {
        return -1;
    }

    let log = log_buf as *const FaultLogInfo;
    let flag = unsafe { &(*log).flag };
    let mut i = 0usize;
    while i < LOG_FLAG.len() {
        if flag[i] != LOG_FLAG[i] {
            return -1;
        }
        i += 1;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            &(*log).info as *const ErrorInfo,
            info as *mut ErrorInfo,
            1,
        );
    }
    0
}
