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
const TRACE_HWI_FLAG: u32 = 0x20;
const TRACE_TASK_FLAG: u32 = 0x40;
const TRACE_FRAME_BUF_MAX: usize = 512;

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
    fn TracePipelineInitRust() -> u32;
    fn TraceCreateAgentTaskRust() -> u32;
    fn TraceDeleteAgentTaskRust();
    fn TraceBufInitRust() -> u32;
    fn TraceHookInstallRust();
    fn TraceCnvInitRust();
    fn TraceResetEventCountRust();
    fn TraceSetInitialStateRust();
    fn TraceLogInitAlreadyRust(state: u32);
    fn TraceLogCreateAgentFailRust(ret: u32);
    fn TraceIsEnabledRust() -> Bool;
    fn TraceGetMaskRust() -> u32;
    fn TraceGetModeFlagRust(event_type: u32) -> u32;
    fn TraceIsTaskCreateOrPrioSetRust(event_type: u32) -> Bool;
    fn TraceIsMemInfoReqRust(event_type: u32) -> Bool;
    fn TraceHwiFilterRust(hwi_num: u32) -> Bool;
    fn TraceHandleMemInfoReqRust(identity: usize);
    fn TraceGetCurTaskIdRust() -> u32;
    fn TraceGetCurPidRust() -> u32;
    fn TraceGetCyclesRust() -> u64;
    fn TraceGetMaskTidRust(task_id: u32) -> u32;
    fn TraceObjAddRust(event_type: u32, identity: usize);
    fn TraceFrameSizeRust() -> u32;
    fn TraceFrameMaxParamsRust() -> u16;
    fn TraceFrameClearRust(frame: *mut c_void);
    fn TraceFrameSetBasicRust(
        frame: *mut c_void,
        event_type: u32,
        cur_task: u32,
        cur_pid: u32,
        identity: usize,
        cur_time: u64,
    );
    fn TraceFrameSetCoreRust(frame: *mut c_void, param_count: u16);
    fn TraceFrameSetEventCountRust(frame: *mut c_void);
    fn TraceFrameRecordLRRust(frame: *mut c_void);
    fn TraceFrameSetParamsRust(frame: *mut c_void, params: *const usize, param_count: u16);
    fn OsTraceWriteOrSendEvent(frame: *const c_void);
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

#[no_mangle]
pub extern "C" fn OsTraceInitRust() -> u32 {
    let state = unsafe { TraceGetStateRust() };
    if state != TRACE_UNINIT {
        unsafe { TraceLogInitAlreadyRust(state) };
        return unsafe { TraceGetErrnoTraceErrorStatusRust() };
    }

    let mut ret = unsafe { TracePipelineInitRust() };
    if ret != LOS_OK {
        return ret;
    }

    ret = unsafe { TraceCreateAgentTaskRust() };
    if ret != LOS_OK {
        unsafe { TraceLogCreateAgentFailRust(ret) };
        return ret;
    }

    ret = unsafe { TraceBufInitRust() };
    if ret != LOS_OK {
        unsafe { TraceDeleteAgentTaskRust() };
        return ret;
    }

    unsafe {
        TraceHookInstallRust();
        TraceCnvInitRust();
        TraceResetEventCountRust();
        TraceSetInitialStateRust();
    }

    LOS_OK
}

#[repr(C, align(8))]
struct TraceFrameBuf {
    buf: [u8; TRACE_FRAME_BUF_MAX],
}

/// # Safety
/// Caller must provide a valid `params` pointer when `param_count > 0`.
#[no_mangle]
pub unsafe extern "C" fn OsTraceHookRust(
    event_type: u32,
    identity: usize,
    params: *const usize,
    param_count: u16,
) {
    if unsafe { TraceIsTaskCreateOrPrioSetRust(event_type) } != 0 {
        unsafe { TraceObjAddRust(event_type, identity) };
    }

    if unsafe { TraceIsEnabledRust() } == 0 {
        return;
    }
    if (event_type & unsafe { TraceGetMaskRust() }) == 0 {
        return;
    }

    let mut id = identity;
    let mode = unsafe { TraceGetModeFlagRust(event_type) };
    if mode == TRACE_HWI_FLAG {
        if unsafe { TraceHwiFilterRust(identity as u32) } != 0 {
            return;
        }
    } else if mode == TRACE_TASK_FLAG {
        id = unsafe { TraceGetMaskTidRust(identity as u32) } as usize;
    } else if unsafe { TraceIsMemInfoReqRust(event_type) } != 0 {
        unsafe { TraceHandleMemInfoReqRust(identity) };
        return;
    }

    let frame_size = unsafe { TraceFrameSizeRust() } as usize;
    if frame_size > TRACE_FRAME_BUF_MAX {
        return;
    }

    let mut frame = TraceFrameBuf {
        buf: [0u8; TRACE_FRAME_BUF_MAX],
    };
    let frame_ptr = frame.buf.as_mut_ptr() as *mut c_void;

    unsafe { TraceFrameClearRust(frame_ptr) };
    let max_params = unsafe { TraceFrameMaxParamsRust() };
    let use_params = if param_count > max_params {
        max_params
    } else {
        param_count
    };

    let int_save = unsafe { TraceLockSaveRust() };
    let cur_task = unsafe { TraceGetMaskTidRust(TraceGetCurTaskIdRust()) };
    let cur_pid = unsafe { TraceGetCurPidRust() };
    let cur_time = unsafe { TraceGetCyclesRust() };

    unsafe {
        TraceFrameSetBasicRust(frame_ptr, event_type, cur_task, cur_pid, id, cur_time);
        TraceFrameSetCoreRust(frame_ptr, use_params);
        TraceFrameSetEventCountRust(frame_ptr);
        TraceFrameRecordLRRust(frame_ptr);
        TraceUnlockRestoreRust(int_save);
    }

    if use_params > 0 && !params.is_null() {
        TraceFrameSetParamsRust(frame_ptr, params, use_params);
    }

    OsTraceWriteOrSendEvent(frame_ptr as *const c_void);
}
