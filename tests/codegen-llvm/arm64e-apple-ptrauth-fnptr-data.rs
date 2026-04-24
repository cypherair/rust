//@ add-minicore
//@ revisions: DARWIN IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=0 -Zverify-llvm-ir
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=0 -Zverify-llvm-ir
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=0 -Zverify-llvm-ir
//@ [TVOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos -Copt-level=0 -Zverify-llvm-ir
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

#[unsafe(no_mangle)]
pub extern "C" fn invoke_main(main_fn: extern "C" fn(*const u8) -> i32, state: *const u8) -> i32 {
    main_fn(state)
}

#[unsafe(no_mangle)]
pub extern "C" fn sample_main(_: *const u8) -> i32 {
    7
}

#[unsafe(no_mangle)]
pub extern "C" fn call_sample() -> i32 {
    let f = sample_main;
    invoke_main(f, 0 as *const u8)
}

// CHECK-LABEL: define{{.*}} i32 @invoke_main(
// CHECK: call i32 %{{.*}}(ptr %{{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]

// CHECK-LABEL: define{{.*}} i32 @call_sample(
// CHECK: call i32 @invoke_main(ptr ptrauth (ptr @sample_main, i32 0), ptr null)
