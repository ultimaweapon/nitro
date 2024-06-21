fn main() {
    // Link FFI library.
    let ffi = std::env::var("NITRO_FFI").unwrap();

    println!("cargo::rustc-link-search={}", ffi);
    println!("cargo::rustc-link-lib=ffi");

    // Link LLVM.
    let llvm = std::env::var("LLVM_PREFIX").unwrap();

    println!("cargo::rustc-link-search={}/lib", llvm);
    println!("cargo::rustc-link-lib=lldCOFF");
    println!("cargo::rustc-link-lib=lldCommon");
    println!("cargo::rustc-link-lib=lldELF");
    println!("cargo::rustc-link-lib=lldMachO");
    println!("cargo::rustc-link-lib=LLVMWindowsManifest");
    println!("cargo::rustc-link-lib=LLVMLibDriver");
    println!("cargo::rustc-link-lib=LLVMX86AsmParser");
    println!("cargo::rustc-link-lib=LLVMX86CodeGen");
    println!("cargo::rustc-link-lib=LLVMX86Desc");
    println!("cargo::rustc-link-lib=LLVMX86Info");
    println!("cargo::rustc-link-lib=LLVMAArch64AsmParser");
    println!("cargo::rustc-link-lib=LLVMAArch64CodeGen");
    println!("cargo::rustc-link-lib=LLVMAArch64Desc");
    println!("cargo::rustc-link-lib=LLVMAArch64Utils");
    println!("cargo::rustc-link-lib=LLVMAArch64Info");
    println!("cargo::rustc-link-lib=LLVMWindowsDriver");
    println!("cargo::rustc-link-lib=LLVMOption");
    println!("cargo::rustc-link-lib=LLVMMCDisassembler");
    println!("cargo::rustc-link-lib=LLVMLTO");
    println!("cargo::rustc-link-lib=LLVMPasses");
    println!("cargo::rustc-link-lib=LLVMCFGuard");
    println!("cargo::rustc-link-lib=LLVMCoroutines");
    println!("cargo::rustc-link-lib=LLVMipo");
    println!("cargo::rustc-link-lib=LLVMVectorize");
    println!("cargo::rustc-link-lib=LLVMLinker");
    println!("cargo::rustc-link-lib=LLVMInstrumentation");
    println!("cargo::rustc-link-lib=LLVMFrontendOpenMP");
    println!("cargo::rustc-link-lib=LLVMGlobalISel");
    println!("cargo::rustc-link-lib=LLVMAsmPrinter");
    println!("cargo::rustc-link-lib=LLVMSelectionDAG");
    println!("cargo::rustc-link-lib=LLVMCodeGen");
    println!("cargo::rustc-link-lib=LLVMTarget");
    println!("cargo::rustc-link-lib=LLVMObjCARCOpts");
    println!("cargo::rustc-link-lib=LLVMCodeGenTypes");
    println!("cargo::rustc-link-lib=LLVMIRPrinter");
    println!("cargo::rustc-link-lib=LLVMScalarOpts");
    println!("cargo::rustc-link-lib=LLVMInstCombine");
    println!("cargo::rustc-link-lib=LLVMAggressiveInstCombine");
    println!("cargo::rustc-link-lib=LLVMTransformUtils");
    println!("cargo::rustc-link-lib=LLVMBitWriter");
    println!("cargo::rustc-link-lib=LLVMAnalysis");
    println!("cargo::rustc-link-lib=LLVMProfileData");
    println!("cargo::rustc-link-lib=LLVMDebugInfoPDB");
    println!("cargo::rustc-link-lib=LLVMDebugInfoMSF");
    println!("cargo::rustc-link-lib=LLVMDebugInfoDWARF");
    println!("cargo::rustc-link-lib=LLVMObject");
    println!("cargo::rustc-link-lib=LLVMTextAPI");
    println!("cargo::rustc-link-lib=LLVMMCParser");
    println!("cargo::rustc-link-lib=LLVMIRReader");
    println!("cargo::rustc-link-lib=LLVMAsmParser");
    println!("cargo::rustc-link-lib=LLVMMC");
    println!("cargo::rustc-link-lib=LLVMDebugInfoCodeView");
    println!("cargo::rustc-link-lib=LLVMBitReader");
    println!("cargo::rustc-link-lib=LLVMCore");
    println!("cargo::rustc-link-lib=LLVMRemarks");
    println!("cargo::rustc-link-lib=LLVMBitstreamReader");
    println!("cargo::rustc-link-lib=LLVMBinaryFormat");
    println!("cargo::rustc-link-lib=LLVMTargetParser");
    println!("cargo::rustc-link-lib=LLVMSupport");
    println!("cargo::rustc-link-lib=LLVMDemangle");
    println!("cargo::rustc-link-lib=LLVMFrontendOffloading");
    println!("cargo::rustc-link-lib=LLVMHipStdPar");

    // Link zstd.
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let zstd = std::env::var("ZSTD_PREFIX").unwrap();

    println!("cargo::rustc-link-search={}/lib", zstd);

    if os == "windows" {
        println!("cargo::rustc-link-lib=zstd_static");
    } else {
        println!("cargo::rustc-link-lib=zstd");
    }

    // Link C++.
    match os.as_str() {
        "linux" => {
            let profile = std::env::var("PROFILE").unwrap();

            if profile == "release" {
                println!("cargo::rustc-link-lib=static=stdc++");
            } else {
                println!("cargo::rustc-link-lib=stdc++");
            }
        }
        "macos" => println!("cargo::rustc-link-lib=c++"),
        "windows" => {}
        _ => todo!(),
    }
}
