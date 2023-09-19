use super::Codegen;
use llvm_sys::core::{LLVMCreateBasicBlockInContext, LLVMDeleteBasicBlock};
use llvm_sys::prelude::LLVMBasicBlockRef;
use std::marker::PhantomData;

/// Encapsulate an LLVM basic block.
pub struct BasicBlock<'a, 'b: 'a> {
    value: LLVMBasicBlockRef,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> BasicBlock<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            value: unsafe { LLVMCreateBasicBlockInContext(cx.llvm, b"\0".as_ptr() as _) },
            phantom: PhantomData,
        }
    }

    pub fn as_raw(&self) -> LLVMBasicBlockRef {
        self.value
    }
}

impl<'a, 'b: 'a> Drop for BasicBlock<'a, 'b> {
    fn drop(&mut self) {
        unsafe { LLVMDeleteBasicBlock(self.value) };
    }
}
