// Indirect calls authenticate with `blraaz` or `braaz`; direct calls do not.
// Disabling `v8.3a` does not disable `paca` and `pacg`, which it implies: the backend keeps `pauth`
// and still emits these instructions.
//@ add-minicore
//@ assembly-output: emit-asm
//@ revisions: DEFAULT NO_V83A
//@ [DEFAULT] compile-flags: --target arm64e-apple-darwin -Copt-level=2
//@ [DEFAULT] needs-llvm-components: aarch64
//@ [NO_V83A] compile-flags: --target arm64e-apple-darwin -Copt-level=2 -Ctarget-feature=-v8.3a
//@ [NO_V83A] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn target(x: i32) -> i32 {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn direct(x: i32) -> i32 {
    target(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn indirect_tail(f: extern "C" fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

#[unsafe(no_mangle)]
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
