pub use self::meta::*;

use crate::ast::{ParseError, SourceFile};
use crate::codegen::{BuildError, Codegen, Resolver, Target};
use crate::lexer::SyntaxError;
use crate::pkg::{Arch, Package, PackageMeta};
use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::target_machine::LLVMGetDefaultTargetTriple;
use std::collections::{BTreeMap, VecDeque};
use std::ffi::{CStr, CString};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod meta;

/// A Nitro project.
pub struct Project {
    path: PathBuf,
    meta: ProjectMeta,
    sources: BTreeMap<String, SourceFile>,
}

impl Project {
    pub const SUPPORTED_TARGETS: [&str; 4] = [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "x86_64-pc-win32-msvc",
        "x86_64-unknown-linux-gnu",
    ];

    pub fn open<P: Into<PathBuf>>(path: P) -> Result<Self, ProjectOpenError> {
        // Read the project.
        let path = path.into();
        let project = path.join(".nitro");
        let data = match std::fs::read_to_string(&project) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::ReadFileFailed(project, e)),
        };

        // Load the project.
        let meta = match toml::from_str::<ProjectMeta>(&data) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::ParseTomlFailed(project, e)),
        };

        Ok(Self {
            path,
            meta,
            sources: BTreeMap::new(),
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

    pub fn build(&self) -> Result<Package, ProjectBuildError> {
        // Setup type resolver.
        let mut resolver: Resolver<'_> = Resolver::new();

        resolver.populate_project_types(&self.sources);

        // Get host target.
        let target = unsafe {
            let ptr = LLVMGetDefaultTargetTriple();
            let str = CStr::from_ptr(ptr).to_str().unwrap().to_owned();
            LLVMDisposeMessage(ptr);
            str
        };

        if !Self::SUPPORTED_TARGETS.iter().any(|&v| v == target) {
            todo!("cross-compilation");
        }

        // Build.
        let bin = Arch::new();
        let lib = Arch::new();

        self.build_for(&target, &resolver)?;

        // Setup metadata.
        let pkg = &self.meta.package;
        let meta = PackageMeta::new(pkg.name.clone(), pkg.version.clone());

        Ok(Package::new(meta, bin, lib))
    }

    fn load_source(&mut self, path: PathBuf) -> Result<(), ProjectLoadError> {
        // Parse the source.
        let source = match SourceFile::parse(path.as_path()) {
            Ok(v) => v,
            Err(e) => return Err(ProjectLoadError::ParseSourceFailed(path, e)),
        };

        // Get fully qualified type name.
        if source.ty().is_some() {
            let mut fqtn = String::new();

            for c in path.strip_prefix(&self.path).unwrap().components() {
                let name = match c {
                    std::path::Component::Normal(v) => match v.to_str() {
                        Some(v) => v,
                        None => return Err(ProjectLoadError::NonUtf8Path(path)),
                    },
                    _ => unreachable!(),
                };

                if !fqtn.is_empty() {
                    fqtn.push('.');
                }

                fqtn.push_str(name);
            }

            // Strip extension.
            fqtn.pop();
            fqtn.pop();
            fqtn.pop();

            assert!(self.sources.insert(fqtn, source).is_none());
        }

        Ok(())
    }

    fn build_for(&self, target: &str, resolver: &Resolver<'_>) -> Result<(), ProjectBuildError> {
        // Setup codegen context.
        let pkg = &self.meta.package;
        let target = Target::parse(target);
        let mut cx = Codegen::new(
            &pkg.name,
            &pkg.version,
            &target,
            CString::new(pkg.name.as_str()).unwrap(),
            resolver,
        );

        // Compile the sources.
        for (fqtn, src) in &self.sources {
            cx.set_namespace(match fqtn.rfind('.') {
                Some(i) => &fqtn[..i],
                None => "",
            });

            for im in src.impls() {
                for func in im.functions() {
                    let func = match func.build(&cx, &fqtn) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(ProjectBuildError::InvalidSyntax(src.path().to_owned(), e));
                        }
                    };
                }
            }
        }

        // Create output directory.
        let mut outputs = self.path.join(".build");

        outputs.push(target.to_llvm());

        if let Err(e) = create_dir_all(&outputs) {
            return Err(ProjectBuildError::CreateDirectoryFailed(outputs, e));
        }

        // Build the object file.
        let obj = outputs.join(format!("{}.o", self.meta.package.name));

        if let Err(e) = cx.build(&obj) {
            return Err(ProjectBuildError::BuildFailed(obj, e));
        }

        Ok(())
    }
}

/// Represents an error when a [`Project`] is failed to open.
#[derive(Debug, Error)]
pub enum ProjectOpenError {
    #[error("cannot read {0}")]
    ReadFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot parse {0}")]
    ParseTomlFailed(PathBuf, #[source] toml::de::Error),
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

    #[error("path {0} is not UTF-8")]
    NonUtf8Path(PathBuf),
}

/// Represents an error when a [`Project`] is failed to build.
#[derive(Debug, Error)]
pub enum ProjectBuildError {
    #[error("invalid syntax in {0}")]
    InvalidSyntax(PathBuf, #[source] SyntaxError),

    #[error("cannot create {0}")]
    CreateDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot build {0}")]
    BuildFailed(PathBuf, #[source] BuildError),
}
