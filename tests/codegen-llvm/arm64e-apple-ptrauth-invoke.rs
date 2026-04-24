// ignore-tidy-linelength
//@ add-minicore
//@ revisions: DARWIN IOS TVOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [TVOS] needs-llvm-components: aarch64
#![crate_type = "lib"]
#![feature(intrinsics, no_core, lang_items)]
#![no_core]

extern crate minicore;

#[rustc_intrinsic]
pub unsafe fn catch_unwind(
    try_fn: fn(*mut u8),
    data: *mut u8,
    catch_fn: fn(*mut u8, *mut u8),
) -> i32;

fn try_fn(_: *mut u8) {}
fn catch_fn(_: *mut u8, _: *mut u8) {}

#[unsafe(no_mangle)]
pub unsafe fn invoke_catch() -> i32 {
    unsafe { catch_unwind(try_fn, 0 as *mut u8, catch_fn) }
}

// CHECK: define {{.*}}@__rust_try(
// CHECK: invoke void %{{.*}}(ptr %{{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]
