pub use self::meta::*;

use crate::ast::{ParseError, Public, SourceFile};
use crate::codegen::{BuildError, Codegen, TypeResolver};
use crate::lexer::SyntaxError;
use crate::pkg::{
    Binary, DependencyResolver, ExportedFunc, ExportedType, FunctionParam, Library, LibraryBinary,
    Package, PackageMeta, PackageName, PackageVersion, PrimitiveTarget, Target, TargetArch,
    TargetOs, TargetResolveError, TargetResolver, Type,
};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr::null;
use thiserror::Error;

mod meta;

/// A Nitro project.
pub struct Project<'a> {
    path: PathBuf,
    meta: ProjectMeta,
    sources: HashMap<String, SourceFile>,
    deps: &'a DependencyResolver,
}

impl<'a> Project<'a> {
    pub fn open<P: Into<PathBuf>>(
        path: P,
        deps: &'a mut DependencyResolver,
    ) -> Result<Self, ProjectOpenError> {
        // Read the project.
        let path = path.into();
        let project = path.join(".nitro");
        let data = match std::fs::read_to_string(&project) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::ReadFileFailed(project, e)),
        };

        // Load the project.
        let meta = match serde_yaml::from_str::<ProjectMeta>(&data) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::ParseTomlFailed(project, e)),
        };

        Ok(Self {
            path,
            meta,
            sources: HashMap::new(),
            deps,
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
        let mut types = TypeResolver::new();

        types.populate_project_types(&self.sources);

        // Build.
        let mut targets = TargetResolver::new();
        let mut exes = HashMap::new();
        let mut libs = HashMap::new();

        for target in PrimitiveTarget::ALL.iter().map(|t| Target::Primitive(t)) {
            let (exe, lib) = self.build_for(&target, &mut types, &mut targets)?;

            if let Some(exe) = exe {
                assert!(exes.insert(target.clone(), exe).is_none());
            }

            if let Some(lib) = lib {
                assert!(libs.insert(target.clone(), lib).is_none());
            }
        }

        // Setup metadata.
        let pkg = self.meta.package();
        let meta = PackageMeta::new(pkg.name().clone(), pkg.version().clone());

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
        types: &mut TypeResolver<'_>,
        targets: &mut TargetResolver,
    ) -> Result<(Option<Binary<PathBuf>>, Option<Binary<Library>>), ProjectBuildError> {
        // Get primitive target.
        let pt = match targets.resolve(target) {
            Ok(v) => v,
            Err(e) => return Err(ProjectBuildError::ResolveTargetFailed(target.clone(), e)),
        };

        // Setup codegen context.
        let pkg = self.meta.package();
        let mut cx = Codegen::new(pkg.name(), pkg.version(), pt, types);

        // Compile source files.
        let mut types = HashSet::new();

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
                    // Compile the function.
                    match func.build(&cx, &fqtn) {
                        Ok(v) => {
                            if v.is_none() {
                                continue;
                            }
                        }
                        Err(e) => {
                            return Err(ProjectBuildError::InvalidSyntax(src.path().to_owned(), e));
                        }
                    }

                    // Export the function.
                    let public = func
                        .attrs()
                        .public()
                        .filter(|(_, p)| *p == Public::External);

                    if let Some((exp, _)) = exp.as_mut().zip(public) {
                        let name = func.name().value().to_owned();
                        let params = func
                            .params()
                            .iter()
                            .map(|p| {
                                FunctionParam::new(
                                    p.name().value().to_owned(),
                                    p.ty().to_export(&cx, &[]),
                                )
                            })
                            .collect();
                        let ret = match func.ret() {
                            Some(v) => v.to_export(&cx, &[]),
                            None => Type::Unit(0),
                        };

                        exp.add_func(ExportedFunc::new(name, params, ret));
                    }
                }
            }

            // Export the type.
            if let Some(v) = exp {
                assert!(types.insert(v));
            }
        }

        // Create output directory.
        let mut dir = self.artifacts();

        dir.push(target.to_string());

        if let Err(e) = create_dir_all(&dir) {
            return Err(ProjectBuildError::CreateDirectoryFailed(dir, e));
        }

        // Build the object file.
        let obj = dir.join(format!("{}.o", pkg.name()));

        if let Err(e) = cx.build(&obj, false) {
            return Err(ProjectBuildError::BuildFailed(obj, e));
        }

        // Prepare to link.
        let mut args: Vec<Cow<'static, str>> = Vec::new();
        let out = dir.join(match pt.os() {
            TargetOs::Darwin => format!("lib{}.dylib", pkg.name()),
            TargetOs::Linux => format!("lib{}.so", pkg.name()),
            TargetOs::Win32 => format!("{}.dll", pkg.name()),
        });

        let linker = match pt.os() {
            TargetOs::Darwin => {
                args.push("-o".into());
                args.push(out.to_str().unwrap().to_owned().into());
                args.push("-arch".into());
                args.push(match pt.arch() {
                    TargetArch::AArch64 => "arm64".into(),
                    TargetArch::X86_64 => "x86_64".into(),
                });
                args.push("-platform_version".into());
                args.push("macos".into());
                args.push("10".into());
                args.push("11".into());
                args.push("-dylib".into());
                "ld64.lld"
            }
            TargetOs::Linux => {
                args.push("-o".into());
                args.push(out.to_str().unwrap().to_owned().into());
                args.push("--shared".into());
                "ld.lld"
            }
            TargetOs::Win32 => {
                let def = dir.join(format!("{}.def", pkg.name()));

                if let Err(e) =
                    Self::write_module_definition(pkg.name(), pkg.version(), &types, &def)
                {
                    return Err(ProjectBuildError::CreateModuleDefinitionFailed(def, e));
                }

                args.push(format!("/out:{}", out.to_str().unwrap()).into());
                args.push("/dll".into());
                args.push(format!("/def:{}", def.to_str().unwrap()).into());
                "lld-link"
            }
        };

        args.push(obj.to_str().unwrap().to_owned().into());

        // Link.
        if let Err(e) = Self::link(linker, &args) {
            return Err(ProjectBuildError::LinkFailed(out, e));
        }

        Ok((
            None,
            Some(Binary::new(
                Library::new(LibraryBinary::Bundle(out), types),
                HashSet::new(),
            )),
        ))
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

    fn write_module_definition<'b, F, T>(
        pkg: &PackageName,
        ver: &PackageVersion,
        types: T,
        file: F,
    ) -> Result<(), std::io::Error>
    where
        F: AsRef<Path>,
        T: IntoIterator<Item = &'b ExportedType>,
    {
        // Create the file.
        let mut file = File::create(file)?;

        file.write_all(b"EXPORTS\n")?;

        // Dumpt public types.
        for ty in types {
            for func in ty.funcs() {
                file.write_all(b"    ")?;
                file.write_all(func.mangle(pkg, ver, ty).as_bytes())?;
                file.write_all(b"\n")?;
            }
        }

        Ok(())
    }
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
    ParseTomlFailed(PathBuf, #[source] serde_yaml::Error),
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
    #[error("cannot resolve end target of {0}")]
    ResolveTargetFailed(Target, #[source] TargetResolveError),

    #[error("invalid syntax in {0}")]
    InvalidSyntax(PathBuf, #[source] SyntaxError),

    #[error("cannot create {0}")]
    CreateDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot build {0}")]
    BuildFailed(PathBuf, #[source] BuildError),

    #[error("cannot create module defition at {0}")]
    CreateModuleDefinitionFailed(PathBuf, #[source] std::io::Error),

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
