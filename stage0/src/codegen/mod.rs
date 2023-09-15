pub use self::ty::*;

use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use std::ffi::CStr;

mod ty;

/// A context for code generation.
///
/// Each [`Codegen`] can output only one binary.
pub struct Codegen {
    module: LLVMModuleRef,
    llvm: LLVMContextRef,
}

impl Codegen {
    pub fn new<M: AsRef<CStr>>(module: M) -> Self {
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ref().as_ptr(), llvm) };

        Self { module, llvm }
    }
}

impl Drop for Codegen {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
    }
}
