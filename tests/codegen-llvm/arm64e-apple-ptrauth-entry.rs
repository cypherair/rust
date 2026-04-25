//@ add-minicore
//@ revisions: DARWIN IOS TVOS
//@ [DARWIN] compile-flags: --target arm64e-apple-darwin -Copt-level=0
//@ [DARWIN] needs-llvm-components: aarch64
//@ [IOS] compile-flags: --target arm64e-apple-ios -Copt-level=0
//@ [IOS] needs-llvm-components: aarch64
//@ [TVOS] compile-flags: --target arm64e-apple-tvos -Copt-level=0
//@ [TVOS] needs-llvm-components: aarch64
#![feature(no_core, lang_items)]
#![no_core]

extern crate minicore;
use minicore::*;

#[lang = "start"]
fn lang_start<T: 'static>(
    _main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    0
}

fn main() {}

// CHECK: define i32 @main({{.*}}) unnamed_addr [[MAIN_ATTR:#[0-9]+]]
// CHECK: attributes [[MAIN_ATTR]] = {{.*}} "ptrauth-auth-traps" "ptrauth-calls"
// CHECK-SAME: "ptrauth-indirect-gotos" "ptrauth-returns"
