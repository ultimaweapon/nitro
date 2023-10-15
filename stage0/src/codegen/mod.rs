pub use self::block::*;
pub use self::builder::*;
pub use self::func::*;
pub use self::resolver::*;
pub use self::ty::*;

use crate::ffi::{
    llvm_context_dispose, llvm_context_new, llvm_layout_dispose, llvm_layout_new,
    llvm_layout_pointer_size, llvm_module_dispose, llvm_module_new, llvm_module_set_layout,
    llvm_target_create_machine, llvm_target_dispose_machine, llvm_target_emit_object,
    llvm_target_lookup,
};
use crate::pkg::{PackageName, PackageVersion, PrimitiveTarget, TargetOs};
use std::ffi::{CStr, CString};
use std::ptr::null;
use thiserror::Error;

mod block;
mod builder;
mod func;
mod resolver;
mod ty;

/// A context for code generation.
///
/// Each [`Codegen`] can output only one binary.
pub struct Codegen<'a> {
    module: *mut crate::ffi::LlvmModule,
    llvm: *mut crate::ffi::LlvmContext,
    layout: *mut crate::ffi::LlvmLayout,
    machine: *mut crate::ffi::LlvmMachine,
    pkg: &'a PackageName,
    version: &'a PackageVersion,
    target: &'static PrimitiveTarget,
    executable: bool,
    namespace: &'a str,
    entry: String,
    resolver: &'a TypeResolver<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new(
        pkg: &'a PackageName,
        version: &'a PackageVersion,
        target: &'static PrimitiveTarget,
        executable: bool,
        resolver: &'a TypeResolver<'a>,
    ) -> Self {
        // Get LLVM target.
        let triple = CString::new(target.to_string()).unwrap();
        let llvm = {
            let mut err = String::new();
            let ptr = unsafe { llvm_target_lookup(triple.as_ptr(), &mut err) };
            assert!(!ptr.is_null());
            ptr
        };

        // Create LLVM target machine.
        let machine = unsafe { llvm_target_create_machine(llvm, triple.as_ptr(), null(), null()) };

        // Create LLVM layout.
        let layout = unsafe { llvm_layout_new(machine) };

        // Create LLVM module.
        let llvm = unsafe { llvm_context_new() };
        let name = CString::new(pkg.as_str()).unwrap();
        let module = unsafe { llvm_module_new(llvm, name.as_ptr()) };

        unsafe { llvm_module_set_layout(module, layout) };

        Self {
            module,
            llvm,
            layout,
            machine,
            pkg,
            version,
            target,
            executable,
            namespace: "",
            entry: String::new(),
            resolver,
        }
    }

    pub fn pkg(&self) -> &'a PackageName {
        self.pkg
    }

    pub fn version(&self) -> &'a PackageVersion {
        self.version
    }

    pub fn target(&self) -> &'static PrimitiveTarget {
        self.target
    }

    pub fn executable(&self) -> bool {
        self.executable
    }

    pub fn namespace(&self) -> &'a str {
        self.namespace
    }

    pub fn set_namespace(&mut self, v: &'a str) {
        self.namespace = v;
    }

    pub fn entry(&self) -> &str {
        &self.entry
    }

    pub fn set_entry(&mut self, v: String) {
        self.entry = v;
    }

    pub fn resolver(&self) -> &'a TypeResolver<'a> {
        self.resolver
    }

    /// Returns the pointer size, in bytes.
    pub fn pointer_size(&self) -> u32 {
        unsafe { llvm_layout_pointer_size(self.layout) }
    }

    pub fn build<F: AsRef<std::path::Path>>(self, file: F) -> Result<(), BuildError> {
        // Generate entry point.
        match self.executable {
            true => self.build_main()?,
            false => match self.target.os() {
                TargetOs::Darwin => {}
                TargetOs::Linux => {}
                TargetOs::Win32 => self.build_dll_main()?,
            },
        }

        // TODO: Invoke LLVMVerifyModule.
        let mut err = String::new();
        let file = file.as_ref().to_str().unwrap();
        let file = CString::new(file).unwrap();

        if !unsafe { llvm_target_emit_object(self.machine, self.module, file.as_ptr(), &mut err) } {
            Err(BuildError::EmitObjectFailed(err))
        } else {
            Ok(())
        }
    }

    fn build_main(&self) -> Result<(), BuildError> {
        if self.entry.is_empty() {
            return Err(BuildError::NoEntryPoint);
        }

        // Get exit function.
        let name = CStr::from_bytes_with_nul(b"exit\0").unwrap();
        let exit = match LlvmFunc::get(self, name) {
            Some(_) => todo!(),
            None => {
                let params = [LlvmType::I32(LlvmI32::new(self))];
                let ret = LlvmType::Void(LlvmVoid::new(self));
                let mut func = LlvmFunc::new(self, name, &params, ret);

                func.set_noreturn();
                func
            }
        };

        // Create a function.
        let name = CStr::from_bytes_with_nul(b"_main\0").unwrap();
        let ret = LlvmType::Void(LlvmVoid::new(self));
        let mut func = LlvmFunc::new(self, name, &[], ret);

        // Build body.
        let mut body = BasicBlock::new(self);
        let mut b = Builder::new(self, &mut body);

        b.call(exit.as_raw(), &[LlvmI32::new(self).get_const(0) as _]);
        b.ret_void(); // TODO: Is it possible to remove this?

        func.append(body);

        Ok(())
    }

    fn build_dll_main(&self) -> Result<(), BuildError> {
        // Build parameter list.
        let params = [
            LlvmType::Ptr(LlvmPtr::new(self, LlvmType::Void(LlvmVoid::new(self)))),
            LlvmType::U32(LlvmU32::new(self)),
            LlvmType::Ptr(LlvmPtr::new(self, LlvmType::Void(LlvmVoid::new(self)))),
        ];

        // Create a function.
        let name = CStr::from_bytes_with_nul(b"_DllMainCRTStartup\0").unwrap();
        let ret = LlvmType::I32(LlvmI32::new(self));
        let mut func = LlvmFunc::new(self, name, &params, ret);

        func.set_stdcall();

        // Build body.
        let mut body = BasicBlock::new(self);
        let mut b = Builder::new(self, &mut body);

        b.ret(LlvmI32::new(self).get_const(1) as _);

        func.append(body);

        Ok(())
    }
}

impl<'a> Drop for Codegen<'a> {
    fn drop(&mut self) {
        unsafe { llvm_module_dispose(self.module) };
        unsafe { llvm_context_dispose(self.llvm) };
        unsafe { llvm_layout_dispose(self.layout) };
        unsafe { llvm_target_dispose_machine(self.machine) };
    }
}

/// Represents an error when [`Codegen::build()`] is failed.
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("no entry point has been defined")]
    NoEntryPoint,

    #[error("{0}")]
    EmitObjectFailed(String),
}
