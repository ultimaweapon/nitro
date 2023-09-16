pub use self::func::*;
pub use self::ty::*;

use crate::pkg::PackageVersion;
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use std::ffi::CStr;

mod func;
mod ty;

/// A context for code generation.
///
/// Each [`Codegen`] can output only one binary.
pub struct Codegen<'a> {
    module: LLVMModuleRef,
    llvm: LLVMContextRef,
    pkg: &'a str,
    version: &'a PackageVersion,
}

impl<'a> Codegen<'a> {
    pub fn new<M: AsRef<CStr>>(pkg: &'a str, version: &'a PackageVersion, module: M) -> Self {
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ref().as_ptr(), llvm) };

        Self {
            module,
            llvm,
            pkg,
            version,
        }
    }

    pub fn encode_name(&self, container: &str, name: &str) -> String {
        // TODO: Create a mangleg name according to Itanium C++ ABI.
        // https://itanium-cxx-abi.github.io/cxx-abi/abi.html might be useful.
        if self.version.major() == 0 {
            format!(
                "{}::0.{}::{}.{}",
                self.pkg,
                self.version.minor(),
                container,
                name
            )
        } else {
            format!(
                "{}::{}::{}.{}",
                self.pkg,
                self.version.major(),
                container,
                name
            )
        }
    }
}

impl<'a> Drop for Codegen<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
    }
}
