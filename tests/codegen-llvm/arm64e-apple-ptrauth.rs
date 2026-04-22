//@ add-minicore
//@ revisions: DARWIN IOS TVOS VISIONOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin
//@ [DARWIN] needs-llvm-components: aarch64
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
use minicore::*;

// CHECK: @test(){{.*}} [[ATTR:#[0-9]+]] {
#[no_mangle]
pub fn test() {}

// CHECK: attributes [[ATTR]] = {{.*}} "ptrauth-auth-traps" "ptrauth-calls" "ptrauth-indirect-gotos" "ptrauth-returns"
// CHECK-SAME: "target-features"="{{.*}}+pauth{{.*}}"
// CHECK-NOT: "sign-return-address"
// CHECK-NOT: "sign-return-address-key"
// CHECK: !{{[0-9]+}} = !{i32 6, !"ptrauth.abi-version", ![[OUTER:[0-9]+]]}
// CHECK: ![[OUTER]] = !{![[PAYLOAD:[0-9]+]]}
// CHECK: ![[PAYLOAD]] = !{i32 0, i1 false}
