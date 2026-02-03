#![no_std]

use core::ffi::{c_char, c_int};
use core::panic::PanicInfo;
use core::ptr;

extern "C" {
    fn HidumperPrintk(msg: *const c_char);
    fn HidumperShellCmdUname() -> c_int;
    fn HidumperShellCmdSystemInfo() -> c_int;
    fn HidumperShellCmdFree() -> c_int;
    fn HidumperShellCmdDumpPmm() -> c_int;
    fn HidumperShellCmdTaskInfo() -> c_int;
    fn HidumperInjectKernelCrash() -> c_int;
    fn HidumperGetProcessMaxNum() -> u32;
    fn HidumperGetProcessInfo(pid: u32, out: *mut HidumperProcessInfo) -> c_int;
    fn HidumperGetAllProcessCpuUsage(mode: u16, cpup: *mut CpupInfo, len: u32) -> u32;
    fn HidumperMemAlloc(size: u32) -> *mut core::ffi::c_void;
    fn HidumperMemFree(ptr: *mut core::ffi::c_void);
    fn HidumperPrintCpuUsageHeader();
    fn HidumperPrintCpuUsageLine(name: *const c_char, pid: u32, all: u32, ten: u32, one: u32);
}

#[repr(C)]
struct CpupInfo {
    status: u16,
    usage: u32,
}

#[repr(C)]
struct HidumperProcessInfo {
    name: *const c_char,
    pid: u32,
    unused: u8,
    _pad: [u8; 3],
}

const SYS_INFO_HEADER: &[u8] = b"\n************ sys info ***********\n\0";
const CPU_USAGE_HEADER: &[u8] = b"\n************ cpu usage ***********\n\0";
const MEM_USAGE_HEADER: &[u8] = b"\n************ mem usage ***********\n\0";
const PAGE_USAGE_HEADER: &[u8] = b"************ physical page usage ***********\n\0";
const TASK_INFO_HEADER: &[u8] = b"\n************ task info ***********\n\0";
const UNIT_KB: &[u8] = b"Unit: KB\n\0";
const UNSUPPORTED: &[u8] = b"\nUnsupported!\n\0";
const MEMDATA_UNSUPPORTED: &[u8] = b"Unsupported now!\n\0";
const PANIC_MSG: &[u8] = b"\nHiDumper rust panic\n\0";

#[inline(always)]
fn printk(msg: &[u8]) {
    unsafe {
        HidumperPrintk(msg.as_ptr() as *const c_char);
    }
}

#[no_mangle]
pub extern "C" fn HiDumperDumpCpuUsageRust() {
    const CPUP_ALL_TIME: u16 = 0xffff;
    const CPUP_LAST_TEN_SECONDS: u16 = 0;
    const CPUP_LAST_ONE_SECONDS: u16 = 1;
    const CPUP_TYPE_COUNT: u32 = 3;

    printk(CPU_USAGE_HEADER);
    let max = unsafe { HidumperGetProcessMaxNum() };
    if max == 0 {
        printk(UNSUPPORTED);
        return;
    }

    let total = max.saturating_mul(CPUP_TYPE_COUNT) as usize;
    let size = total
        .saturating_mul(core::mem::size_of::<CpupInfo>())
        .min(u32::MAX as usize) as u32;
    let base = unsafe { HidumperMemAlloc(size) } as *mut CpupInfo;
    if base.is_null() {
        printk(UNSUPPORTED);
        return;
    }

    unsafe {
        ptr::write_bytes(base, 0, total);
    }

    let all = base;
    let ten = unsafe { base.add(max as usize) };
    let one = unsafe { base.add((max as usize) * 2) };

    let len = (max as usize * core::mem::size_of::<CpupInfo>()) as u32;
    let ret_all = unsafe { HidumperGetAllProcessCpuUsage(CPUP_ALL_TIME, all, len) };
    let ret_ten = unsafe { HidumperGetAllProcessCpuUsage(CPUP_LAST_TEN_SECONDS, ten, len) };
    let ret_one = unsafe { HidumperGetAllProcessCpuUsage(CPUP_LAST_ONE_SECONDS, one, len) };
    if ret_all != 0 || ret_ten != 0 || ret_one != 0 {
        unsafe { HidumperMemFree(base as *mut core::ffi::c_void) };
        printk(UNSUPPORTED);
        return;
    }

    unsafe { HidumperPrintCpuUsageHeader() };

    let mut pid: u32 = 0;
    while pid < max {
        let mut info = HidumperProcessInfo {
            name: ptr::null(),
            pid: 0,
            unused: 1,
            _pad: [0; 3],
        };
        let ok = unsafe { HidumperGetProcessInfo(pid, &mut info as *mut HidumperProcessInfo) };
        if ok == 0 && info.unused == 0 && !info.name.is_null() {
            let all_u = unsafe { (*all.add(pid as usize)).usage };
            let ten_u = unsafe { (*ten.add(pid as usize)).usage };
            let one_u = unsafe { (*one.add(pid as usize)).usage };
            unsafe { HidumperPrintCpuUsageLine(info.name, info.pid, all_u, ten_u, one_u) };
        }
        pid += 1;
    }

    unsafe { HidumperMemFree(base as *mut core::ffi::c_void) };
}

#[no_mangle]
pub extern "C" fn HiDumperDumpSysInfoRust() {
    printk(SYS_INFO_HEADER);
    let ret1 = unsafe { HidumperShellCmdUname() };
    let ret2 = unsafe { HidumperShellCmdSystemInfo() };
    if ret1 != 0 || ret2 != 0 {
        printk(UNSUPPORTED);
    }
}

#[no_mangle]
pub extern "C" fn HiDumperDumpMemUsageRust() {
    printk(MEM_USAGE_HEADER);
    printk(UNIT_KB);
    if unsafe { HidumperShellCmdFree() } != 0 {
        printk(UNSUPPORTED);
        return;
    }
    printk(PAGE_USAGE_HEADER);
    if unsafe { HidumperShellCmdDumpPmm() } != 0 {
        printk(UNSUPPORTED);
    }
}

#[no_mangle]
pub extern "C" fn HiDumperDumpTaskInfoRust() {
    printk(TASK_INFO_HEADER);
    if unsafe { HidumperShellCmdTaskInfo() } != 0 {
        printk(UNSUPPORTED);
    }
}

#[no_mangle]
pub extern "C" fn HiDumperDumpMemDataRust(_param: *mut core::ffi::c_void) {
    printk(MEMDATA_UNSUPPORTED);
}

#[no_mangle]
pub extern "C" fn HiDumperInjectKernelCrashRust() {
    if unsafe { HidumperInjectKernelCrash() } != 0 {
        printk(UNSUPPORTED);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    printk(PANIC_MSG);
    loop {}
}
