use std::{mem, slice, str};

#[link(wasm_import_module = "my-host")]
extern "C" {
    #[link_name = "log"]
    fn wasm_my_host_log(ptr: u32, len: u32);
}

mod my_host {
    pub fn log(msg: &str) {
        unsafe { crate::wasm_my_host_log(msg.as_ptr() as u32, msg.len() as u32) }
    }
}

pub struct FfiString {
    ptr: u32,
    len: u32,
}

impl From<FfiString> for u64 {
    fn from(value: FfiString) -> Self {
        (value.ptr as u64) << 32 | (value.len as u64)
    }
}

impl From<u64> for FfiString {
    fn from(value: u64) -> Self {
        FfiString {
            ptr: (value >> 32) as u32,
            len: (value & 0xFFFFFFFFu64) as u32,
        }
    }
}

#[export_name = "alloc_string"]
pub extern "C" fn wasm_export_alloc_string(len: u32) -> u32 {
    let mut s = Vec::<u8>::new();
    s.reserve_exact(len as usize);
    let ptr = s.as_ptr() as u32;
    mem::forget(s);
    ptr
}

#[export_name = "exchange_strings"]
pub extern "C" fn wasm_exchange_strings(s: u64) -> u64 {
    let s: FfiString = s.into();
    let s = unsafe { slice::from_raw_parts(s.ptr as usize as *const _, s.len as usize) };
    let s = match str::from_utf8(s) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let s = my_module::exchange_strings(s);

    let out = FfiString {
        ptr: s.as_ptr() as u32,
        len: s.len() as u32,
    }
    .into();
    mem::forget(s);
    out
}

mod my_module {
    pub fn exchange_strings(s: &str) -> String {
        crate::my_host::log(s);
        s.to_owned()
    }
}
