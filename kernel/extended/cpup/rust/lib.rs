#![no_std]

use core::ffi::c_char;
use kernel_rust::log::{printk, printk_cstr};

const CPUP_LAST_TEN_SECONDS: u16 = 0;
const CPUP_LAST_ONE_SECONDS: u16 = 1;
const CPUP_ALL_TIME: u16 = 0xffff;
const LOS_OK: u32 = 0;
const SYS_CPU_10S: &[u8] = b"\nSysCpuUsage in 10s: ";
const SYS_CPU_1S: &[u8] = b"\nSysCpuUsage in 1s: ";
const SYS_CPU_ALL: &[u8] = b"\nSysCpuUsage in all time: ";
const PID_CPU_10S: &[u8] = b" CpuUsage in 10s: ";
const PID_CPU_1S: &[u8] = b" CpuUsage in 1s: ";
const PID_CPU_ALL: &[u8] = b" CpuUsage in all time: ";

extern "C" {
    fn LOS_HistorySysCpuUsage(mode: u16) -> u32;
    fn LOS_HistoryProcessCpuUsage(pid: u32, mode: u16) -> u32;
    fn CpupIsProcessValid(pid: u32) -> i32;
    fn CpupGetPrecisionMult() -> u32;
}

fn cstr_len(ptr: *const c_char) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    len
}

fn cstr_eq(ptr: *const c_char, bytes: &[u8]) -> bool {
    if ptr.is_null() {
        return false;
    }
    let mut idx = 0usize;
    unsafe {
        while idx < bytes.len() {
            if *ptr.add(idx) as u8 != bytes[idx] {
                return false;
            }
            idx += 1;
        }
        *ptr.add(idx) == 0
    }
}

fn parse_u32_base0(ptr: *const c_char) -> Result<u32, ()> {
    if ptr.is_null() {
        return Err(());
    }
    let len = cstr_len(ptr);
    if len == 0 {
        return Err(());
    }

    let mut base: u32 = 10;
    let mut start = 0usize;
    unsafe {
        let first = *ptr;
        if first as u8 == b'0' {
            if len >= 2 {
                let second = *ptr.add(1);
                if second as u8 == b'x' || second as u8 == b'X' {
                    base = 16;
                    start = 2;
                    if start >= len {
                        return Err(());
                    }
                } else {
                    base = 8;
                }
            } else {
                return Ok(0);
            }
        }
    }

    let mut value: u64 = 0;
    let mut idx = start;
    unsafe {
        while idx < len {
            let ch = *ptr.add(idx) as u8;
            let digit = match ch {
                b'0'..=b'9' => (ch - b'0') as u32,
                b'a'..=b'f' => (10 + (ch - b'a')) as u32,
                b'A'..=b'F' => (10 + (ch - b'A')) as u32,
                _ => return Err(()),
            };
            if digit >= base {
                return Err(());
            }
            value = value * base as u64 + digit as u64;
            if value > u32::MAX as u64 {
                return Err(());
            }
            idx += 1;
        }
    }
    Ok(value as u32)
}

fn write_bytes(buf: &mut [u8], mut idx: usize, bytes: &[u8]) -> usize {
    for b in bytes {
        if idx >= buf.len() {
            return idx;
        }
        buf[idx] = *b;
        idx += 1;
    }
    idx
}

fn write_char(buf: &mut [u8], idx: usize, ch: u8) -> usize {
    if idx >= buf.len() {
        return idx;
    }
    buf[idx] = ch;
    idx + 1
}

fn write_u32(buf: &mut [u8], mut idx: usize, mut value: u32) -> usize {
    if idx >= buf.len() {
        return idx;
    }
    if value == 0 {
        return write_char(buf, idx, b'0');
    }
    let mut tmp = [0u8; 10];
    let mut len = 0usize;
    while value > 0 && len < tmp.len() {
        tmp[len] = b'0' + (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    while len > 0 {
        len -= 1;
        idx = write_char(buf, idx, tmp[len]);
    }
    idx
}

fn print_help() {
    printk(
        b"usage:\n\
      cpup\n\
      cpup [MODE]\n\
      cpup [MODE] [PID] \n\
\r\nMode parameter description:\n\
  0       SysCpuUsage in 10s\n\
  1       SysCpuUsage in 1s\n\
  others  SysCpuUsage in all time\n\0",
    );
}

fn print_unknown(prefix: &[u8], value: *const c_char) {
    printk(prefix);
    unsafe { printk_cstr(value) };
    printk(b"\n\0");
}

fn print_unknown_pid(pid: u32) {
    let mut buf = [0u8; 64];
    let mut idx = 0usize;
    idx = write_bytes(&mut buf, idx, b"\nUnknown pid: ");
    idx = write_u32(&mut buf, idx, pid);
    idx = write_char(&mut buf, idx, b'\n');
    if idx < buf.len() {
        buf[idx] = 0;
    }
    unsafe { printk_cstr(buf.as_ptr() as *const c_char) };
}

fn print_sys_usage(mode: u16) {
    let prefix: &[u8] = match mode {
        CPUP_LAST_TEN_SECONDS => SYS_CPU_10S,
        CPUP_LAST_ONE_SECONDS => SYS_CPU_1S,
        _ => SYS_CPU_ALL,
    };
    let usage = unsafe { LOS_HistorySysCpuUsage(mode) };
    let precision = unsafe { CpupGetPrecisionMult() };
    let mut buf = [0u8; 96];
    let mut idx = 0usize;
    idx = write_bytes(&mut buf, idx, prefix);
    idx = write_u32(&mut buf, idx, usage / precision);
    idx = write_char(&mut buf, idx, b'.');
    idx = write_u32(&mut buf, idx, usage % precision);
    idx = write_char(&mut buf, idx, b'\n');
    if idx < buf.len() {
        buf[idx] = 0;
    }
    unsafe { printk_cstr(buf.as_ptr() as *const c_char) };
}

fn print_pid_usage(mode: u16, pid: u32) {
    let suffix: &[u8] = match mode {
        CPUP_LAST_TEN_SECONDS => PID_CPU_10S,
        CPUP_LAST_ONE_SECONDS => PID_CPU_1S,
        _ => PID_CPU_ALL,
    };
    let usage = unsafe { LOS_HistoryProcessCpuUsage(pid, mode) };
    let precision = unsafe { CpupGetPrecisionMult() };
    let mut buf = [0u8; 128];
    let mut idx = 0usize;
    idx = write_bytes(&mut buf, idx, b"\npid ");
    idx = write_u32(&mut buf, idx, pid);
    idx = write_bytes(&mut buf, idx, suffix);
    idx = write_u32(&mut buf, idx, usage / precision);
    idx = write_char(&mut buf, idx, b'.');
    idx = write_u32(&mut buf, idx, usage % precision);
    idx = write_char(&mut buf, idx, b'\n');
    if idx < buf.len() {
        buf[idx] = 0;
    }
    unsafe { printk_cstr(buf.as_ptr() as *const c_char) };
}

#[no_mangle]
pub extern "C" fn OsShellCmdCpupRust(argc: i32, argv: *const *const c_char) -> u32 {
    if argc <= 0 {
        print_sys_usage(CPUP_LAST_TEN_SECONDS);
        return LOS_OK;
    }
    if argv.is_null() {
        return LOS_OK;
    }

    let arg0 = unsafe { *argv };
    if cstr_eq(arg0, b"-h") || cstr_eq(arg0, b"--help") {
        print_help();
        return LOS_OK;
    }

    let mut mode = match parse_u32_base0(arg0) {
        Ok(v) => v,
        Err(_) => {
            print_unknown(b"\nUnknown mode: \0", arg0);
            print_help();
            return LOS_OK;
        }
    };

    if mode > CPUP_ALL_TIME as u32 {
        mode = CPUP_ALL_TIME as u32;
    }

    if argc == 1 {
        print_sys_usage(mode as u16);
        return LOS_OK;
    }

    let arg1 = unsafe { *argv.add(1) };
    let pid = match parse_u32_base0(arg1) {
        Ok(v) => v,
        Err(_) => {
            print_unknown(b"\nUnknown pid: \0", arg1);
            return LOS_OK;
        }
    };

    if unsafe { CpupIsProcessValid(pid) } == 0 {
        print_unknown_pid(pid);
        return LOS_OK;
    }

    if argc == 2 {
        print_pid_usage(mode as u16, pid);
        return LOS_OK;
    }

    print_help();
    LOS_OK
}
