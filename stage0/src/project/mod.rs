pub use self::meta::*;

use crate::ast::{ParseError, SourceFile};
use crate::codegen::{BuildError, Codegen, TypeResolver};
use crate::lexer::SyntaxError;
use crate::pkg::{
    Binary, DependencyResolver, Library, LibraryBinary, Package, PackageMeta, PackageName,
    PackageVersion, PrimitiveTarget, Target, TargetArch, TargetEnv, TargetOs, TargetResolveError,
    TargetResolver, TypeDeclaration,
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
    stubs: &'a Path,
    deps: &'a DependencyResolver,
}

impl<'a> Project<'a> {
    pub fn open<P: Into<PathBuf>>(
        path: P,
        targets: &'a TargetResolver,
        stubs: &'a Path,
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
            stubs,
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
        let meta = PackageMeta::new(pkg.name().clone(), pkg.version().clone());
        let mut exes = HashMap::new();
        let mut libs = HashMap::new();

        // Build library.
        if !self.lib.is_empty() {
            let root = self.meta.library().unwrap().sources();

            for target in PrimitiveTarget::ALL.iter().map(|t| Target::Primitive(t)) {
                // Populate type resolver with internal types.
                let mut resolver = TypeResolver::new();

                resolver.populate_internal_types(&self.lib);

                // Build.
                let br = self.build_for(root, false, &target, &self.lib, &resolver)?;
                let out = self.link_lib(&br)?;

                assert!(libs
                    .insert(
                        target,
                        Binary::new(
                            Library::new(LibraryBinary::Bundle(out), br.exports),
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
                // Populate type resolver with internal types.
                let mut resolver = TypeResolver::new();

                resolver.populate_internal_types(&self.exe);

                // Populate types from package library.
                if !libs.is_empty() {
                    // Resolve library.
                    let mut target = target.clone();
                    let lib = loop {
                        if let Some(v) = libs.get(&target) {
                            break Some(v);
                        }

                        target = match self.targets.parent(&target) {
                            Ok(v) => match v {
                                Some(v) => v,
                                None => break None,
                            },
                            Err(e) => {
                                return Err(ProjectBuildError::ResolveParentTargetFailed(
                                    target, e,
                                ));
                            }
                        };
                    };

                    // Add types to resolver.
                    if let Some(lib) = lib {
                        resolver.populate_external_types(&meta, lib.bin().types());
                    }
                }

                // Build.
                let br = self.build_for(root, true, &target, &self.exe, &resolver)?;
                let out = self.link_exe(br)?;

                assert!(exes
                    .insert(target, Binary::new(out, HashSet::new()))
                    .is_none());
            }
        }

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
        if source.has_type() {
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

    fn build_for<'b, R, S>(
        &self,
        root: R,
        exe: bool,
        target: &Target,
        sources: S,
        resolver: &TypeResolver<'b>,
    ) -> Result<BuildResult, ProjectBuildError>
    where
        R: AsRef<Path>,
        S: IntoIterator<Item = (&'b String, &'b SourceFile)>,
    {
        // Create workspace directory.
        let mut ws = root.as_ref().join(".build");

        ws.push(target.to_string());

        if let Err(e) = create_dir_all(&ws) {
            return Err(ProjectBuildError::CreateDirectoryFailed(ws, e));
        }

        // Get primitive target.
        let pt = match self.targets.primitive(&target) {
            Ok(v) => v,
            Err(e) => {
                return Err(ProjectBuildError::ResolvePrimitiveTargetFailed(
                    target.clone(),
                    e,
                ));
            }
        };

        // Compile.
        let obj = ws.join(format!("{}.o", self.meta.package().name()));
        let types = self.compile(exe, pt, sources, &obj, resolver)?;

        Ok(BuildResult {
            target: pt,
            workspace: ws,
            object: obj,
            exports: types,
        })
    }

    fn compile<'b, S, O>(
        &self,
        exe: bool,
        target: &'static PrimitiveTarget,
        sources: S,
        output: O,
        resolver: &TypeResolver<'b>,
    ) -> Result<HashSet<TypeDeclaration>, ProjectBuildError>
    where
        S: IntoIterator<Item = (&'b String, &'b SourceFile)>,
        O: AsRef<Path>,
    {
        // Setup codegen context.
        let pkg = self.meta.package();
        let mut cg = Codegen::new(pkg.name(), pkg.version(), target, exe, resolver);

        // Compile source files.
        let mut types = HashSet::new();

        for (fqtn, src) in sources {
            cg.set_namespace(match fqtn.rfind('.') {
                Some(i) => &fqtn[..i],
                None => "",
            });

            match src.build(&mut cg) {
                Ok(v) => {
                    if let Some(v) = v {
                        assert!(types.insert(v));
                    }
                }
                Err(e) => return Err(ProjectBuildError::InvalidSyntax(src.path().to_owned(), e)),
            }
        }

        // Build the object file.
        let obj = output.as_ref();

        if let Err(e) = cg.build(obj) {
            return Err(ProjectBuildError::BuildFailed(obj.to_owned(), e));
        }

        Ok(types)
    }

    fn link_exe(&self, br: BuildResult) -> Result<PathBuf, ProjectBuildError> {
        // Get output path.
        let pkg = self.meta.package();
        let out = match br.target.os() {
            TargetOs::Darwin | TargetOs::Linux => br.workspace.join(pkg.name().as_str()),
            TargetOs::Win32 => br.workspace.join(format!("{}.exe", pkg.name())),
        };

        // Build linker command.
        let mut args: Vec<Cow<'static, str>> = Vec::new();
        let linker = match br.target.os() {
            TargetOs::Darwin => {
                self.set_link_args_darwin(&mut args, br.target, &out);
                "ld64.lld"
            }
            TargetOs::Linux => {
                self.set_link_args_linux(&mut args, br.target, &out);
                args.push("--entry=main".into());
                args.push("--dynamic-linker=/lib64/ld-linux-x86-64.so.2".into());
                "ld.lld"
            }
            TargetOs::Win32 => {
                self.set_link_args_win32(&mut args, br.target, &out);
                args.push("/entry:main".into());
                "lld-link"
            }
        };

        args.push(br.object.to_str().unwrap().to_owned().into());

        // Link.
        match Self::link(linker, &args) {
            Ok(_) => Ok(out),
            Err(e) => Err(ProjectBuildError::LinkFailed(out, e)),
        }
    }

    fn link_lib(&self, br: &BuildResult) -> Result<PathBuf, ProjectBuildError> {
        // Get output path.
        let pkg = self.meta.package();
        let out = br.workspace.join(match br.target.os() {
            TargetOs::Darwin => format!("lib{}.dylib", pkg.name()),
            TargetOs::Linux => format!("lib{}.so", pkg.name()),
            TargetOs::Win32 => format!("{}.dll", pkg.name()),
        });

        // Build linker command.
        let mut args: Vec<Cow<'static, str>> = Vec::new();
        let linker = match br.target.os() {
            TargetOs::Darwin => {
                self.set_link_args_darwin(&mut args, br.target, &out);
                args.push("-dylib".into());
                "ld64.lld"
            }
            TargetOs::Linux => {
                self.set_link_args_linux(&mut args, br.target, &out);
                args.push("--shared".into());
                "ld.lld"
            }
            TargetOs::Win32 => {
                let def = br.workspace.join(format!("{}.def", pkg.name()));

                if let Err(e) =
                    Self::write_module_definition(pkg.name(), pkg.version(), &br.exports, &def)
                {
                    return Err(ProjectBuildError::CreateModuleDefinitionFailed(def, e));
                }

                self.set_link_args_win32(&mut args, br.target, &out);
                args.push("/dll".into());
                args.push(format!("/def:{}", def.to_str().unwrap()).into());
                "lld-link"
            }
        };

        args.push(br.object.to_str().unwrap().to_owned().into());

        // Link.
        match Self::link(linker, &args) {
            Ok(_) => Ok(out),
            Err(e) => Err(ProjectBuildError::LinkFailed(out, e)),
        }
    }

    fn set_link_args_darwin(
        &self,
        args: &mut Vec<Cow<'static, str>>,
        target: &'static PrimitiveTarget,
        out: &Path,
    ) {
        args.push("-o".into());
        args.push(out.to_str().unwrap().to_owned().into());
        args.push("-arch".into());
        args.push(match target.arch() {
            TargetArch::AArch64 => "arm64".into(),
            TargetArch::X86_64 => "x86_64".into(),
        });
        args.push("-platform_version".into());
        args.push("macos".into());
        args.push("10".into());
        args.push("11".into());
        args.push("-lSystem".into());
        args.push("-L".into());
        args.push(
            self.stubs
                .join("darwin")
                .into_os_string()
                .into_string()
                .unwrap()
                .into(),
        );
    }

    fn set_link_args_linux(
        &self,
        args: &mut Vec<Cow<'static, str>>,
        target: &'static PrimitiveTarget,
        out: &Path,
    ) {
        let stubs = self
            .stubs
            .join(match (target.env().unwrap(), target.arch()) {
                (TargetEnv::Gnu, TargetArch::X86_64) => "linux-gnu-x86_64",
                _ => todo!(),
            });

        args.push("-o".into());
        args.push(out.to_str().unwrap().to_owned().into());
        args.push("-l".into());
        args.push("c".into());
        args.push("-L".into());
        args.push(stubs.into_os_string().into_string().unwrap().into());
    }

    fn set_link_args_win32(
        &self,
        args: &mut Vec<Cow<'static, str>>,
        target: &'static PrimitiveTarget,
        out: &Path,
    ) {
        let stubs = self.stubs.join(match target.arch() {
            TargetArch::X86_64 => "win32-x86_64",
            _ => todo!(),
        });

        args.push(format!("/out:{}", out.to_str().unwrap()).into());
        args.push(format!("/libpath:{}", stubs.to_str().unwrap()).into());
        args.push("/defaultlib:msvcrt".into());
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
        T: IntoIterator<Item = &'b TypeDeclaration>,
    {
        // Create the file.
        let mut file = File::create(file)?;

        file.write_all(b"EXPORTS\n")?;

        // Dump public types.
        for ty in types {
            let ty = match ty {
                TypeDeclaration::Basic(v) => v,
            };

            for func in ty.funcs() {
                let name = func.mangle(Some((pkg.as_str(), ver.major())), ty.name());

                file.write_all(b"    ")?;
                file.write_all(name.as_bytes())?;
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

struct BuildResult {
    target: &'static PrimitiveTarget,
    workspace: PathBuf,
    object: PathBuf,
    exports: HashSet<TypeDeclaration>,
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
    #[error("cannot resolve primitive target of {0}")]
    ResolvePrimitiveTargetFailed(Target, #[source] TargetResolveError),

    #[error("cannot resolve parent target of {0}")]
    ResolveParentTargetFailed(Target, #[source] TargetResolveError),

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
