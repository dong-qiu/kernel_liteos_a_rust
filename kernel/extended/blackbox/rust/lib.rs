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
    fn BlackboxInvokeModuleOpsForInfo(info: *const ErrorInfo) -> i32;
    fn BlackboxSysRebootAndLog() -> i32;
    fn BlackboxGetOpsList() -> *mut LosDlList;
    fn BlackboxOpsListSemPend() -> u32;
    fn BlackboxOpsListSemPost() -> u32;
    fn CreateLogDir(dir_path: *const c_char) -> i32;
    fn UploadEventByFile(file_path: *const c_char) -> i32;
    fn BlackboxLogErrSimple(msg: *const c_char);
    fn BlackboxLogErrModule(module: *const c_char, msg: *const c_char);
    fn BlackboxLogInfoModule(module: *const c_char, msg: *const c_char);
    fn BlackboxLogInfoModuleEvent(module: *const c_char, event: *const c_char, msg: *const c_char);
    fn BlackboxLogErrPathFailed(prefix: *const c_char, path: *const c_char);
}

const LOG_FLAG: &[u8; 7] = b"GOODLOG";
const ERROR_INFO_HEADER: &[u8] = b"#### error info ####\n";
const ERROR_INFO_EVENT: &[u8] = b"event: ";
const ERROR_INFO_MODULE: &[u8] = b"\nmodule: ";
const ERROR_INFO_DESC: &[u8] = b"\nerrorDesc: ";
const ERROR_INFO_TAIL: &[u8] = b"\n";
const LOG_ERR_SEM_PEND: &[u8] = b"Request g_opsListSem failed!\n\0";
const LOG_ERR_OPS_LIST_NULL: &[u8] = b"ops list is NULL!\n\0";
const LOG_ERR_OPS_MISSING: &[u8] = b"GetLastLogInfo or SaveLastLog is NULL!\n\0";
const LOG_ERR_GET_INFO: &[u8] = b"failed to get log info!\n\0";
const LOG_ERR_SAVE_LOG: &[u8] = b"failed to save log!\n\0";
const LOG_INFO_START_SAVE: &[u8] = b"starts saving log!\n\0";
const LOG_INFO_END_SAVE: &[u8] = b"ends saving log!\n\0";
const LOG_INFO_START_UPLOAD: &[u8] = b"starts uploading event\0";
const LOG_INFO_END_UPLOAD: &[u8] = b"ends uploading event\0";
const LOG_ERR_CREATE_LOG_DIR: &[u8] = b"Create log dir\0";

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

#[repr(C)]
struct LosDlList {
    pst_prev: *mut LosDlList,
    pst_next: *mut LosDlList,
}

#[repr(C)]
struct ModuleOps {
    module: [u8; 32],
    dump: Option<unsafe extern "C" fn(*const c_char, *mut ErrorInfo)>,
    reset: Option<unsafe extern "C" fn(*mut ErrorInfo)>,
    get_last_log_info: Option<unsafe extern "C" fn(*mut ErrorInfo) -> i32>,
    save_last_log: Option<unsafe extern "C" fn(*const c_char, *mut ErrorInfo) -> i32>,
}

#[repr(C)]
struct BBoxOps {
    ops_list: LosDlList,
    ops: ModuleOps,
}

fn log_err_simple(msg: &[u8]) {
    unsafe { BlackboxLogErrSimple(msg.as_ptr() as *const c_char) };
}

fn log_err_module(module: *const c_char, msg: &[u8]) {
    unsafe { BlackboxLogErrModule(module, msg.as_ptr() as *const c_char) };
}

fn log_info_module(module: *const c_char, msg: &[u8]) {
    unsafe { BlackboxLogInfoModule(module, msg.as_ptr() as *const c_char) };
}

fn log_info_module_event(module: *const c_char, event: *const c_char, msg: &[u8]) {
    unsafe { BlackboxLogInfoModuleEvent(module, event, msg.as_ptr() as *const c_char) };
}

fn log_err_path_failed(prefix: &[u8], path: *const c_char) {
    unsafe { BlackboxLogErrPathFailed(prefix.as_ptr() as *const c_char, path) };
}

#[no_mangle]
pub extern "C" fn BlackboxGetLastLogInfoRust(log_buf: *const c_void, info: *mut c_void) -> i32 {
    if log_buf.is_null() || info.is_null() {
        return -1;
    }

    let log_bytes = log_buf as *const u8;
    let mut flag = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(log_bytes, flag.as_mut_ptr(), flag.len());
    }
    for i in 0..LOG_FLAG.len() {
        if flag[i] != LOG_FLAG[i] {
            return -1;
        }
    }

    let info_offset = core::mem::size_of::<[u8; 8]>() + core::mem::size_of::<i32>();
    unsafe {
        core::ptr::copy_nonoverlapping(
            log_bytes.add(info_offset),
            info as *mut u8,
            size_of::<ErrorInfo>(),
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
        dst[i] = ch as u8;
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
    if ok {
        0
    } else {
        -1
    }
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
        log_err_path_failed(LOG_ERR_CREATE_LOG_DIR, log_dir);
        return -1;
    }
    BlackboxSaveBasicErrorInfoRust(file_path, info)
}

/// # Safety
/// Caller must provide a valid pointer to `ErrorInfo`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveLogWithResetRust(info: *const ErrorInfo) {
    if info.is_null() {
        return;
    }
    if BlackboxInvokeModuleOpsForInfo(info) != 0 {
        return;
    }
    let _ = BlackboxSysRebootAndLog();
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

    let log_bytes = log_buf as *const u8;
    let mut flag = [0u8; 8];
    unsafe {
        core::ptr::copy_nonoverlapping(log_bytes, flag.as_mut_ptr(), flag.len());
    }
    let mut ok = true;
    for i in 0..LOG_FLAG.len() {
        if flag[i] != LOG_FLAG[i] {
            ok = false;
            break;
        }
    }

    if ok {
        let len_ptr = unsafe { log_bytes.add(core::mem::size_of::<[u8; 8]>()) as *const i32 };
        let len = unsafe { core::ptr::read_unaligned(len_ptr) };
        let payload = log_size.saturating_sub(size_of::<FaultLogInfo>());
        let use_len = if len <= 0 {
            0
        } else {
            min(payload, len as usize)
        };
        let data_ptr = unsafe { log_bytes.add(size_of::<FaultLogInfo>()) };
        let _ = BlackboxSaveFaultLogRust(file_path, data_ptr, use_len, info);
    }

    unsafe {
        core::ptr::write_bytes(log_buf as *mut u8, 0, log_size);
    }
    0
}

/// # Safety
/// Caller must provide valid pointers for `log_dir` and `kernel_fault_path`.
#[no_mangle]
pub unsafe extern "C" fn BlackboxSaveLastLogCoreRust(
    log_dir: *const c_char,
    kernel_fault_path: *const c_char,
) -> i32 {
    if log_dir.is_null() || kernel_fault_path.is_null() {
        return -1;
    }
    if BlackboxOpsListSemPend() != 0 {
        log_err_simple(LOG_ERR_SEM_PEND);
        return -1;
    }
    if CreateLogDir(log_dir) != 0 {
        let _ = BlackboxOpsListSemPost();
        return -1;
    }

    let head = BlackboxGetOpsList();
    if head.is_null() {
        log_err_simple(LOG_ERR_OPS_LIST_NULL);
        let _ = BlackboxOpsListSemPost();
        return -1;
    }

    let mut node = unsafe { (*head).pst_next };
    while node != head {
        let ops = node as *mut BBoxOps;
        if !ops.is_null() {
            let module_ops = unsafe { &(*ops).ops };
            let module_ptr = module_ops.module.as_ptr() as *const c_char;
            if let (Some(get_info), Some(save_log)) =
                (module_ops.get_last_log_info, module_ops.save_last_log)
            {
                let mut info = ErrorInfo {
                    event: [0; 32],
                    module: [0; 32],
                    error_desc: [0; 512],
                };
                if unsafe { get_info(&mut info as *mut ErrorInfo) } != 0 {
                    log_err_module(module_ptr, LOG_ERR_GET_INFO);
                } else {
                    log_info_module(module_ptr, LOG_INFO_START_SAVE);
                    if unsafe { save_log(log_dir, &mut info as *mut ErrorInfo) } != 0 {
                        log_err_module(module_ptr, LOG_ERR_SAVE_LOG);
                    } else {
                        log_info_module(module_ptr, LOG_INFO_END_SAVE);
                        let event_ptr = info.event.as_ptr() as *const c_char;
                        log_info_module_event(module_ptr, event_ptr, LOG_INFO_START_UPLOAD);
                        let _ = unsafe { UploadEventByFile(kernel_fault_path) };
                        log_info_module_event(module_ptr, event_ptr, LOG_INFO_END_UPLOAD);
                    }
                }
            } else {
                log_err_module(module_ptr, LOG_ERR_OPS_MISSING);
            }
        }
        node = unsafe { (*node).pst_next };
    }

    let _ = BlackboxOpsListSemPost();
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
            cur[idx] = ch as u8;
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
