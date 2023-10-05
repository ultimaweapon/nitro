pub use self::block::*;
pub use self::builder::*;
pub use self::func::*;
pub use self::resolver::*;
pub use self::ty::*;

use crate::ast::{Expression, Function, Path, TypeName, Use};
use crate::ffi::{
    llvm_context_dispose, llvm_context_new, llvm_layout_dispose, llvm_layout_new,
    llvm_layout_pointer_size, llvm_module_dispose, llvm_module_new, llvm_module_set_layout,
    llvm_target_create_machine, llvm_target_dispose_machine, llvm_target_emit_object,
    llvm_target_lookup,
};
use crate::lexer::SyntaxError;
use crate::pkg::{
    ExportedType, PackageMeta, PackageName, PackageVersion, PrimitiveTarget, TargetOs,
};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter, Write};
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
    pkg: &'a PackageName,
    version: &'a PackageVersion,
    target: &'static PrimitiveTarget,
    namespace: &'a str,
    resolver: &'a TypeResolver<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new(
        pkg: &'a PackageName,
        version: &'a PackageVersion,
        target: &'static PrimitiveTarget,
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
            None => {
                return Ok(if lhs.value() == "unix" {
                    os.is_unix()
                } else {
                    lhs.value() == os.name()
                })
            }
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
            rhs.value() == os.name()
        } else {
            rhs.value() != os.name()
        };

        if expr.next().is_some() {
            todo!()
        }

        Ok(res)
    }

    pub fn mangle(&self, uses: &[Use], ty: &str, mem: &Function) -> Result<String, SyntaxError> {
        let mut buf = String::new();
        let pkg = self.pkg.as_str();

        // Package name.
        write!(buf, "_N{}{}", pkg.len(), pkg).unwrap();

        // Package version.
        if self.version.major() != 0 {
            write!(buf, "V{}", self.version.major()).unwrap();
        }

        // Type name.
        write!(buf, "T").unwrap();

        for p in ty.split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
        }

        // Function name.
        let name = mem.name().value();

        write!(buf, "F{}{}", name.len(), name).unwrap();
        write!(buf, "0").unwrap(); // C calling convention.

        // Return type.
        match mem.ret() {
            Some(v) => {
                for _ in v.prefixes() {
                    buf.push('P');
                }

                match v.name() {
                    TypeName::Unit(_, _) => buf.push('U'),
                    TypeName::Never(_) => buf.push('N'),
                    TypeName::Ident(p) => match self.resolve(uses, p) {
                        Some((n, t)) => match t {
                            ResolvedType::Project(_) => Self::mangle_self(&mut buf, &n),
                            ResolvedType::External((p, t)) => Self::mangle_ext(&mut buf, p, t),
                        },
                        None => return Err(SyntaxError::new(p.span(), "undefined type")),
                    },
                }
            }
            None => buf.push('U'),
        }

        // Parameters.
        for p in mem.params().iter().map(|p| p.ty()) {
            for _ in p.prefixes() {
                buf.push('P');
            }

            match p.name() {
                TypeName::Unit(_, _) => buf.push('U'),
                TypeName::Never(_) => buf.push('N'),
                TypeName::Ident(p) => match self.resolve(uses, p) {
                    Some((n, t)) => match t {
                        ResolvedType::Project(_) => Self::mangle_self(&mut buf, &n),
                        ResolvedType::External((p, t)) => Self::mangle_ext(&mut buf, p, t),
                    },
                    None => return Err(SyntaxError::new(p.span(), "undefined type")),
                },
            }
        }

        Ok(buf)
    }

    pub fn resolve(&self, uses: &[Use], name: &Path) -> Option<(String, &ResolvedType<'a>)> {
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

        // Resolve type.
        let ty = self.resolver.resolve(&name)?;

        Some((name, ty))
    }

    pub fn build<F: AsRef<std::path::Path>>(self, file: F, exe: bool) -> Result<(), BuildError> {
        // Generate DllMain for DLL on Windows.
        if self.target.os() == TargetOs::Win32 && !exe {
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

    fn mangle_self(buf: &mut String, path: &str) {
        buf.push('S');

        for p in path.strip_prefix("self.").unwrap().split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
        }
    }

    fn mangle_ext(buf: &mut String, pkg: &PackageMeta, ty: &ExportedType) {
        let name = pkg.name().as_str();
        let ver = pkg.version().major();

        write!(buf, "E{}{}", name.len(), name).unwrap();

        if ver != 0 {
            write!(buf, "V{ver}T",).unwrap();
        } else {
            buf.push('T');
        }

        for p in ty.name().split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
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
