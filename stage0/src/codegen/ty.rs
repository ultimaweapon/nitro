use super::Codegen;
use llvm_sys::core::{LLVMInt8TypeInContext, LLVMPointerTypeInContext, LLVMVoidTypeInContext};
use llvm_sys::prelude::LLVMTypeRef;
use std::marker::PhantomData;

/// Encapsulate an LLVM type.
pub enum LlvmType<'a> {
    Void(LlvmVoid<'a>),
    U8(LlvmU8<'a>),
    Ptr(LlvmPtr<'a>),
}

/// An unit type.
pub struct LlvmVoid<'a> {
    ty: LLVMTypeRef,
    phantom: PhantomData<&'a Codegen>,
}

impl<'a> LlvmVoid<'a> {
    pub fn new(cx: &'a Codegen) -> Self {
        Self {
            ty: unsafe { LLVMVoidTypeInContext(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A `u8` type.
pub struct LlvmU8<'a> {
    ty: LLVMTypeRef,
    phantom: PhantomData<&'a Codegen>,
}

impl<'a> LlvmU8<'a> {
    pub fn new(cx: &'a Codegen) -> Self {
        Self {
            ty: unsafe { LLVMInt8TypeInContext(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A pointer to something.
pub struct LlvmPtr<'a> {
    ty: LLVMTypeRef,
    pointee: Box<LlvmType<'a>>,
    phantom: PhantomData<&'a Codegen>,
}

impl<'a> LlvmPtr<'a> {
    pub fn new(cx: &'a Codegen, pointee: LlvmType<'a>) -> Self {
        Self {
            ty: unsafe { LLVMPointerTypeInContext(cx.llvm, 0) },
            pointee: Box::new(pointee),
            phantom: PhantomData,
        }
    }
}
