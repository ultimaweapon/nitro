fn main() {
    // Static link to libstdc++ on Linux.
    let profile = std::env::var("PROFILE").unwrap();
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if profile == "release" && os == "linux" {
        println!("cargo:rustc-link-arg-bins=-static-libstdc++");
    }

    // Link FFI library.
    let ffi = std::env::var("NITRO_FFI").unwrap();

    println!("cargo:rustc-link-search={}", ffi);
    println!("cargo:rustc-link-lib=ffi");

    // Link LLVM.
    let llvm = std::env::var("LLVM_SYS_170_PREFIX").unwrap();

    println!("cargo:rustc-link-search={}/lib", llvm);
    println!("cargo:rustc-link-lib=lldCOFF");
    println!("cargo:rustc-link-lib=lldCommon");
    println!("cargo:rustc-link-lib=lldELF");
    println!("cargo:rustc-link-lib=lldMachO");
}
