// ignore-tidy-file-linelength
// `__rust_try`, which rustc generates for `catch_unwind`, carries the pointer-authentication
// attributes, and its calls through the `try_fn` and `catch_fn` pointers authenticate the callee.
//@ add-minicore
//@ revisions: DARWIN IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [TVOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos -Copt-level=0 -Cpanic=unwind -Zverify-llvm-ir
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(intrinsics, no_core, lang_items)]
#![no_core]

extern crate minicore;

#[rustc_intrinsic]
pub unsafe fn catch_unwind<Data>(
    try_fn: unsafe fn(*mut Data),
    data: *mut Data,
    catch_fn: unsafe fn(*mut Data, *mut u8),
) -> bool;

unsafe fn try_fn(_: *mut u8) {}
unsafe fn catch_fn(_: *mut u8, _: *mut u8) {}

#[unsafe(no_mangle)]
pub unsafe fn invoke_catch() -> bool {
    unsafe { catch_unwind(try_fn, 0 as *mut u8, catch_fn) }
}

// CHECK: define internal {{.*}}@__rust_try({{.*}}) unnamed_addr [[TRY_ATTR:#[0-9]+]] personality
// CHECK: invoke void %{{.*}}(ptr %{{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]
// CHECK: call void %{{.*}}(ptr %{{.*}}, ptr %{{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]
// CHECK: attributes [[TRY_ATTR]] = {{.*}} "ptrauth-auth-traps" "ptrauth-calls" "ptrauth-indirect-gotos" "ptrauth-returns"
