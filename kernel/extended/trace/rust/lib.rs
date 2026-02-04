#![no_std]

use core::ffi::c_void;

type Bool = u32;

#[repr(C)]
struct TraceClientCmd {
    cmd: u8,
    param1: u8,
    param2: u8,
    param3: u8,
    param4: u8,
    param5: u8,
    end: u8,
}

const TRACE_CMD_END_CHAR: u8 = 0x0d;
const TRACE_CMD_START: u8 = 1;
const TRACE_CMD_STOP: u8 = 2;
const TRACE_CMD_SET_EVENT_MASK: u8 = 3;
const TRACE_CMD_RECODE_DUMP: u8 = 4;
const TRACE_CMD_MAX_CODE: u8 = 5;
const LOS_OK: u32 = 0;
const TRACE_UNINIT: u32 = 0;
const TRACE_STARTED: u32 = 2;
const TRACE_STOPED: u32 = 3;
const TRACE_EVENT_MASK: u32 = 0xFFFFFFF0;
const TRACE_ENABLE_TRUE: Bool = 1;
const TRACE_ENABLE_FALSE: Bool = 0;

extern "C" {
    fn LOS_TraceStart() -> u32;
    fn LOS_TraceStop();
    fn LOS_TraceEventMaskSet(mask: u32);
    fn LOS_TraceRecordDump(to_client: Bool);
    fn OsTraceDataWait() -> u32;
    fn OsTraceDataRecv(data: *mut u8, size: u32, timeout: u32) -> u32;
    fn TraceLockSaveRust() -> u32;
    fn TraceUnlockRestoreRust(state: u32);
    fn TraceGetStateRust() -> u32;
    fn TraceSetStateRust(state: u32);
    fn TraceSetEnableRust(enable: Bool);
    fn TraceSetMaskRust(mask: u32);
    fn TraceNotifyStartRust();
    fn TraceNotifyStopRust();
    fn TraceRecordDumpRust(to_client: Bool);
    fn TraceMemInfoReqRust();
    fn TraceLogNotInitedRust();
    fn TraceLogDumpStateRust(state: u32);
    fn TraceGetErrnoTraceErrorStatusRust() -> u32;
}

#[no_mangle]
pub extern "C" fn OsTraceCmdIsValidRust(msg: *const c_void) -> Bool {
    let msg = msg as *const TraceClientCmd;
    if msg.is_null() {
        return 0;
    }
    unsafe {
        if (*msg).end != TRACE_CMD_END_CHAR {
            return 0;
        }
        if (*msg).cmd >= TRACE_CMD_MAX_CODE {
            return 0;
        }
    }
    1
}

#[no_mangle]
pub extern "C" fn OsTraceCmdHandleRust(msg: *const c_void) {
    if OsTraceCmdIsValidRust(msg) == 0 {
        return;
    }
    let msg = msg as *const TraceClientCmd;
    let cmd = unsafe { (*msg).cmd };
    match cmd {
        TRACE_CMD_START => {
            unsafe { LOS_TraceStart() };
        }
        TRACE_CMD_STOP => {
            unsafe { LOS_TraceStop() };
        }
        TRACE_CMD_SET_EVENT_MASK => {
            let m = unsafe {
                let p1 = (*msg).param1 as u32;
                let p2 = (*msg).param2 as u32;
                let p3 = (*msg).param3 as u32;
                let p4 = (*msg).param4 as u32;
                (p1 << 24) | (p2 << 16) | (p3 << 8) | p4
            };
            unsafe { LOS_TraceEventMaskSet(m) };
        }
        TRACE_CMD_RECODE_DUMP => {
            unsafe { LOS_TraceRecordDump(1) };
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn TraceAgentRust() {
    loop {
        let mut msg = TraceClientCmd {
            cmd: 0,
            param1: 0,
            param2: 0,
            param3: 0,
            param4: 0,
            param5: 0,
            end: 0,
        };
        let ret = unsafe { OsTraceDataWait() };
        if ret == LOS_OK {
            unsafe {
                let _ = OsTraceDataRecv(
                    &mut msg as *mut TraceClientCmd as *mut u8,
                    core::mem::size_of::<TraceClientCmd>() as u32,
                    0,
                );
            }
            OsTraceCmdHandleRust(&msg as *const TraceClientCmd as *const c_void);
        }
    }
}

#[no_mangle]
pub extern "C" fn LOS_TraceStartRust() -> u32 {
    let int_save = unsafe { TraceLockSaveRust() };
    let state = unsafe { TraceGetStateRust() };
    if state == TRACE_STARTED {
        unsafe { TraceUnlockRestoreRust(int_save) };
        return LOS_OK;
    }
    if state == TRACE_UNINIT {
        unsafe {
            TraceLogNotInitedRust();
            TraceUnlockRestoreRust(int_save);
            return TraceGetErrnoTraceErrorStatusRust();
        }
    }

    unsafe {
        TraceNotifyStartRust();
        TraceSetEnableRust(TRACE_ENABLE_TRUE);
        TraceSetStateRust(TRACE_STARTED);
        TraceUnlockRestoreRust(int_save);
        TraceMemInfoReqRust();
    }
    LOS_OK
}

#[no_mangle]
pub extern "C" fn LOS_TraceStopRust() {
    let int_save = unsafe { TraceLockSaveRust() };
    if unsafe { TraceGetStateRust() } != TRACE_STARTED {
        unsafe { TraceUnlockRestoreRust(int_save) };
        return;
    }

    unsafe {
        TraceSetEnableRust(TRACE_ENABLE_FALSE);
        TraceSetStateRust(TRACE_STOPED);
        TraceNotifyStopRust();
        TraceUnlockRestoreRust(int_save);
    }
}

#[no_mangle]
pub extern "C" fn LOS_TraceEventMaskSetRust(mask: u32) {
    unsafe { TraceSetMaskRust(mask & TRACE_EVENT_MASK) };
}

#[no_mangle]
pub extern "C" fn LOS_TraceRecordDumpRust(to_client: Bool) {
    let state = unsafe { TraceGetStateRust() };
    if state != TRACE_STOPED {
        unsafe { TraceLogDumpStateRust(state) };
        return;
    }
    unsafe { TraceRecordDumpRust(to_client) };
}
