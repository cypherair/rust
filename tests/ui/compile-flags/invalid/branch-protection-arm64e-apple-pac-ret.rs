//@ revisions: DARWIN_PACRET DARWIN_PAUTHLR_LEAF IOS_PACRET IOS_PAUTHLR_LEAF TVOS_PACRET TVOS_PAUTHLR_LEAF VISIONOS_PACRET VISIONOS_PAUTHLR_LEAF
//@ [DARWIN_PACRET] compile-flags: --target=arm64e-apple-darwin -Zbranch-protection=pac-ret
//@ [DARWIN_PACRET] check-fail
//@ [DARWIN_PACRET] needs-llvm-components: aarch64
//@ [DARWIN_PAUTHLR_LEAF] compile-flags: --target=arm64e-apple-darwin -Zbranch-protection=pac-ret,pc,leaf
//@ [DARWIN_PAUTHLR_LEAF] check-fail
//@ [DARWIN_PAUTHLR_LEAF] needs-llvm-components: aarch64
//@ [IOS_PACRET] compile-flags: --target=arm64e-apple-ios -Zbranch-protection=pac-ret
//@ [IOS_PACRET] check-fail
//@ [IOS_PACRET] needs-llvm-components: aarch64
//@ [IOS_PAUTHLR_LEAF] compile-flags: --target=arm64e-apple-ios -Zbranch-protection=pac-ret,pc,leaf
//@ [IOS_PAUTHLR_LEAF] check-fail
//@ [IOS_PAUTHLR_LEAF] needs-llvm-components: aarch64
//@ [TVOS_PACRET] compile-flags: --target=arm64e-apple-tvos -Zbranch-protection=pac-ret
//@ [TVOS_PACRET] check-fail
//@ [TVOS_PACRET] needs-llvm-components: aarch64
//@ [TVOS_PAUTHLR_LEAF] compile-flags: --target=arm64e-apple-tvos -Zbranch-protection=pac-ret,pc,leaf
//@ [TVOS_PAUTHLR_LEAF] check-fail
//@ [TVOS_PAUTHLR_LEAF] needs-llvm-components: aarch64
//@ [VISIONOS_PACRET] compile-flags: --target=arm64e-apple-visionos -Zbranch-protection=pac-ret
//@ [VISIONOS_PACRET] check-fail
//@ [VISIONOS_PACRET] needs-llvm-components: aarch64
//@ [VISIONOS_PAUTHLR_LEAF] compile-flags: --target=arm64e-apple-visionos -Zbranch-protection=pac-ret,pc,leaf
//@ [VISIONOS_PAUTHLR_LEAF] check-fail
//@ [VISIONOS_PAUTHLR_LEAF] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

//~? ERROR `-Zbranch-protection` with `pac-ret` is incompatible with arm64e Apple targets because they enable `ptrauth-returns` by default
