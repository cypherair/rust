//@ add-minicore
//@ revisions: DARWIN IOS TVOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=2
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=2
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=2
//@ [TVOS] needs-llvm-components: aarch64
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

// CHECK-LABEL: define{{.*}} @direct(
// CHECK-NOT: [ "ptrauth"(i32 0, i64 0) ]
// CHECK: ret i32

// CHECK-LABEL: define{{.*}} @indirect_tail(
// CHECK: {{(tail )?}}call{{.*}} {{%.*}}(i32 {{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]{{$}}

// CHECK-LABEL: define{{.*}} @indirect_non_tail(
// CHECK: {{(tail )?}}call{{.*}} {{%.*}}(i32 {{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]{{$}}
