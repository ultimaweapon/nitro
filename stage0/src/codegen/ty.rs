use super::Codegen;
use crate::ffi::{llvm_type_int64, llvm_type_int8, llvm_type_ptr, llvm_type_void};
use std::marker::PhantomData;

/// Encapsulate an LLVM type.
pub enum LlvmType<'a, 'b: 'a> {
    Void(LlvmVoid<'a, 'b>),
    U8(LlvmU8<'a, 'b>),
    U64(LlvmU64<'a, 'b>),
    Ptr(LlvmPtr<'a, 'b>),
}

impl<'a, 'b: 'a> LlvmType<'a, 'b> {
    pub fn as_raw(&self) -> *mut crate::ffi::LlvmType {
        match self {
            Self::Void(v) => v.raw,
            Self::U8(v) => v.raw as _,
            Self::U64(v) => v.raw as _,
            Self::Ptr(v) => v.raw as _,
        }
    }
}

/// An unit type.
pub struct LlvmVoid<'a, 'b: 'a> {
    raw: *mut crate::ffi::LlvmType,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmVoid<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            raw: unsafe { llvm_type_void(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A 8-bits unsigned integer.
pub struct LlvmU8<'a, 'b: 'a> {
    raw: *mut crate::ffi::LlvmInteger,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmU8<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            raw: unsafe { llvm_type_int8(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A 64-bits unsigned integer.
pub struct LlvmU64<'a, 'b: 'a> {
    raw: *mut crate::ffi::LlvmInteger,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmU64<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>) -> Self {
        Self {
            raw: unsafe { llvm_type_int64(cx.llvm) },
            phantom: PhantomData,
        }
    }
}

/// A pointer to something.
pub struct LlvmPtr<'a, 'b: 'a> {
    raw: *mut crate::ffi::LlvmPointer,
    pointee: Box<LlvmType<'a, 'b>>,
    phantom: PhantomData<&'a Codegen<'b>>,
}

impl<'a, 'b: 'a> LlvmPtr<'a, 'b> {
    pub fn new(cx: &'a Codegen<'b>, pointee: LlvmType<'a, 'b>) -> Self {
        Self {
            raw: unsafe { llvm_type_ptr(cx.llvm) },
            pointee: Box::new(pointee),
            phantom: PhantomData,
        }
    }
}
