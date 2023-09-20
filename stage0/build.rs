fn main() {
    let ffi = std::env::var("NITRO_FFI").unwrap();

    println!("cargo:rustc-link-search={}", ffi);
    println!("cargo:rustc-link-lib=ffi");
}
