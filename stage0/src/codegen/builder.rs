use super::{BasicBlock, Codegen};
use crate::ffi::{
    llvm_builder_append_block, llvm_builder_dispose, llvm_builder_new, llvm_builder_ret,
    llvm_builder_ret_void,
};
use std::marker::PhantomData;

/// Encapsulate an LLVM IR builder.
pub struct Builder<'a, 'b: 'a> {
    raw: *mut crate::ffi::LlvmBuilder,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> Builder<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>, block: &mut BasicBlock<'a, 'b>) -> Self {
        let raw = unsafe { llvm_builder_new(cx.llvm) };

        unsafe { llvm_builder_append_block(raw, block.as_raw()) };

        Self {
            raw,
            phantom: PhantomData,
        }
    }

    pub fn ret_void(&mut self) -> *mut crate::ffi::LlvmReturn {
        unsafe { llvm_builder_ret_void(self.raw) }
    }

    pub fn ret(&mut self, v: *mut crate::ffi::LlvmValue) -> *mut crate::ffi::LlvmReturn {
        unsafe { llvm_builder_ret(self.raw, v) }
    }
}

impl<'a, 'b: 'a> Drop for Builder<'a, 'b> {
    fn drop(&mut self) {
        unsafe { llvm_builder_dispose(self.raw) };
    }
}
