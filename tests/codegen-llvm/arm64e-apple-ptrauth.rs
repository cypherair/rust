// ignore-tidy-linelength
// Every function on an arm64e Apple target carries the four attributes that clang's Darwin driver
// turns on, and none of the ELF PAuth ABI's attributes or module flags.
//@ add-minicore
//@ filecheck-flags: --implicit-check-not=aarch64-jump-table-hardening
//@ filecheck-flags: --implicit-check-not=aarch64-elf-pauthabi --implicit-check-not=ptrauth-elf-got
//@ filecheck-flags: --implicit-check-not=ptrauth-sign-personality
//@ filecheck-flags: --implicit-check-not=ptrauth.abi-version
//@ revisions: DARWIN DARWIN_BTI DARWIN_GCS IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin
//@ [DARWIN] needs-llvm-components: aarch64
//@ [DARWIN_BTI] compile-flags: --target arm64e-apple-darwin -Zbranch-protection=bti
//@ [DARWIN_BTI] needs-llvm-components: aarch64
//@ [DARWIN_GCS] compile-flags: --target arm64e-apple-darwin -Zbranch-protection=gcs
//@ [DARWIN_GCS] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos
//@ [TVOS] needs-llvm-components: aarch64
//@ [VISIONOS] compile-flags: --target arm64e-apple-visionos
//@ [VISIONOS] needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;

// CHECK: @test(){{.*}} [[ATTR:#[0-9]+]] {
#[unsafe(no_mangle)]
pub fn test() {}

// CHECK: attributes [[ATTR]] = {{.*}} "ptrauth-auth-traps" "ptrauth-calls" "ptrauth-indirect-gotos" "ptrauth-returns"
// CHECK-SAME: "target-features"="{{.*}}+pauth{{.*}}"
// DARWIN_BTI: !{{[0-9]+}} = !{i32 8, !"branch-target-enforcement", i32 1}
// DARWIN_GCS: !{{[0-9]+}} = !{i32 8, !"guarded-control-stack", i32 1}
