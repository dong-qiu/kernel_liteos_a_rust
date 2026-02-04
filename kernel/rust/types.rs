pub type U8 = u8;
pub type U16 = u16;
pub type U32 = u32;
pub type I32 = i32;
pub type Void = core::ffi::c_void;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Errno(pub U32);

pub type Result<T> = core::result::Result<T, Errno>;

impl Errno {
    pub const OK: Errno = Errno(0);

    pub const fn from_u32(code: U32) -> Errno {
        Errno(code)
    }

    pub const fn from_i32(code: I32) -> Errno {
        Errno(code as U32)
    }

    pub const fn code(self) -> U32 {
        self.0
    }

    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }
}

pub fn result_from_u32<T>(code: U32, ok: T) -> Result<T> {
    if code == 0 {
        Ok(ok)
    } else {
        Err(Errno(code))
    }
}
