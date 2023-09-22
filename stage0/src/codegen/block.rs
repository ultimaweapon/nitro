use super::Codegen;
use crate::ffi::{llvm_block_dispose, llvm_block_new};
use std::marker::PhantomData;

/// Encapsulate an LLVM basic block.
pub struct BasicBlock<'a, 'b: 'a> {
    value: *mut crate::ffi::LlvmBlock,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> BasicBlock<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            value: unsafe { llvm_block_new(cx.llvm) },
            phantom: PhantomData,
        }
    }

    pub fn as_raw(&self) -> *mut crate::ffi::LlvmBlock {
        self.value
    }
}

impl<'a, 'b: 'a> Drop for BasicBlock<'a, 'b> {
    fn drop(&mut self) {
        unsafe { llvm_block_dispose(self.value) };
    }
}
