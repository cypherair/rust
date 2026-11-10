// ignore-tidy-file-linelength
// Callees that codegen calls directly stay unsigned and carry no `ptrauth` bundle, even
// unoptimised: drop glue, a lang item (the abort when a `nounwind` function would unwind), and
// the personality, which unwind tables name. minicore has no `Drop` impl and no personality, so
// this test declares its own lang items.
//@ revisions: DARWIN IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=0 -Cpanic=unwind
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=0 -Cpanic=unwind
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=0 -Cpanic=unwind
//@ [TVOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos -Copt-level=0 -Cpanic=unwind
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

#[lang = "destruct"]
pub trait Destruct: PointeeSized {}

#[lang = "copy"]
pub trait Copy: Sized {}

#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}

impl<T: PointeeSized> LegacyReceiver for &mut T {}

#[lang = "drop"]
pub trait Drop {
    fn drop(&mut self);
}

#[lang = "drop_glue"]
fn drop_glue<T>(_: &mut T) {}

#[lang = "panic_cannot_unwind"]
fn panic_cannot_unwind() -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn rust_eh_personality() {}

pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {}
}

unsafe extern "C-unwind" {
    safe fn may_unwind();
}

// CHECK-LABEL: @drop_guard(
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: call void @{{.*}}drop_glue{{.*}}(ptr
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: ret void
#[unsafe(no_mangle)]
pub fn drop_guard(_guard: Guard) {}

// CHECK-LABEL: @no_unwind_boundary(
// CHECK-SAME: personality ptr @{{[^ ]*}}rust_eh_personality
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: invoke void @may_unwind()
// CHECK-NOT: "ptrauth"
// CHECK-NOT: ptrauth (ptr
// CHECK: call void @{{.*}}panic_cannot_unwind{{.*}}()
#[unsafe(no_mangle)]
#[rustc_nounwind]
pub fn no_unwind_boundary() {
    may_unwind();
}
