// A jump table on arm64e dispatches through the hardened sequence, as Apple's LLVM emits it for
// every arm64e function: the index is clamped again in x16, and the table address, the entry and
// the target stay in x16 and x17 up to the `br`.
//@ add-minicore
//@ assembly-output: emit-asm
//@ revisions: O2 O0 IOS VISIONOS
//@ [O2] compile-flags: --target arm64e-apple-darwin -Copt-level=2
//@ [O2] needs-llvm-components: aarch64
//@ [O0] compile-flags: --target arm64e-apple-darwin -Copt-level=0
//@ [O0] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=2
//@ [IOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos -Copt-level=2
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

unsafe extern "C" {
    safe fn f0();
    safe fn f1();
    safe fn f2();
    safe fn f3();
    safe fn f4();
    safe fn f5();
    safe fn f6();
    safe fn f7();
    safe fn f8();
    safe fn f9();
    safe fn f10();
    safe fn f11();
}

// CHECK-LABEL: {{_?}}dispatch:
// CHECK: cmp x16, #11
// CHECK-NEXT: csel x16, x16, xzr, ls
// CHECK-NEXT: adrp x17, [[TABLE:.*]]@PAGE
// CHECK-NEXT: add x17, x17, [[TABLE]]@PAGEOFF
// CHECK-NEXT: ldrsw x16, [x17, x16, lsl #2]
// CHECK-NEXT: [[ANCHOR:.*]]:
// CHECK-NEXT: adr x17, [[ANCHOR]]
// CHECK-NEXT: add x16, x17, x16
// CHECK-NEXT: br x16
#[unsafe(no_mangle)]
pub fn dispatch(x: u32) {
    match x {
        0 => f0(),
        1 => f1(),
        2 => f2(),
        3 => f3(),
        4 => f4(),
        5 => f5(),
        6 => f6(),
        7 => f7(),
        8 => f8(),
        9 => f9(),
        10 => f10(),
        11 => f11(),
        _ => {}
    }
}
