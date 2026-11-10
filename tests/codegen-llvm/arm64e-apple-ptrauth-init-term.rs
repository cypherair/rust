// ignore-tidy-file-linelength
// On arm64e Apple targets the entries of `__mod_init_func` and `__mod_term_func` stay unsigned,
// whether the section is named or typed, like the constructors clang emits itself. (Clang signs a
// static that C code places in these sections by hand.) Either form works: Apple's linker turns
// initializer entries into offsets and dyld re-signs them, and dyld ignores `__mod_term_func` on
// arm64e. The same function pointer in an ordinary static is signed.
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
#![feature(no_core)]
#![no_core]

extern crate minicore;

// Different signatures, so that LLVM does not merge the two functions.
extern "C" fn init_fn() {}
extern "C" fn term_fn(_: i32) {}

// CHECK-DAG: @{{.*}}CTOR_CRATE = {{.*}}constant ptr @{{.*}}init_fn{{.*}}, section "__DATA,__mod_init_func,mod_init_funcs"
#[used]
#[unsafe(link_section = "__DATA,__mod_init_func,mod_init_funcs")]
static CTOR_CRATE: extern "C" fn() = init_fn;

// CHECK-DAG: @{{.*}}BY_NAME = {{.*}}constant ptr @{{.*}}init_fn{{.*}}, section "__DATA,__mod_init_func"
#[used]
#[unsafe(link_section = "__DATA,__mod_init_func")]
static BY_NAME: extern "C" fn() = init_fn;

// CHECK-DAG: @{{.*}}BY_TYPE = {{.*}}constant ptr @{{.*}}init_fn{{.*}}, section "__DATA, __inits , mod_init_funcs"
#[used]
#[unsafe(link_section = "__DATA, __inits , mod_init_funcs")]
static BY_TYPE: extern "C" fn() = init_fn;

// CHECK-DAG: @{{.*}}TERM = {{.*}}constant ptr @{{.*}}term_fn{{.*}}, section "__DATA,__mod_term_func,mod_term_funcs"
#[used]
#[unsafe(link_section = "__DATA,__mod_term_func,mod_term_funcs")]
static TERM: extern "C" fn(i32) = term_fn;

// CHECK-DAG: @{{.*}}TERM_BY_TYPE = {{.*}}constant ptr @{{.*}}term_fn{{.*}}, section "__DATA,__terms,mod_term_funcs"
#[used]
#[unsafe(link_section = "__DATA,__terms,mod_term_funcs")]
static TERM_BY_TYPE: extern "C" fn(i32) = term_fn;

// CHECK-DAG: @{{.*}}ORDINARY = {{.*}}constant ptr ptrauth (ptr @{{.*}}init_fn{{.*}}, i32 0)
#[used]
static ORDINARY: extern "C" fn() = init_fn;
