//@ needs-llvm-components: aarch64

use std::path::{Path, PathBuf};

use run_make_support::{llvm_ar, llvm_dis, llvm_objcopy, rfs, rustc};

fn assert_no_direct_ptrauth(path: &str) {
    let ir = rfs::read_to_string(path);
    for (line_idx, line) in ir.lines().enumerate() {
        if line.contains("@target(") && line.contains("\"ptrauth\"") {
            panic!(
                "{path}:{} still contains a direct call with a ptrauth operand bundle:\n{line}",
                line_idx + 1
            );
        }
    }
}

fn main() {
    rustc()
        .input("ptrauth.rs")
        .target("arm64e-apple-darwin")
        .opt_level("2")
        .arg("--emit=llvm-ir=module.ll,llvm-bc=module.bc")
        .run();
    assert_no_direct_ptrauth("module.ll");

    llvm_dis().input("module.bc").arg("-o").arg("module-bc.ll").run();
    assert_no_direct_ptrauth("module-bc.ll");

    rustc()
        .input("ptrauth.rs")
        .target("arm64e-apple-darwin")
        .opt_level("2")
        .arg("-Cembed-bitcode=yes")
        .crate_type("rlib")
        .arg("-o")
        .arg("libptrauth.rlib")
        .run();

    if Path::new("extract").exists() {
        rfs::recursive_remove("extract");
    }
    rfs::create_dir("extract");
    llvm_ar().extract().arg("../libptrauth.rlib").current_dir("extract").run();
    let object = rfs::read_dir("extract")
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "o"))
        .expect("embedded-bitcode rlib should contain an object file");
    let embedded_bc = PathBuf::from("extract").join("embedded.bc");
    llvm_objcopy().dump_section("__LLVM,__bitcode", &embedded_bc).arg(&object).run();
    llvm_dis().input(&embedded_bc).arg("-o").arg("embedded.ll").run();
    assert_no_direct_ptrauth("embedded.ll");
}
