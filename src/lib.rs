#![no_std]
#![feature(alloc, core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;
use alloc::{
    vec::Vec,
    format,
};
use core::{
    alloc::Layout,
    panic::PanicInfo,
};
use parity_codec::{Encode, Decode};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    unsafe { core::intrinsics::abort() }
}

#[alloc_error_handler]
pub extern "C" fn oom(_: Layout) -> ! {
    unsafe {
        core::intrinsics::abort();
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

mod sys {
    extern "C" {
        pub fn ext_set_storage(
            key_ptr: u32,
            value_non_null: u32,
            value_ptr: u32,
            value_len: u32,
        );
        pub fn ext_get_storage(key_ptr: u32) -> u32;
        pub fn ext_scratch_size() -> u32;
        pub fn ext_scratch_copy(dest_ptr: u32, offset: u32, len: u32);
        pub fn ext_println(str_ptr: u32, str_len: u32);
    }
}

fn srml_println(content: &str) {
    unsafe {
        sys::ext_println(content.as_ptr() as u32, content.len() as u32)
    }
}

type Key = [u8; 32];

unsafe fn srml_store(key: Key, value: &[u8]) {
    srml_println("<SrmlEnv as EnvStorage>::store");
    sys::ext_set_storage(
        key.as_ptr() as u32,
        1,
        value.as_ptr() as u32,
        value.len() as u32,
    );
}

unsafe fn srml_load(key: Key) -> Option<Vec<u8>> {
    srml_println("<SrmlEnv as EnvStorage>::load");
    const SUCCESS: u32 = 0;
    let result = sys::ext_get_storage(key.as_ptr() as u32);
    if result != SUCCESS {
        srml_println("<SrmlEnv as EnvStorage>::load FAIL");
        return None
    }
    srml_println("<SrmlEnv as EnvStorage>::load ok");
    let size = sys::ext_scratch_size();
    srml_println(&format!(
        "<SrmlEnv as EnvStorage>::load ext_scratch_size = {:?}",
        size
    ));
    let mut value = Vec::new();
    if size > 0 {
        value.resize(size as usize, 0);
        sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
    }
    Some(value)
}

const KEY: [u8; 32] = [0x1; 32];
const INIT_VALUE: u32 = 100;
const INCREASE_BY: u32 = 5;

#[no_mangle]
fn deploy() {
    srml_println("called `deploy` start");
    unsafe { srml_store(KEY, u32::encode(&INIT_VALUE).as_slice()) };
    srml_println("called `deploy` end");
}

#[no_mangle]
fn call() {
    srml_println("called `call` start");
    if let Some(bytes) = unsafe { srml_load(KEY) } {
        if let Some(mut val) = u32::decode(&mut bytes.as_slice()) {
            srml_println(&format!("called `call` val = {:?}", val));
            val += INCREASE_BY;
            srml_println("called `call` ok val += inc");
            unsafe { srml_store(KEY, u32::encode(&val).as_slice()) };
            srml_println("called `call` ok stored");
        } else {
            srml_println("called `call` FAIL -> error upon decoding `u32`");
        }
    } else {
        srml_println("called `call` FAIL -> `srml_load` returned `None`");
    }
    srml_println("called `call` end");
}
