// The plain arm64 Apple targets keep upstream's behaviour: no pointer-authentication attributes,
// no signed function pointers and no `ptrauth` bundles.
//@ add-minicore
//@ revisions: AARCH64_DARWIN AARCH64_IOS AARCH64_VISIONOS
//@ [AARCH64_DARWIN] compile-flags: --target aarch64-apple-darwin -Copt-level=0
//@ [AARCH64_DARWIN] needs-llvm-components: aarch64
//@ [AARCH64_IOS] compile-flags: --target aarch64-apple-ios -Copt-level=0
//@ [AARCH64_IOS] needs-llvm-components: aarch64
//@ [AARCH64_VISIONOS] compile-flags: --target aarch64-apple-visionos -Copt-level=0
//@ [AARCH64_VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

// CHECK-NOT: "ptrauth
// CHECK-NOT: ptrauth (ptr
// CHECK-LABEL: @target(
#[unsafe(no_mangle)]
pub fn target() -> u32 {
    7
}

// CHECK-NOT: "ptrauth
// CHECK-NOT: ptrauth (ptr
// CHECK-LABEL: @reify(
// CHECK: ret ptr @target
#[unsafe(no_mangle)]
pub fn reify() -> fn() -> u32 {
    target
}

// CHECK-LABEL: @call(
// CHECK-NOT: "ptrauth
// CHECK: ret i32
#[unsafe(no_mangle)]
pub fn call(f: fn() -> u32) -> u32 {
    f()
}

// CHECK: attributes #
// CHECK-NOT: "ptrauth
