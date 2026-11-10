#![crate_type = "lib"]
#![feature(lang_items, no_core)]
#![no_core]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

#[lang = "copy"]
pub trait Copy: Sized {}

// External, so that LLVM cannot remove the call once it is direct.
extern "C" {
    fn target(x: i32) -> i32;
}

#[inline(always)]
fn call_indirect(f: unsafe extern "C" fn(i32) -> i32, x: i32) -> i32 {
    unsafe { f(x) }
}

#[no_mangle]
pub extern "C" fn devirtualized(x: i32) -> i32 {
    call_indirect(target, x)
}
