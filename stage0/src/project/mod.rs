use crate::ast::{ParseError, SourceFile};
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
    LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef};
use std::collections::VecDeque;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A Nitro project.
pub struct Project {
    path: PathBuf,
    name: String,
    sources: Vec<SourceFile>,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    llvm: LLVMContextRef,
}

impl Project {
    pub fn open<P: Into<PathBuf>>(path: P) -> Result<Self, ProjectOpenError> {
        // Load the project.
        let path = path.into();
        let project = path.join(".nitro");
        let data = std::fs::read_to_string(&project)
            .map_err(|e| ProjectOpenError::ReadFileFailed(project.clone(), e))?
            .parse::<toml::Table>()
            .map_err(|e| ProjectOpenError::ParseTomlFailed(project.clone(), e))?;

        // Get project name.
        let pkg = data
            .get("package")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ProjectOpenError::UndefinedPackage(project.clone()))?;
        let name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProjectOpenError::UndefinedName(project.clone()))?
            .to_owned();

        // Setup LLVM.
        let module = CString::new(name.as_str()).unwrap();
        let llvm = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module.as_ptr(), llvm) };
        let builder = unsafe { LLVMCreateBuilderInContext(llvm) };

        Ok(Self {
            path,
            name,
            sources: Vec::new(),
            builder,
            module,
            llvm,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&mut self) -> Result<(), ProjectLoadError> {
        // Enumerate all project files.
        let mut jobs = VecDeque::from([self.path.clone()]);

        while let Some(path) = jobs.pop_front() {
            // Enumerate files.
            let items = match std::fs::read_dir(&path) {
                Ok(v) => v,
                Err(e) => return Err(ProjectLoadError::EnumerateFilesFailed(path, e)),
            };

            for item in items {
                let item = item.map_err(|e| ProjectLoadError::AccessFileFailed(path.clone(), e))?;

                // Get metadata.
                let path = item.path();
                let meta = match std::fs::metadata(&path) {
                    Ok(v) => v,
                    Err(e) => return Err(ProjectLoadError::GetMetadataFailed(path, e)),
                };

                // Check if directory.
                if meta.is_dir() {
                    jobs.push_back(path);
                    continue;
                }

                // Get file extension.
                let ext = match path.extension() {
                    Some(v) => v,
                    None => continue,
                };

                // Check file type.
                if ext == "nt" {
                    self.load_source(path)?;
                }
            }
        }

        Ok(())
    }

    fn load_source(&mut self, path: PathBuf) -> Result<(), ProjectLoadError> {
        let source = match SourceFile::parse(path.as_path()) {
            Ok(v) => v,
            Err(e) => return Err(ProjectLoadError::ParseSourceFailed(path, e)),
        };

        self.sources.push(source);
        Ok(())
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) };
        unsafe { LLVMDisposeModule(self.module) };
        unsafe { LLVMContextDispose(self.llvm) };
    }
}

/// Represents an error when a [`Project`] is failed to open.
#[derive(Debug, Error)]
pub enum ProjectOpenError {
    #[error("cannot read {0}")]
    ReadFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot parse {0}")]
    ParseTomlFailed(PathBuf, #[source] toml::de::Error),

    #[error("no table package in the {0}")]
    UndefinedPackage(PathBuf),

    #[error("no package name has been defined in {0}")]
    UndefinedName(PathBuf),
}

/// Represents an error when a [`Project`] is failed to load.
#[derive(Debug, Error)]
pub enum ProjectLoadError {
    #[error("cannot enumerate files in {0}")]
    EnumerateFilesFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot access the file in {0}")]
    AccessFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot get metadata of {0}")]
    GetMetadataFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot parse {0}")]
    ParseSourceFailed(PathBuf, #[source] ParseError),
}
