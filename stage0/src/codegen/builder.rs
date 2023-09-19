use super::{BasicBlock, Codegen};
use llvm_sys::core::{
    LLVMBuildRetVoid, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMPositionBuilderAtEnd,
};
use llvm_sys::prelude::LLVMBuilderRef;
use std::marker::PhantomData;

/// Encapsulate an LLVM IR builder.
pub struct Builder<'a, 'b: 'a> {
    raw: LLVMBuilderRef,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> Builder<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>, block: &mut BasicBlock<'a, 'b>) -> Self {
        let raw = unsafe { LLVMCreateBuilderInContext(cx.llvm) };

        unsafe { LLVMPositionBuilderAtEnd(raw, block.as_raw()) };

        Self {
            raw,
            phantom: PhantomData,
        }
    }

    pub fn ret_void(&mut self) {
        unsafe { LLVMBuildRetVoid(self.raw) };
    }
}

impl<'a, 'b: 'a> Drop for Builder<'a, 'b> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.raw) };
    }
}
