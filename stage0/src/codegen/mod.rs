pub use self::func::*;
pub use self::resolver::*;
pub use self::ty::*;

use crate::ast::{Path, Use};
use crate::pkg::PackageVersion;
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use std::ffi::CStr;

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
    resolver: Resolver<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new<M: AsRef<CStr>>(
        pkg: &'a str,
        version: &'a PackageVersion,
        module: M,
        resolver: Resolver<'a>,
    ) -> Self {
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ref().as_ptr(), llvm) };

        Self {
            module,
            llvm,
            pkg,
            version,
            namespace: "",
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
            format!(
                "{}::0.{}::{}.{}",
                self.pkg,
                self.version.minor(),
                container,
                name
            )
        } else {
            format!(
                "{}::{}::{}.{}",
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

        // Get LLVM type.
        let ty = self.resolver.resolve(&name)?;

        todo!()
    }
}

impl<'a> Drop for Codegen<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
    }
}
