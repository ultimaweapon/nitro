use super::Codegen;
use llvm_sys::core::{LLVMInt8TypeInContext, LLVMPointerTypeInContext, LLVMVoidTypeInContext};
use llvm_sys::prelude::LLVMTypeRef;
use std::marker::PhantomData;

/// Encapsulate an LLVM type.
pub enum LlvmType<'a, 'b: 'a> {
    Void(LlvmVoid<'a, 'b>),
    U8(LlvmU8<'a, 'b>),
    Ptr(LlvmPtr<'a, 'b>),
}

impl<'a, 'b: 'a> LlvmType<'a, 'b> {
    pub fn as_raw(&self) -> LLVMTypeRef {
        match self {
            Self::Void(v) => v.ty,
            Self::U8(v) => v.ty,
            Self::Ptr(v) => v.ty,
        }
    }
}

/// An unit type.
pub struct LlvmVoid<'a, 'b: 'a> {
    ty: LLVMTypeRef,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmVoid<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            ty: unsafe { LLVMVoidTypeInContext(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A `u8` type.
pub struct LlvmU8<'a, 'b: 'a> {
    ty: LLVMTypeRef,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmU8<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            ty: unsafe { LLVMInt8TypeInContext(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A pointer to something.
pub struct LlvmPtr<'a, 'b: 'a> {
    ty: LLVMTypeRef,
    pointee: Box<LlvmType<'a, 'b>>,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmPtr<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>, pointee: LlvmType<'a, 'b>) -> Self {
        Self {
            ty: unsafe { LLVMPointerTypeInContext(cx.llvm, 0) },
            pointee: Box::new(pointee),
            phantom: PhantomData,
        }
    }
}
