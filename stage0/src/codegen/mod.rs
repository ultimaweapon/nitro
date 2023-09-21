pub use self::block::*;
pub use self::builder::*;
pub use self::func::*;
pub use self::resolver::*;
pub use self::ty::*;

use crate::ast::{Expression, Path, Representation, SourceFile, Struct, TypeDefinition, Use};
use crate::lexer::SyntaxError;
use crate::pkg::{PackageVersion, Target};
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeMessage, LLVMDisposeModule,
    LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use llvm_sys::target::{LLVMDisposeTargetData, LLVMPointerSize, LLVMTargetDataRef};
use llvm_sys::target_machine::{
    LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout,
    LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetTargetFromTriple,
    LLVMGetTargetMachineTriple, LLVMRelocMode, LLVMTargetMachineEmitToFile, LLVMTargetMachineRef,
};
use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Display, Formatter};
use std::ptr::{null, null_mut};

mod block;
mod builder;
mod func;
mod resolver;
mod ty;

/// A context for code generation.
///
/// Each [`Codegen`] can output only one binary.
pub struct Codegen<'a> {
    module: LLVMModuleRef,
    llvm: LLVMContextRef,
    pkg: &'a str,
    version: &'a PackageVersion,
    namespace: &'a str,
    layout: LLVMTargetDataRef,
    target: LLVMTargetMachineRef,
    resolver: &'a Resolver<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new<M>(
        pkg: &'a str,
        version: &'a PackageVersion,
        target: &Target,
        module: M,
        resolver: &'a Resolver<'a>,
    ) -> Self
    where
        M: AsRef<CStr>,
    {
        let module = module.as_ref();

        // Get LLVM target.
        let triple = CString::new(target.to_llvm()).unwrap();
        let target = {
            let mut ptr = null_mut();

            assert_eq!(
                unsafe { LLVMGetTargetFromTriple(triple.as_ptr(), &mut ptr, null_mut()) },
                0
            );

            ptr
        };

        // Create LLVM target machine.
        let target = unsafe {
            LLVMCreateTargetMachine(
                target,
                triple.as_ptr(),
                null(),
                null(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            )
        };

        // Create LLVM layout.
        let layout = unsafe { LLVMCreateTargetDataLayout(target) };

        // Create LLVM module.
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ptr(), llvm) };

        Self {
            module,
            llvm,
            pkg,
            version,
            namespace: "",
            layout,
            target,
            resolver,
        }
    }

    pub fn set_namespace(&mut self, v: &'a str) {
        self.namespace = v;
    }

    pub fn triple(&self) -> String {
        unsafe {
            let ptr = LLVMGetTargetMachineTriple(self.target);
            let triple = CStr::from_ptr(ptr).to_str().unwrap().to_owned();
            LLVMDisposeMessage(ptr);
            triple
        }
    }

    /// Returns the pointer size, in bytes.
    pub fn pointer_size(&self) -> u32 {
        unsafe { LLVMPointerSize(self.layout) }
    }

    pub fn run_cfg(&self, expr: &[Expression]) -> Result<bool, SyntaxError> {
        // Get first expression.
        let mut expr = expr.iter();
        let lhs = match expr.next().unwrap() {
            Expression::Value(v) => v,
            e => return Err(SyntaxError::new(e.span(), "expect an identifier")),
        };

        // Get second expression.
        let triple = self.triple();
        let os = triple.split('-').nth(2).unwrap();
        let (equal, span) = match expr.next() {
            Some(Expression::NotEqual(f, s)) => (false, f.span() + s.span()),
            Some(Expression::Equal(f, s)) => (true, f.span() + s.span()),
            Some(e) => return Err(SyntaxError::new(e.span(), "unsupported expression")),
            None => match lhs.value() {
                "unix" => match os {
                    "darwin" | "linux" => return Ok(true),
                    "win32" => return Ok(false),
                    _ => todo!(),
                },
                "win32" => match os {
                    "darwin" | "linux" => return Ok(false),
                    "win32" => return Ok(true),
                    _ => todo!(),
                },
                _ => return Err(SyntaxError::new(lhs.span().clone(), "unknown argument")),
            },
        };

        // Check if first expression is "os".
        if lhs.value() != "os" {
            return Err(SyntaxError::new(lhs.span().clone(), "unknown expression"));
        }

        // Get third argument.
        let rhs = match expr.next() {
            Some(Expression::String(v)) => v,
            Some(t) => return Err(SyntaxError::new(t.span(), "expect a string literal")),
            None => return Err(SyntaxError::new(span, "expect a string literal after this")),
        };

        // Compare.
        let res = if equal {
            match rhs.value() {
                "windows" => match os {
                    "darwin" | "linux" => false,
                    "win32" => true,
                    _ => todo!(),
                },
                _ => todo!(),
            }
        } else {
            match rhs.value() {
                "windows" => match os {
                    "darwin" | "linux" => true,
                    "win32" => false,
                    _ => todo!(),
                },
                _ => todo!(),
            }
        };

        if expr.next().is_some() {
            todo!()
        }

        Ok(res)
    }

    pub fn encode_name(&self, container: &str, name: &str) -> String {
        // TODO: Create a mangleg name according to Itanium C++ ABI.
        // https://itanium-cxx-abi.github.io/cxx-abi/abi.html might be useful.
        if self.version.major() == 0 {
            format!("{}.{}.{}", self.pkg, container, name)
        } else {
            format!(
                "{}.v{}.{}.{}",
                self.pkg,
                self.version.major(),
                container,
                name
            )
        }
    }

    pub fn resolve(&self, uses: &[Use], name: &Path) -> Option<LlvmType<'_, 'a>> {
        // Resolve full name.
        let name = match name.as_local() {
            Some(name) => {
                // Search from use declarations first to allow overrides.
                let mut found = None;

                for u in uses.iter().rev() {
                    match u.rename() {
                        Some(v) => {
                            if v == name {
                                found = Some(u);
                                break;
                            }
                        }
                        None => {
                            if u.name().last() == name {
                                found = Some(u);
                                break;
                            }
                        }
                    }
                }

                match found {
                    Some(v) => v.name().to_string(),
                    None => {
                        if self.namespace.is_empty() {
                            format!("self.{}", name)
                        } else {
                            format!("self.{}.{}", self.namespace, name)
                        }
                    }
                }
            }
            None => name.to_string(),
        };

        // Resolve type and build LLVM type.
        let ty = match self.resolver.resolve(&name)? {
            ResolvedType::Project(v) => self.build_project_type(&name, v),
            ResolvedType::External(_) => todo!(),
        };

        Some(ty)
    }

    pub fn build<F: AsRef<std::path::Path>>(self, file: F) -> Result<(), BuildError> {
        // TODO: Invoke LLVMVerifyModule.
        let mut err = null_mut();
        let file = file.as_ref().to_str().unwrap();
        let file = CString::new(file).unwrap();
        let fail = unsafe {
            LLVMTargetMachineEmitToFile(
                self.target,
                self.module,
                file.as_ptr() as _,
                LLVMCodeGenFileType::LLVMObjectFile,
                &mut err,
            )
        };

        if fail != 0 {
            Err(BuildError::new(err))
        } else {
            Ok(())
        }
    }

    fn build_project_type(&self, name: &str, ty: &SourceFile) -> LlvmType<'_, 'a> {
        match ty.ty().unwrap() {
            TypeDefinition::Struct(v) => self.build_project_struct(name, v),
            TypeDefinition::Class(_) => todo!(),
        }
    }

    fn build_project_struct(&self, name: &str, ty: &Struct) -> LlvmType<'_, 'a> {
        match ty {
            Struct::Primitive(_, r, _, _) => match r {
                Representation::U8 => LlvmType::U8(LlvmU8::new(self)),
                Representation::Un => match self.pointer_size() {
                    8 => LlvmType::U64(LlvmU64::new(self)),
                    _ => todo!(),
                },
            },
            Struct::Composite(_, _, _) => todo!(),
        }
    }
}

impl<'a> Drop for Codegen<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
        unsafe { LLVMDisposeTargetData(self.layout) };
        unsafe { LLVMDisposeTargetMachine(self.target) };
    }
}

/// Represents an error when [`Codegen::build()`] is failed.
#[derive(Debug)]
pub struct BuildError(String);

impl BuildError {
    fn new(llvm: *mut c_char) -> Self {
        let reason = unsafe { CStr::from_ptr(llvm).to_str().unwrap().to_owned() };
        unsafe { LLVMDisposeMessage(llvm) };
        Self(reason)
    }
}

impl Error for BuildError {}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
