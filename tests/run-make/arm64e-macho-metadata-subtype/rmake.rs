//@ needs-llvm-components: aarch64

use std::path::{Path, PathBuf};

use run_make_support::{llvm_ar, rfs, rustc};

fn main() {
    rustc()
        .input("metadata.rs")
        .target("arm64e-apple-darwin")
        .crate_type("rlib")
        .arg("-o")
        .arg("libmetadata.rlib")
        .run();

    if Path::new("extract").exists() {
        rfs::recursive_remove("extract");
    }
    rfs::create_dir("extract");
    llvm_ar().extract().arg("../libmetadata.rlib").current_dir("extract").run();

    let metadata = PathBuf::from("extract").join("lib.rmeta");
    let bytes = rfs::read(&metadata);
    assert!(bytes.len() >= 12, "metadata object is too small for a Mach-O header");

    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    assert_eq!(magic, 0xfeedfacf, "metadata object should use a 64-bit Mach-O header");

    let subtype = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    assert_eq!(
        subtype, 0x80000002,
        "arm64e metadata object should use CPU_SUBTYPE_ARM64E | CPU_SUBTYPE_LIB64"
    );
}
