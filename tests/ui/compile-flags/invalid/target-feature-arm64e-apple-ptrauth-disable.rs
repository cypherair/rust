//@ revisions: DARWIN_PACA DARWIN_PACG DARWIN_BOTH IOS_BOTH TVOS_BOTH VISIONOS_BOTH
//@ [DARWIN_PACA] compile-flags: --target=arm64e-apple-darwin -Ctarget-feature=-paca
//@ [DARWIN_PACA] check-fail
//@ [DARWIN_PACA] needs-llvm-components: aarch64
//@ [DARWIN_PACG] compile-flags: --target=arm64e-apple-darwin -Ctarget-feature=-pacg
//@ [DARWIN_PACG] check-fail
//@ [DARWIN_PACG] needs-llvm-components: aarch64
//@ [DARWIN_BOTH] compile-flags: --target=arm64e-apple-darwin -Ctarget-feature=-paca,-pacg
//@ [DARWIN_BOTH] check-fail
//@ [DARWIN_BOTH] needs-llvm-components: aarch64
//@ [IOS_BOTH] compile-flags: --target=arm64e-apple-ios -Ctarget-feature=-paca,-pacg
//@ [IOS_BOTH] check-fail
//@ [IOS_BOTH] needs-llvm-components: aarch64
//@ [TVOS_BOTH] compile-flags: --target=arm64e-apple-tvos -Ctarget-feature=-paca,-pacg
//@ [TVOS_BOTH] check-fail
//@ [TVOS_BOTH] needs-llvm-components: aarch64
//@ [VISIONOS_BOTH] compile-flags: --target=arm64e-apple-visionos -Ctarget-feature=-paca,-pacg
//@ [VISIONOS_BOTH] check-fail
//@ [VISIONOS_BOTH] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

//~? ERROR `-Ctarget-feature` cannot disable `paca` or `pacg` on arm64e Apple targets because they enable ptrauth by default
