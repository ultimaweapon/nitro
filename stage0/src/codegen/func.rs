use super::{Codegen, LlvmType};
use llvm_sys::core::{LLVMAddFunction, LLVMFunctionType, LLVMGetNamedFunction};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use std::ffi::CStr;
use std::marker::PhantomData;

/// A function.
pub struct LlvmFunc<'a, 'b: 'a> {
    value: LLVMValueRef,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmFunc<'a, 'b> {
    pub fn get<N: AsRef<CStr>>(cx: &'a Codegen<'b>, name: N) -> Option<Self> {
        let name = name.as_ref();
        let value = unsafe { LLVMGetNamedFunction(cx.module, name.as_ptr()) };

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
        let mut params: Vec<LLVMTypeRef> = params.iter().map(|p| p.as_raw()).collect();
        let ty = unsafe {
            LLVMFunctionType(
                ret.as_raw(),
                params.as_mut_ptr(),
                params.len().try_into().unwrap(),
                0,
            )
        };

        Self {
            value: unsafe { LLVMAddFunction(cx.module, name.as_ptr(), ty) },
            phantom: PhantomData,
        }
    }
}
