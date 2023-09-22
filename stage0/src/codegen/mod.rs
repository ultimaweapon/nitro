pub use self::block::*;
pub use self::builder::*;
pub use self::func::*;
pub use self::resolver::*;
pub use self::ty::*;

use crate::ast::{Expression, Path, Representation, SourceFile, Struct, TypeDefinition, Use};
use crate::ffi::{
    llvm_context_dispose, llvm_context_new, llvm_layout_dispose, llvm_layout_new,
    llvm_layout_pointer_size, llvm_module_dispose, llvm_module_new, llvm_module_set_layout,
    llvm_target_create_machine, llvm_target_dispose_machine, llvm_target_emit_object,
    llvm_target_lookup,
};
use crate::lexer::SyntaxError;
use crate::pkg::{OperatingSystem, PackageVersion, Target};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::ptr::null;

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
    pkg: &'a str,
    version: &'a PackageVersion,
    target: &'a Target,
    namespace: &'a str,
    resolver: &'a Resolver<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new<M>(
        pkg: &'a str,
        version: &'a PackageVersion,
        target: &'a Target,
        module: M,
        resolver: &'a Resolver<'a>,
    ) -> Self
    where
        M: AsRef<CStr>,
    {
        let module = module.as_ref();

        // Get LLVM target.
        let triple = CString::new(target.to_llvm()).unwrap();
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
        let module = unsafe { llvm_module_new(llvm, module.as_ptr()) };

        unsafe { llvm_module_set_layout(module, layout) };

        Self {
            module,
            llvm,
            layout,
            machine,
            pkg,
            version,
            target,
            namespace: "",
            resolver,
        }
    }

    pub fn set_namespace(&mut self, v: &'a str) {
        self.namespace = v;
    }

    /// Returns the pointer size, in bytes.
    pub fn pointer_size(&self) -> u32 {
        unsafe { llvm_layout_pointer_size(self.layout) }
    }

    pub fn check_condition(&self, cond: &[Expression]) -> Result<bool, SyntaxError> {
        // Get first expression.
        let mut expr = cond.iter();
        let lhs = match expr.next().unwrap() {
            Expression::Value(v) => v,
            e => return Err(SyntaxError::new(e.span(), "expect an identifier")),
        };

        // Get second expression.
        let os = self.target.os();
        let (equal, span) = match expr.next() {
            Some(Expression::NotEqual(f, s)) => (false, f.span() + s.span()),
            Some(Expression::Equal(f, s)) => (true, f.span() + s.span()),
            Some(e) => return Err(SyntaxError::new(e.span(), "unsupported expression")),
            None => match lhs.value() {
                "unix" => match os {
                    OperatingSystem::Darwin | OperatingSystem::Linux => return Ok(true),
                    OperatingSystem::Win32 => return Ok(false),
                },
                "win32" => match os {
                    OperatingSystem::Darwin | OperatingSystem::Linux => return Ok(false),
                    OperatingSystem::Win32 => return Ok(true),
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
                    OperatingSystem::Darwin | OperatingSystem::Linux => false,
                    OperatingSystem::Win32 => true,
                },
                _ => todo!(),
            }
        } else {
            match rhs.value() {
                "windows" => match os {
                    OperatingSystem::Darwin | OperatingSystem::Linux => true,
                    OperatingSystem::Win32 => false,
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

    pub fn build<F: AsRef<std::path::Path>>(self, file: F, exe: bool) -> Result<(), BuildError> {
        // Generate DllMain for DLL on Windows.
        if self.target.os() == OperatingSystem::Win32 && !exe {
            self.build_dll_main()?;
        }

        // TODO: Invoke LLVMVerifyModule.
        let mut err = String::new();
        let file = file.as_ref().to_str().unwrap();
        let file = CString::new(file).unwrap();

        if !unsafe { llvm_target_emit_object(self.machine, self.module, file.as_ptr(), &mut err) } {
            Err(BuildError(err))
        } else {
            Ok(())
        }
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
        unsafe { llvm_module_dispose(self.module) };
        unsafe { llvm_context_dispose(self.llvm) };
        unsafe { llvm_layout_dispose(self.layout) };
        unsafe { llvm_target_dispose_machine(self.machine) };
    }
}

/// Represents an error when [`Codegen::build()`] is failed.
#[derive(Debug)]
pub struct BuildError(String);

impl Error for BuildError {}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
