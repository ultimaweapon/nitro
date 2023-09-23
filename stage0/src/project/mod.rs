pub use self::meta::*;

use crate::ast::{ParseError, Public, SourceFile};
use crate::codegen::{BuildError, Codegen, TypeResolver};
use crate::dep::DepResolver;
use crate::lexer::SyntaxError;
use crate::pkg::{
    Architecture, ExportedType, Library, OperatingSystem, Package, PackageMeta, Target,
};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::ptr::null;
use thiserror::Error;

mod meta;

/// A Nitro project.
pub struct Project<'a> {
    path: PathBuf,
    meta: ProjectMeta,
    sources: HashMap<String, SourceFile>,
    resolver: &'a mut DepResolver,
}

impl<'a> Project<'a> {
    pub fn open<P: Into<PathBuf>>(
        path: P,
        resolver: &'a mut DepResolver,
    ) -> Result<Self, ProjectOpenError> {
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
            sources: HashMap::new(),
            resolver,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn artifacts(&self) -> PathBuf {
        self.path.join(".build")
    }

    pub fn meta(&self) -> &ProjectMeta {
        &self.meta
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
        let mut resolver = TypeResolver::new();

        resolver.populate_project_types(&self.sources);

        // Build.
        let mut exes = HashMap::new();
        let mut libs = HashMap::new();

        for target in &Target::SUPPORTED_TARGETS {
            let bins = self.build_for(target, &resolver)?;

            if let Some(exe) = bins.exe {
                assert!(exes.insert(target.clone(), exe).is_none());
            }

            if let Some(lib) = bins.lib {
                assert!(libs.insert(target.clone(), lib).is_none());
            }
        }

        // Setup metadata.
        let pkg = self.meta.package();
        let meta = PackageMeta::new(pkg.name().to_owned(), pkg.version().clone());

        Ok(Package::new(meta, exes, libs))
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

    fn build_for(
        &self,
        target: &Target,
        resolver: &TypeResolver<'_>,
    ) -> Result<BuildOutputs, ProjectBuildError> {
        // Setup codegen context.
        let pkg = self.meta.package();
        let mut cx = Codegen::new(
            pkg.name(),
            pkg.version(),
            &target,
            CString::new(pkg.name()).unwrap(),
            resolver,
        );

        // Enumerate the sources.
        let mut lib = Library::new();

        for (fqtn, src) in &self.sources {
            // Check type condition.
            let ty = src.ty().unwrap();
            let attrs = ty.attrs();

            if let Some((_, cond)) = attrs.condition() {
                match cx.check_condition(cond) {
                    Ok(v) => {
                        if !v {
                            continue;
                        }
                    }
                    Err(e) => {
                        return Err(ProjectBuildError::InvalidSyntax(src.path().to_owned(), e));
                    }
                }
            }

            // Check if the type is public.
            let mut exp = ty
                .attrs()
                .public()
                .filter(|(_, p)| *p == Public::External)
                .map(|_| ExportedType::new(fqtn.clone()));

            // Compile implementations.
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

            // Export the type.
            if let Some(v) = exp {
                lib.add_type(v);
            }
        }

        // Create output directory.
        let mut dir = self.artifacts();

        dir.push(target.to_llvm());

        if let Err(e) = create_dir_all(&dir) {
            return Err(ProjectBuildError::CreateDirectoryFailed(dir, e));
        }

        // Build the object file.
        let obj = dir.join(format!("{}.o", self.meta.package().name()));

        if let Err(e) = cx.build(&obj, false) {
            return Err(ProjectBuildError::BuildFailed(obj, e));
        }

        // Prepare to link.
        let mut args: Vec<Cow<'static, str>> = Vec::new();
        let out = dir.join(format!(
            "{}.{}",
            pkg.name(),
            match target.os() {
                OperatingSystem::Darwin => "dylib",
                OperatingSystem::Linux => "so",
                OperatingSystem::Win32 => "dll",
            }
        ));

        let linker = match target.os() {
            OperatingSystem::Darwin => {
                args.push("-o".into());
                args.push(out.to_str().unwrap().to_owned().into());
                args.push("-arch".into());
                args.push(match target.arch() {
                    Architecture::Aarch64 => "arm64".into(),
                    Architecture::X86_64 => "x86_64".into(),
                });
                args.push("-platform_version".into());
                args.push("macos".into());
                args.push("10".into());
                args.push("11".into());
                args.push("-dylib".into());
                "ld64.lld"
            }
            OperatingSystem::Linux => {
                args.push("-o".into());
                args.push(out.to_str().unwrap().to_owned().into());
                "ld.lld"
            }
            OperatingSystem::Win32 => {
                args.push(format!("/out:{}", out.to_str().unwrap()).into());
                args.push("/dll".into());
                "lld-link"
            }
        };

        args.push(obj.to_str().unwrap().to_owned().into());

        // Link.
        if let Err(e) = Self::link(linker, &args) {
            return Err(ProjectBuildError::LinkFailed(out, e));
        }

        Ok(BuildOutputs {
            exe: None,
            lib: Some((out, lib)),
        })
    }

    fn link(linker: &str, args: &[Cow<'static, str>]) -> Result<(), LinkError> {
        // Setup arguments.
        let args: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.as_ref()).unwrap())
            .collect();

        // Run linker.
        let linker = CString::new(linker).unwrap();
        let mut args: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
        let mut err = String::new();

        args.push(null());

        if unsafe { lld_link(linker.as_ptr(), args.as_ptr(), &mut err) } {
            Ok(())
        } else {
            Err(LinkError(err.trim_end().to_owned()))
        }
    }
}

struct BuildOutputs {
    exe: Option<PathBuf>,
    lib: Option<(PathBuf, Library)>,
}

#[allow(improper_ctypes)]
extern "C" {
    fn lld_link(linker: *const c_char, args: *const *const c_char, err: &mut String) -> bool;
}

#[no_mangle]
unsafe extern "C" fn nitro_string_set(s: &mut String, v: *const c_char) {
    s.clear();
    s.push_str(CStr::from_ptr(v).to_str().unwrap());
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

    #[error("cannot link {0}")]
    LinkFailed(PathBuf, #[source] LinkError),
}

/// Represents an error when a [`Project`] is failed to link.
#[derive(Debug)]
pub struct LinkError(String);

impl Error for LinkError {}

impl Display for LinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
