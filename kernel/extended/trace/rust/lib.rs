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

extern "C" {
    fn LOS_TraceStart() -> u32;
    fn LOS_TraceStop();
    fn LOS_TraceEventMaskSet(mask: u32);
    fn LOS_TraceRecordDump(to_client: Bool);
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
