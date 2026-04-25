//@ add-minicore
//@ revisions: DARWIN IOS TVOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos
//@ [TVOS] needs-llvm-components: aarch64
#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;
use minicore::*;

#[no_mangle]
pub unsafe fn inline_asm() {
    unsafe { asm!("nop") };
}

// CHECK-LABEL: define{{.*}} @inline_asm(
// CHECK: call void asm sideeffect alignstack "nop"
// CHECK-NOT: [ "ptrauth"(i32 0, i64 0) ]
// CHECK: ret void
