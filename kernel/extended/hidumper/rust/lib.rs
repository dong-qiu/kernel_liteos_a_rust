#![no_std]

use core::ffi::{c_char, c_int};
use core::panic::PanicInfo;

extern "C" {
    fn HidumperPrintk(msg: *const c_char);
    fn HidumperShellCmdUname() -> c_int;
    fn HidumperShellCmdSystemInfo() -> c_int;
    fn HidumperShellCmdFree() -> c_int;
    fn HidumperShellCmdDumpPmm() -> c_int;
    fn HidumperShellCmdTaskInfo() -> c_int;
    fn HidumperInjectKernelCrash() -> c_int;
}

const SYS_INFO_HEADER: &[u8] = b"\n************ sys info ***********\n\0";
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
