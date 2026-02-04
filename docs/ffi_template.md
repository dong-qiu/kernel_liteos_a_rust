# Rust FFI Template (LiteOS-A)

## Goals
1. Keep existing C ABI stable.
2. Move implementation into Rust behind a thin C adapter layer.
3. Minimize unsafe to FFI boundaries only.

## Recommended Layout
1. C adapter: `<module>/<module>_rust_ffi.c`
2. Rust impl: `<module>/rust/lib.rs`
3. Optional shared Rust: `kernel/rust/*`

## C Adapter Guidelines
1. Wrap `#ifdef` config gates in C and expose stable FFI functions.
2. Validate pointers and sizes before passing into Rust.
3. Provide small helpers for kernel-only APIs (e.g. printk, alloc).

## Rust Guidelines
1. `#![no_std]` only at crate root.
2. Use `extern "C"` with `#[no_mangle]` for exported symbols.
3. Prefer small safe wrappers; keep `unsafe` at FFI boundaries.
4. Document `# Safety` for unsafe functions.

## Minimal Example

C adapter:
```c
void KernelPrintk(const char *msg) {
    if (msg != NULL) {
        PRINTK("%s", msg);
    }
}
```

Rust:
```rust
extern "C" { fn KernelPrintk(msg: *const c_char); }

#[no_mangle]
pub extern "C" fn MyFunc() {
    unsafe { KernelPrintk(b"hello\0".as_ptr() as *const c_char); }
}
```
