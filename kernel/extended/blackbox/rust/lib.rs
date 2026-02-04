#![no_std]

use core::cmp::min;
use core::ffi::{c_char, c_void};
use core::mem::size_of;

extern "C" {
    fn BlackboxAccess(path: *const c_char) -> i32;
    fn BlackboxMkdir(path: *const c_char) -> i32;
    fn BlackboxOpenForWrite(path: *const c_char, is_append: i32) -> i32;
    fn BlackboxWrite(fd: i32, buf: *const u8, len: usize) -> i32;
    fn BlackboxFsync(fd: i32) -> i32;
    fn BlackboxClose(fd: i32) -> i32;
    fn IsLogPartReady() -> bool;
}

const LOG_FLAG: &[u8; 7] = b"GOODLOG";
const ERROR_INFO_HEADER: &[u8] = b"#### error info ####\n";
const ERROR_INFO_EVENT: &[u8] = b"event: ";
const ERROR_INFO_MODULE: &[u8] = b"\nmodule: ";
const ERROR_INFO_DESC: &[u8] = b"\nerrorDesc: ";
const ERROR_INFO_TAIL: &[u8] = b"\n";

#[repr(C)]
pub struct ErrorInfo {
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

unsafe fn copy_cstr(dst: &mut [u8], src: *const c_char) {
    if dst.is_empty() {
        return;
    }
    let mut i = 0usize;
    while i + 1 < dst.len() {
        let ch = unsafe { *src.add(i) };
        if ch == 0 {
            break;
        }
        dst[i] = ch;
        i += 1;
    }
    dst[i] = 0;
}

fn c_buf_len(buf: &[u8]) -> usize {
    for (idx, ch) in buf.iter().enumerate() {
        if *ch == 0 {
            return idx;
        }
    }
    buf.len()
}

unsafe fn write_all(fd: i32, mut ptr: *const u8, mut len: usize) -> bool {
    while len > 0 {
        let written = BlackboxWrite(fd, ptr, len);
        if written <= 0 {
            return false;
        }
        let step = written as usize;
        len -= step;
        ptr = ptr.add(step);
    }
    true
}

fn write_all_slice(fd: i32, buf: &[u8]) -> bool {
    unsafe { write_all(fd, buf.as_ptr(), buf.len()) }
}

/// # Safety
/// Caller must provide valid pointers for `file_path` and `buf`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxFullWriteFileRust(
    file_path: *const c_char,
    buf: *const u8,
    buf_size: usize,
    is_append: i32,
) -> i32 {
    if file_path.is_null() || buf.is_null() || buf_size == 0 {
        return -1;
    }
    if !IsLogPartReady() {
        return -1;
    }

    let fd = BlackboxOpenForWrite(file_path, is_append);
    if fd < 0 {
        return -1;
    }

    let ok = write_all(fd, buf, buf_size);
    let _ = BlackboxFsync(fd);
    let _ = BlackboxClose(fd);
    if ok { 0 } else { -1 }
}

/// # Safety
/// Caller must provide a valid, null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveBasicErrorInfoRust(
    file_path: *const c_char,
    info: *const ErrorInfo,
) -> i32 {
    if file_path.is_null() || info.is_null() {
        return -1;
    }
    if !IsLogPartReady() {
        return 0;
    }

    let fd = BlackboxOpenForWrite(file_path, 0);
    if fd < 0 {
        return 0;
    }

    let info_ref = unsafe { &*info };
    let event_len = c_buf_len(&info_ref.event);
    let module_len = c_buf_len(&info_ref.module);
    let desc_len = c_buf_len(&info_ref.error_desc);

    let _ = write_all_slice(fd, ERROR_INFO_HEADER);
    let _ = write_all_slice(fd, ERROR_INFO_EVENT);
    let _ = write_all_slice(fd, &info_ref.event[..event_len]);
    let _ = write_all_slice(fd, ERROR_INFO_MODULE);
    let _ = write_all_slice(fd, &info_ref.module[..module_len]);
    let _ = write_all_slice(fd, ERROR_INFO_DESC);
    let _ = write_all_slice(fd, &info_ref.error_desc[..desc_len]);
    let _ = write_all_slice(fd, ERROR_INFO_TAIL);

    let _ = BlackboxFsync(fd);
    let _ = BlackboxClose(fd);
    0
}

/// # Safety
/// Caller must provide valid pointers for `file_path`, `data_buf`, and `info`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveFaultLogRust(
    file_path: *const c_char,
    data_buf: *const u8,
    buf_size: usize,
    info: *const ErrorInfo,
) -> i32 {
    if file_path.is_null() || info.is_null() {
        return -1;
    }

    let _ = BlackboxSaveBasicErrorInfoRust(file_path, info);
    if !data_buf.is_null() && buf_size > 0 {
        let _ = BlackboxFullWriteFileRust(file_path, data_buf, buf_size, 1);
    }
    0
}

/// # Safety
/// Caller must provide valid pointers for `info`, `log_dir`, and `file_path`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveLogWithoutResetRust(
    info: *const ErrorInfo,
    log_dir: *const c_char,
    file_path: *const c_char,
) -> i32 {
    if info.is_null() || log_dir.is_null() || file_path.is_null() {
        return -1;
    }
    if BlackboxCreateLogDirRust(log_dir) != 0 {
        return -1;
    }
    BlackboxSaveBasicErrorInfoRust(file_path, info)
}

/// # Safety
/// Caller must provide valid pointers for all parameters.
#[no_mangle]
pub unsafe extern "C" fn BlackboxFormatErrorInfoRust(
    info: *mut ErrorInfo,
    event: *const c_char,
    module: *const c_char,
    error_desc: *const c_char,
) {
    if info.is_null() || event.is_null() || module.is_null() || error_desc.is_null() {
        return;
    }

    unsafe {
        core::ptr::write_bytes(info as *mut u8, 0, size_of::<ErrorInfo>());
        let info_ref = &mut *info;
        copy_cstr(&mut info_ref.event, event);
        copy_cstr(&mut info_ref.module, module);
        copy_cstr(&mut info_ref.error_desc, error_desc);
    }
}

/// # Safety
/// Caller must provide valid pointers for `log_buf`, `info`, and `file_path`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveLastLogRust(
    log_buf: *mut c_void,
    log_size: usize,
    info: *const ErrorInfo,
    file_path: *const c_char,
) -> i32 {
    if log_buf.is_null() || info.is_null() || file_path.is_null() {
        return -1;
    }
    if log_size < size_of::<FaultLogInfo>() {
        return -1;
    }

    let log = log_buf as *const FaultLogInfo;
    let flag = unsafe { &(*log).flag };
    let mut ok = true;
    for i in 0..LOG_FLAG.len() {
        if flag[i] != LOG_FLAG[i] {
            ok = false;
            break;
        }
    }

    if ok {
        let len = unsafe { (*log).len };
        let payload = log_size.saturating_sub(size_of::<FaultLogInfo>());
        let use_len = if len <= 0 { 0 } else { min(payload, len as usize) };
        let data_ptr = unsafe { (log_buf as *const u8).add(size_of::<FaultLogInfo>()) };
        let _ = BlackboxSaveFaultLogRust(file_path, data_ptr, use_len, info);
    }

    unsafe {
        core::ptr::write_bytes(log_buf as *mut u8, 0, log_size);
    }
    0
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
