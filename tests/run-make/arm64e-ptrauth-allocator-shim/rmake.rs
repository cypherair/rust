//! rustc builds the allocator shim outside the usual function path, as its own LLVM module. On
//! arm64e Apple targets its functions must still carry the pointer-authentication attributes.

//@ needs-llvm-components: aarch64
//@ ignore-backends: gcc

use run_make_support::{llvm_dis, rfs, rustc};

const PTRAUTH_ATTRS: [&str; 5] = [
    "\"aarch64-jump-table-hardening\"",
    "\"ptrauth-auth-traps\"",
    "\"ptrauth-calls\"",
    "\"ptrauth-indirect-gotos\"",
    "\"ptrauth-returns\"",
];

fn main() {
    // Only `-Csave-temps` keeps the shim's module, as bitcode.
    rustc()
        .input("shim.rs")
        .target("arm64e-apple-darwin")
        .emit("obj")
        .arg("-Csave-temps")
        .arg("-Ccodegen-units=1")
        .run();

    let bitcode: Vec<_> = rfs::read_dir(".")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bc"))
        .collect();
    let mut shims = 0;
    for bc in bitcode {
        let ll = bc.with_extension("ll");
        llvm_dis().input(&bc).arg("-o").arg(&ll).run();
        let ir = rfs::read_to_string(&ll);
        if ir.lines().any(|line| {
            line.starts_with("define ") && line.contains("__rust_no_alloc_shim_is_unstable_v2")
        }) {
            shims += 1;
            check_functions(&ir);
        }
    }
    assert!(shims > 0, "no bitcode file holds the allocator shim");
}

fn check_functions(ir: &str) {
    for define in ir.lines().filter(|line| line.starts_with("define ")) {
        let group = define
            .split_whitespace()
            .find(|word| word.starts_with('#'))
            .unwrap_or_else(|| panic!("no attribute group on `{define}`"));
        let attrs = ir
            .lines()
            .find(|line| line.starts_with(&format!("attributes {group} = ")))
            .unwrap_or_else(|| panic!("attribute group {group} is not defined"));
        for attr in PTRAUTH_ATTRS {
            assert!(attrs.contains(attr), "`{define}` lacks {attr}: `{attrs}`");
        }
    }
}
