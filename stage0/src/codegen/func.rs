use super::{BasicBlock, Codegen, LlvmType};
use crate::ffi::{
    llvm_function_append, llvm_function_new, llvm_module_get_function, llvm_type_func,
};
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::forget;

/// A function.
pub struct LlvmFunc<'a, 'b: 'a> {
    value: *mut crate::ffi::LlvmFunction,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmFunc<'a, 'b> {
    pub fn get<N: AsRef<CStr>>(cx: &'a Codegen<'b>, name: N) -> Option<Self> {
        let name = name.as_ref();
        let value = unsafe { llvm_module_get_function(cx.module, name.as_ptr()) };

        if value.is_null() {
            None
        } else {
            Some(Self {
                value,
                phantom: PhantomData,
            })
        }
    }

    pub fn new<N: AsRef<CStr>>(
        cx: &'a Codegen<'b>,
        name: N,
        params: &[LlvmType<'a, 'b>],
        ret: LlvmType<'a, 'b>,
    ) -> Self {
        let name = name.as_ref();
        let params: Vec<*mut crate::ffi::LlvmType> = params.iter().map(|p| p.as_raw()).collect();
        let ty = unsafe { llvm_type_func(ret.as_raw(), params.as_ptr(), params.len(), false) };

        Self {
            value: unsafe { llvm_function_new(cx.module, ty, name.as_ptr()) },
            phantom: PhantomData,
        }
    }

    pub fn append(&mut self, block: BasicBlock<'a, 'b>) {
        unsafe { llvm_function_append(self.value, block.as_raw()) };
        forget(block);
    }
}
