//@ add-minicore
//@ assembly-output: emit-asm
//@ compile-flags: --target arm64e-apple-darwin -Copt-level=2
//@ needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

#[inline(never)]
#[no_mangle]
pub extern "C" fn target(x: i32) -> i32 {
    x
}

#[no_mangle]
pub extern "C" fn direct(x: i32) -> i32 {
    target(x)
}

#[no_mangle]
pub extern "C" fn indirect_tail(f: extern "C" fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

#[no_mangle]
pub extern "C" fn indirect_non_tail(f: extern "C" fn(i32) -> i32, x: i32) -> i32 {
    match f(x) {
        0 => target(x),
        y => y,
    }
}

// CHECK-LABEL: _direct:
// CHECK-NOT: braaz
// CHECK-NOT: blraaz

// CHECK-LABEL: _indirect_tail:
// CHECK: braaz

// CHECK-LABEL: _indirect_non_tail:
// CHECK: blraaz
