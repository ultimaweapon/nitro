use crate::ast::{ParseError, SourceFile};
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A Pluto project.
pub struct Project {
    path: PathBuf,
    name: String,
    sources: Vec<SourceFile>,
    module: LLVMModuleRef,
    llvm: LLVMContextRef,
}

impl Project {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, ProjectError> {
        // Load the project.
        let path = path.into();
        let project = path.join(".nitro");
        let data = std::fs::read_to_string(&project)
            .map_err(|e| ProjectError::ReadFileFailed(project.clone(), e))?
            .parse::<toml::Table>()
            .map_err(|e| ProjectError::ParseTomlFailed(project.clone(), e))?;

        // Get project name.
        let pkg = data
            .get("package")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ProjectError::UndefinedPackage(project.clone()))?;
        let name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProjectError::UndefinedName(project.clone()))?
            .to_owned();

        // Setup LLVM.
        let module = CString::new(name.as_str()).unwrap();
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ptr(), llvm) };

        Ok(Self {
            path,
            name,
            sources: Vec::new(),
            module,
            llvm,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn parse_source<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ParseError> {
        self.sources.push(SourceFile::parse(path.as_ref())?);
        Ok(())
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
    }
}

/// Represents an error when [`Project`] is failed to load.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("cannot read {0}")]
    ReadFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot parse {0}")]
    ParseTomlFailed(PathBuf, #[source] toml::de::Error),

    #[error("no table package in the {0}")]
    UndefinedPackage(PathBuf),

    #[error("no package name has been defined in {0}")]
    UndefinedName(PathBuf),
}
