//! A Rust static library in the shape of the engine. A C host built by clang links it with the
//! system linker, calls into it, hands it a table of callbacks, and calls it from a thread that
//! Rust did not create. `run.sh` builds both and checks what the host prints.

use std::cell::Cell;
use std::ffi::c_void;
use std::panic::catch_unwind;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static CONSTRUCTOR_RAN: AtomicBool = AtomicBool::new(false);

extern "C" fn constructor() {
    CONSTRUCTOR_RAN.store(true, Ordering::SeqCst);
}

/// Where the `ctor` crate puts its runner.
#[used]
#[unsafe(link_section = "__DATA,__mod_init_func,mod_init_funcs")]
static CONSTRUCTOR: extern "C" fn() = constructor;

#[unsafe(no_mangle)]
pub extern "C" fn rust_constructor_ran() -> i32 {
    CONSTRUCTOR_RAN.load(Ordering::SeqCst) as i32
}

/// The host's callbacks, signed by clang, which Rust keeps and calls later from its own thread.
#[repr(C)]
pub struct Callbacks {
    add: extern "C" fn(u32, u32) -> u32,
}

static CALLBACKS: Mutex<Option<Callbacks>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn rust_register(callbacks: Callbacks) {
    *CALLBACKS.lock().unwrap() = Some(callbacks);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_call_registered_on_thread(a: u32, b: u32) -> u32 {
    std::thread::spawn(move || (CALLBACKS.lock().unwrap().as_ref().unwrap().add)(a, b))
        .join()
        .unwrap()
}

/// Rust's functions, signed by rustc, which the host calls.
#[repr(C)]
pub struct RustTable {
    square: extern "C" fn(u32) -> u32,
}

extern "C" fn square(x: u32) -> u32 {
    x * x
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_table() -> RustTable {
    RustTable { square }
}

unsafe extern "C" {
    fn qsort(
        base: *mut c_void,
        count: usize,
        width: usize,
        compare: extern "C" fn(*const c_void, *const c_void) -> i32,
    );
}

extern "C" fn compare(a: *const c_void, b: *const c_void) -> i32 {
    let (a, b) = unsafe { (*a.cast::<u32>(), *b.cast::<u32>()) };
    a.cmp(&b) as i32
}

/// libSystem calls a Rust comparator.
#[unsafe(no_mangle)]
pub extern "C" fn rust_sort_with_qsort() -> u32 {
    let mut values = [5u32, 3, 9, 1];
    unsafe { qsort(values.as_mut_ptr().cast(), values.len(), size_of::<u32>(), compare) };
    values.iter().fold(0, |acc, v| acc * 10 + v)
}

/// Catches its own panic at the C boundary, as UniFFI does.
#[unsafe(no_mangle)]
pub extern "C" fn rust_checked(divisor: u32) -> i32 {
    catch_unwind(|| {
        if divisor == 0 {
            panic!("caught at the C boundary");
        }
        (100 / divisor) as i32
    })
    .unwrap_or(-1)
}

/// A panic that escapes an `extern "C"` function: the process must abort, not trap.
#[unsafe(no_mangle)]
pub extern "C" fn rust_escaping_panic() {
    panic!("escapes an extern \"C\" function");
}

thread_local! {
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

/// Called on a thread the host created: a caught panic, a thread-local, and `thread::current`.
#[unsafe(no_mangle)]
pub extern "C" fn rust_on_foreign_thread() -> i32 {
    let caught = rust_checked(0) == -1;
    let calls = CALLS.with(|calls| {
        calls.set(calls.get() + 1);
        calls.get()
    });
    let unnamed = std::thread::current().name().is_none();
    (caught && calls == 1 && unnamed) as i32
}
