//@ add-minicore
//@ revisions: DARWIN IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos
//@ [TVOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(asm_goto_with_outputs, no_core, lang_items)]
#![no_core]

extern crate minicore;
use minicore::*;

#[no_mangle]
pub unsafe fn inline_asm_goto() {
    unsafe { asm!("b {}", label {}) };
}

// CHECK-LABEL: define{{.*}} @inline_asm_goto(
// CHECK-NOT: callbr{{.*}}"ptrauth"
// CHECK: callbr void asm sideeffect alignstack "b ${0:l}", "!i{{.*}}"() #{{[0-9]+}}{{$}}
// CHECK-NEXT: to label %{{.*}} [label %{{.*}}]
