pub use self::meta::*;

use crate::ast::{ParseError, SourceFile};
use crate::codegen::{Codegen, Resolver};
use crate::lexer::SyntaxError;
use crate::pkg::{Arch, Package, PackageMeta};
use std::collections::{BTreeMap, VecDeque};
use std::ffi::CString;
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

    pub fn build(&mut self) -> Result<Package, ProjectBuildError> {
        // Setup type resolver.
        let resolver: Resolver<'_> = Resolver::new();

        // Setup codegen context.
        let pkg = &self.meta.package;
        let mut cx = Codegen::new(
            &pkg.name,
            &pkg.version,
            CString::new(pkg.name.as_str()).unwrap(),
            resolver,
        );

        // Compile the sources.
        let bin = Arch::new();
        let lib = Arch::new();

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

        // Setup metadata.
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
}
