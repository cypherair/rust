// ignore-tidy-linelength
// On arm64e Apple targets every function address that becomes a value is signed with key IA and
// discriminator 0, whatever the function's ABI: function pointers, closures, constants, statics,
// `extern_weak` statics and v-table entries. Every indirect call authenticates its callee, and
// direct calls stay direct, even unoptimised.
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
#![feature(no_core, lang_items, linkage)]
#![no_core]

extern crate minicore;
use minicore::*;

pub trait Shape {
    fn area(&self) -> u32;
}

pub struct Square(pub u32);

impl Shape for Square {
    fn area(&self) -> u32 {
        self.0
    }
}

#[inline(never)]
fn rust_target() -> u32 {
    7
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn c_target(x: u32) -> u32 {
    x
}

const RUST_TARGET: fn() -> u32 = rust_target;

// rustc recognises an `extern_weak` function by the `Option` lang item, which minicore lacks.
#[lang = "Option"]
pub enum Weak<T> {
    Missing,
    Present(T),
}
impl<T: Copy> Copy for Weak<T> {}

#[allow(improper_ctypes)]
extern "C" {
    #[linkage = "extern_weak"]
    static weak_rust_fn: Weak<fn() -> u32>;
}

// CHECK-DAG: @RUST_TARGET_STATIC = {{.*}}constant ptr ptrauth (ptr @{{.*}}rust_target{{.*}}, i32 0)
// CHECK-DAG: @_rust_extern_with_linkage_{{.*}}weak_rust_fn = internal {{.*}}global ptr ptrauth (ptr @weak_rust_fn, i32 0)
// CHECK-DAG: constant {{.*}}ptrauth (ptr @{{.*}}Square{{.*}}area{{.*}}, i32 0)
#[unsafe(no_mangle)]
pub static RUST_TARGET_STATIC: fn() -> u32 = rust_target;

// CHECK-LABEL: @rust_fn_ptr(
// CHECK: ret ptr ptrauth (ptr @{{.*}}rust_target{{.*}}, i32 0)
#[unsafe(no_mangle)]
pub fn rust_fn_ptr() -> fn() -> u32 {
    rust_target
}

// CHECK-LABEL: @closure_fn_ptr(
// CHECK: ret ptr ptrauth (ptr @{{.*}}, i32 0)
#[unsafe(no_mangle)]
pub fn closure_fn_ptr() -> fn() -> u32 {
    || 7
}

// CHECK-LABEL: @const_fn_ptr(
// CHECK: ret ptr ptrauth (ptr @{{.*}}rust_target{{.*}}, i32 0)
#[unsafe(no_mangle)]
pub fn const_fn_ptr() -> fn() -> u32 {
    RUST_TARGET
}

#[unsafe(no_mangle)]
pub fn weak_fn_ptr() -> Weak<fn() -> u32> {
    unsafe { weak_rust_fn }
}

// CHECK-LABEL: @call_rust_fn_ptr(
// CHECK: call {{.*}}i32 %{{.*}}(){{.*}} [ "ptrauth"(i32 0, i64 0) ]
#[unsafe(no_mangle)]
pub fn call_rust_fn_ptr(f: fn() -> u32) -> u32 {
    f()
}

// CHECK-LABEL: @call_reified(
// CHECK: call {{.*}}i32 ptrauth (ptr @{{.*}}rust_target{{.*}}, i32 0)(){{.*}} [ "ptrauth"(i32 0, i64 0) ]
#[unsafe(no_mangle)]
pub fn call_reified() -> u32 {
    let f: fn() -> u32 = rust_target;
    f()
}

// CHECK-LABEL: @call_dyn(
// CHECK: call {{.*}}i32 %{{.*}}(ptr {{.*}}%{{.*}}){{.*}} [ "ptrauth"(i32 0, i64 0) ]
#[unsafe(no_mangle)]
pub fn call_dyn(shape: &dyn Shape) -> u32 {
    shape.area()
}

// CHECK-LABEL: @make_dyn(
#[unsafe(no_mangle)]
pub fn make_dyn(square: &Square) -> u32 {
    call_dyn(square)
}

// CHECK-LABEL: @direct_calls(
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: call {{.*}}i32 @{{.*}}rust_target{{.*}}()
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: call {{.*}}i32 @c_target(i32 {{.*}}%{{.*}})
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: ret i32
#[unsafe(no_mangle)]
pub fn direct_calls(x: u32) -> u32 {
    rust_target();
    c_target(x)
}
