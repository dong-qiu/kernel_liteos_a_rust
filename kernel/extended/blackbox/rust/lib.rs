#![no_std]

use core::ffi::{c_char, c_void};

extern "C" {
    fn BlackboxAccess(path: *const c_char) -> i32;
    fn BlackboxMkdir(path: *const c_char) -> i32;
}

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

#[no_mangle]
pub extern "C" fn BlackboxIsLogPartReadyRust(current_ready: i32) -> i32 {
    if current_ready != 0 {
        1
    } else {
        0
    }
}

/// # Safety
/// Caller must provide a valid, null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn BlackboxCreateNewDirRust(dir_path: *const c_char) -> i32 {
    if dir_path.is_null() {
        return -1;
    }
    let exists = BlackboxAccess(dir_path);
    if exists == 0 {
        return 0;
    }
    let ret = BlackboxMkdir(dir_path);
    if ret != 0 {
        return -1;
    }
    0
}

/// # Safety
/// Caller must provide a valid, null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn BlackboxCreateLogDirRust(dir_path: *const c_char) -> i32 {
    if dir_path.is_null() {
        return -1;
    }

    let mut idx: usize = 0;
    let mut cur = [0u8; 256];
    let mut p = dir_path;

    unsafe {
        if *p != b'/' as c_char {
            return -1;
        }

        cur[idx] = b'/';
        idx += 1;
        p = p.add(1);

        while idx < cur.len() {
            let ch = *p;
            if ch == 0 {
                break;
            }
            if ch == b'/' as c_char {
                cur[idx] = 0;
                if BlackboxCreateNewDirRust(cur.as_ptr() as *const c_char) != 0 {
                    return -1;
                }
            }
            cur[idx] = ch;
            idx += 1;
            p = p.add(1);
        }
    }

    if idx >= cur.len() {
        return -1;
    }

    cur[idx] = 0;
    if BlackboxCreateNewDirRust(cur.as_ptr() as *const c_char) != 0 {
        return -1;
    }

    0
}
