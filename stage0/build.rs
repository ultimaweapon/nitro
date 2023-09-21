fn main() {
    // Static link to libstdc++ on Linux.
    let profile = std::env::var("PROFILE").unwrap();
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if profile == "release" && os == "linux" {
        println!("cargo:rustc-link-arg-bins=-static-libgcc");
        println!("cargo:rustc-link-arg-bins=-static-libstdc++");
    }

    // Link FFI library.
    let ffi = std::env::var("NITRO_FFI").unwrap();

    println!("cargo:rustc-link-search={}", ffi);
    println!("cargo:rustc-link-lib=ffi");
}
