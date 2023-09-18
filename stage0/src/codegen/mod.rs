pub use self::func::*;
pub use self::resolver::*;
pub use self::target::*;
pub use self::ty::*;

use crate::ast::{Path, Representation, SourceFile, Struct, TypeDefinition, Use};
use crate::pkg::PackageVersion;
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use llvm_sys::target::{LLVMDisposeTargetData, LLVMTargetDataRef};
use llvm_sys::target_machine::{
    LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout, LLVMCreateTargetMachine,
    LLVMDisposeTargetMachine, LLVMGetTargetFromTriple, LLVMRelocMode, LLVMTargetMachineRef,
};
use std::ffi::CStr;
use std::ptr::{null, null_mut};

mod func;
mod resolver;
mod target;
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
    pub fn new<T, M>(
        pkg: &'a str,
        version: &'a PackageVersion,
        target: Target<T>,
        module: M,
        resolver: &'a Resolver<'a>,
    ) -> Self
    where
        T: AsRef<CStr>,
        M: AsRef<CStr>,
    {
        let triple = target.triple();
        let module = module.as_ref();

        // Get LLVM target.
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

    pub fn encode_name(&self, container: &str, name: &str) -> String {
        // TODO: Create a mangleg name according to Itanium C++ ABI.
        // https://itanium-cxx-abi.github.io/cxx-abi/abi.html might be useful.
        if self.version.major() == 0 {
            format!("{}::{}.{}", self.pkg, container, name)
        } else {
            format!(
                "{}::v{}::{}.{}",
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
                Representation::Un => todo!(),
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
