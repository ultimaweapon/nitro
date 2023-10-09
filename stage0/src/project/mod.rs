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
    exe: HashMap<String, SourceFile>,
    lib: HashMap<String, SourceFile>,
    targets: &'a TargetResolver,
    deps: &'a DependencyResolver,
}

impl<'a> Project<'a> {
    pub fn open<P: Into<PathBuf>>(
        path: P,
        targets: &'a TargetResolver,
        deps: &'a DependencyResolver,
    ) -> Result<Self, ProjectOpenError> {
        // Open the project.
        let path = path.into();
        let project = path.join("Nitro.yml");
        let file = match File::open(&project) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::OpenFileFailed(project, e)),
        };

        // Load the project.
        let meta: ProjectMeta = match serde_yaml::from_reader(file) {
            Ok(v) => v,
            Err(e) => return Err(ProjectOpenError::ParseProjectFailed(project, e)),
        };

        if meta.executable().is_none() && meta.library().is_none() {
            return Err(ProjectOpenError::MissingBinary(project));
        }

        Ok(Self {
            path,
            meta,
            exe: HashMap::new(),
            lib: HashMap::new(),
            targets,
            deps,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&mut self) -> Result<(), ProjectLoadError> {
        // Load executable sources.
        if let Some(bin) = self.meta.executable() {
            let root = bin.sources();

            self.exe = if root.is_absolute() {
                Self::load_sources(root)?
            } else {
                Self::load_sources(self.path.join(root))?
            };
        }

        // Load library sources.
        if let Some(bin) = self.meta.library() {
            let root = bin.sources();

            self.lib = if root.is_absolute() {
                Self::load_sources(root)?
            } else {
                Self::load_sources(self.path.join(root))?
            };
        }

        Ok(())
    }

    pub fn build(&self) -> Result<Package, ProjectBuildError> {
        let pkg = self.meta.package();
        let mut exes = HashMap::new();
        let mut libs = HashMap::new();

        // Build library.
        if !self.lib.is_empty() {
            let root = self.meta.library().unwrap().sources();

            for target in PrimitiveTarget::ALL.iter().map(|t| Target::Primitive(t)) {
                // Create output directory.
                let mut dir = root.join(".build");

                dir.push(target.to_string());

                if let Err(e) = create_dir_all(&dir) {
                    return Err(ProjectBuildError::CreateDirectoryFailed(dir, e));
                }

                // Get primitive target.
                let pt = match self.targets.resolve(&target) {
                    Ok(v) => v,
                    Err(e) => return Err(ProjectBuildError::ResolveTargetFailed(target, e)),
                };

                // Setup type resolver.
                let mut resolver = TypeResolver::new();

                resolver.populate_project_types(&self.lib);

                // Compile.
                let obj = dir.join(format!("{}.o", pkg.name()));
                let types = self.compile(false, pt, &self.lib, &obj, &mut resolver)?;

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

                assert!(libs
                    .insert(
                        target,
                        Binary::new(
                            Library::new(LibraryBinary::Bundle(out), types),
                            HashSet::new()
                        )
                    )
                    .is_none());
            }
        }

        // Build executable.
        if !self.exe.is_empty() {
            let root = self.meta.executable().unwrap().sources();

            for target in PrimitiveTarget::ALL.iter().map(|t| Target::Primitive(t)) {
                // Create output directory.
                let mut dir = root.join(".build");

                dir.push(target.to_string());

                if let Err(e) = create_dir_all(&dir) {
                    return Err(ProjectBuildError::CreateDirectoryFailed(dir, e));
                }

                // Get primitive target.
                let pt = match self.targets.resolve(&target) {
                    Ok(v) => v,
                    Err(e) => return Err(ProjectBuildError::ResolveTargetFailed(target, e)),
                };

                // Setup type resolver.
                let mut resolver = TypeResolver::new();

                resolver.populate_project_types(&self.exe);

                // Compile.
                let obj = dir.join(format!("{}.o", pkg.name()));

                self.compile(true, pt, &self.exe, &obj, &mut resolver)?;

                // Prepare to link.
                let mut args: Vec<Cow<'static, str>> = Vec::new();
                let out = match pt.os() {
                    TargetOs::Darwin | TargetOs::Linux => dir.join(pkg.name().as_str()),
                    TargetOs::Win32 => dir.join(format!("{}.exe", pkg.name())),
                };

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
                        "ld64.lld"
                    }
                    TargetOs::Linux => {
                        args.push("-o".into());
                        args.push(out.to_str().unwrap().to_owned().into());
                        "ld.lld"
                    }
                    TargetOs::Win32 => {
                        args.push(format!("/out:{}", out.to_str().unwrap()).into());
                        "lld-link"
                    }
                };

                args.push(obj.to_str().unwrap().to_owned().into());

                // Link.
                if let Err(e) = Self::link(linker, &args) {
                    return Err(ProjectBuildError::LinkFailed(out, e));
                }

                assert!(exes
                    .insert(target, Binary::new(out, HashSet::new()))
                    .is_none());
            }
        }

        // Construct the package.
        let meta = PackageMeta::new(pkg.name().clone(), pkg.version().clone());

        Ok(Package::new(meta, exes, libs))
    }

    fn load_sources<'b, R>(root: R) -> Result<HashMap<String, SourceFile>, ProjectLoadError>
    where
        R: AsRef<Path> + 'b,
    {
        // Enumerate source files.
        let root = root.as_ref();
        let mut sources = HashMap::new();
        let mut dirs = VecDeque::from([Cow::Borrowed(root)]);

        while let Some(dir) = dirs.pop_front() {
            // Enumerate items.
            let items = match std::fs::read_dir(&dir) {
                Ok(v) => v,
                Err(e) => return Err(ProjectLoadError::EnumerateFilesFailed(dir.into_owned(), e)),
            };

            for item in items {
                // Unwrap the item.
                let item = match item {
                    Ok(v) => v,
                    Err(e) => return Err(ProjectLoadError::AccessFileFailed(dir.into_owned(), e)),
                };

                // Get metadata.
                let path = item.path();
                let meta = match std::fs::metadata(&path) {
                    Ok(v) => v,
                    Err(e) => return Err(ProjectLoadError::GetMetadataFailed(path, e)),
                };

                // Check if directory.
                if meta.is_dir() {
                    dirs.push_back(Cow::Owned(path));
                    continue;
                }

                // Get file extension.
                let ext = match path.extension() {
                    Some(v) => v,
                    None => continue,
                };

                // Check file type.
                if ext == "nt" {
                    Self::load_source(root, path, &mut sources)?;
                }
            }
        }

        Ok(sources)
    }

    fn load_source<R>(
        root: R,
        path: PathBuf,
        set: &mut HashMap<String, SourceFile>,
    ) -> Result<(), ProjectLoadError>
    where
        R: AsRef<Path>,
    {
        // Parse the source.
        let source = match SourceFile::parse(path.as_path()) {
            Ok(v) => v,
            Err(e) => return Err(ProjectLoadError::ParseSourceFailed(path, e)),
        };

        // Get fully qualified type name.
        if source.ty().is_some() {
            let mut fqtn = String::new();

            for c in path.strip_prefix(root).unwrap().components() {
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

            assert!(set.insert(fqtn, source).is_none());
        }

        Ok(())
    }

    fn compile<'b, S, O>(
        &self,
        exe: bool,
        target: &'static PrimitiveTarget,
        sources: S,
        output: O,
        resolver: &'b TypeResolver<'_>,
    ) -> Result<HashSet<ExportedType>, ProjectBuildError>
    where
        S: IntoIterator<Item = (&'b String, &'b SourceFile)>,
        O: AsRef<Path>,
    {
        // Setup codegen context.
        let pkg = self.meta.package();
        let mut cx = Codegen::new(pkg.name(), pkg.version(), target, resolver);

        // Compile source files.
        let mut types = HashSet::new();

        for (fqtn, src) in sources {
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
                    match func.build(&cx, &fqtn, src.uses()) {
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

        // Build the object file.
        let obj = output.as_ref();

        if let Err(e) = cx.build(obj, exe) {
            return Err(ProjectBuildError::BuildFailed(obj.to_owned(), e));
        }

        Ok(types)
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
    #[error("cannot open {0}")]
    OpenFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot parse {0}")]
    ParseProjectFailed(PathBuf, #[source] serde_yaml::Error),

    #[error("{0} must contain at least executable or library definition")]
    MissingBinary(PathBuf),
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
