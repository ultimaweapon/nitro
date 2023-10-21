use crate::ast::ParseError;
use crate::ffi::llvm_init;
use crate::pkg::{
    DependencyResolver, Package, PackageName, PrimitiveTarget, Target, TargetResolver,
};
use crate::project::{Project, ProjectBuildError, ProjectLoadError};
use clap::{command, value_parser, Arg, ArgAction, ArgMatches, Command};
use std::borrow::Cow;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod ast;
mod codegen;
mod ffi;
mod lexer;
mod pkg;
mod project;
mod zstd;

fn main() -> ExitCode {
    // Parse arguments.
    let project = Arg::new("project")
        .help("Path to the project (default to current directory)")
        .value_name("PROJECT")
        .value_parser(value_parser!(PathBuf));
    let args = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("init")
                .about("Create a Nitro project in an existing directory")
                .arg(
                    Arg::new("lib")
                        .help("Create a library project (default is executable project)")
                        .long("lib")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("directory")
                        .help("The directory to create the project (default to current directory)")
                        .value_name("DIRECTORY")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Build a Nitro project")
                .arg(project.clone()),
        )
        .subcommand(
            Command::new("pack")
                .about("Create a Nitro package")
                .arg(
                    Arg::new("output")
                        .help("Path of the output file (default to NAME.npk)")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(project.clone()),
        )
        .subcommand(
            Command::new("export")
                .about("Export binaries")
                .arg(
                    Arg::new("outputs")
                        .help("Path to the directory to place the binaries")
                        .value_name("DEST")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(project),
        )
        .get_matches();

    // Get executable path.
    let exe = match std::env::current_exe() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot get path of the executable: {}.", join_nested(&e));
            return ExitCode::FAILURE;
        }
    };

    let meta = match exe.symlink_metadata() {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Cannot get metadata of {}: {}.",
                exe.display(),
                join_nested(&e)
            );
            return ExitCode::FAILURE;
        }
    };

    let exe = if !meta.is_symlink() {
        exe
    } else {
        match exe.read_link() {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "Cannot read the target of {}: {}.",
                    exe.display(),
                    join_nested(&e)
                );
                return ExitCode::FAILURE;
            }
        }
    };

    // Execute the command.
    let cx = Context {
        prefix: exe.parent().unwrap().parent().unwrap(),
        targets: TargetResolver::new(),
        deps: DependencyResolver::new(),
    };

    match args.subcommand().unwrap() {
        ("init", args) => match init(args) {
            Ok(_) => ExitCode::SUCCESS,
            Err(v) => v,
        },
        ("build", args) => match build(args, &cx) {
            Ok(_) => ExitCode::SUCCESS,
            Err(v) => v,
        },
        ("pack", args) => pack(args, &cx),
        ("export", args) => export(args, &cx),
        _ => todo!(),
    }
}

fn init(args: &ArgMatches) -> Result<(), ExitCode> {
    // Get destination directory.
    let prefix = match args.get_one::<PathBuf>("directory") {
        Some(v) => v.canonicalize().unwrap(),
        None => std::env::current_dir().unwrap(),
    };

    // Get project name.
    let name: PackageName = match prefix.file_name() {
        Some(v) => match v.to_str().unwrap().parse() {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "The name of project directory is not a valid project name: {}.",
                    join_nested(&e)
                );
                return Err(ExitCode::FAILURE);
            }
        },
        None => {
            eprintln!("The location th create the project cannot be a root of filesystem.");
            return Err(ExitCode::FAILURE);
        }
    };

    // Create Nitro.yml.
    let lib = args.get_flag("lib");
    let proj = prefix.join("Nitro.yml");
    let mut proj = match OpenOptions::new().create_new(true).write(true).open(&proj) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot create {}: {}.", proj.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    };

    writeln!(proj, "package:").unwrap();
    writeln!(proj, "  name: {name}").unwrap();
    writeln!(proj, "  version: 1.0.0").unwrap();

    if lib {
        writeln!(proj, "library:").unwrap();
        writeln!(proj, "  sources: lib").unwrap();
        init_lib(prefix.join("lib"))?;
    } else {
        writeln!(proj, "executable:").unwrap();
        writeln!(proj, "  sources: exe").unwrap();
        init_exe(prefix.join("exe"))?;
    }

    // Create .gitignore.
    let ignore = prefix.join(".gitignore");
    let mut ignore = match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&ignore)
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot create {}: {}.", ignore.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    };

    writeln!(ignore, ".build/").unwrap();

    Ok(())
}

fn init_exe(src: PathBuf) -> Result<(), ExitCode> {
    // Create source directory.
    if let Err(e) = std::fs::create_dir(&src) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("Cannot create {}: {}.", src.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    }

    // Create App.nt.
    let path = src.join("App.nt");
    let mut app = match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot create {}: {}.", path.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    };

    writeln!(app, "use nitro.Int32;").unwrap();
    writeln!(app).unwrap();
    writeln!(app, "class App;").unwrap();
    writeln!(app).unwrap();
    writeln!(app, "impl App {{").unwrap();
    writeln!(app, "    @entry").unwrap();
    writeln!(app, "    fn Main(): Int32 {{").unwrap();
    writeln!(app, "        0").unwrap();
    writeln!(app, "    }}").unwrap();
    writeln!(app, "}}").unwrap();

    Ok(())
}

fn init_lib(src: PathBuf) -> Result<(), ExitCode> {
    // Create source directory.
    if let Err(e) = std::fs::create_dir(&src) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("Cannot create {}: {}.", src.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    }

    // Create SampleClass.nt.
    let path = src.join("SampleClass.nt");
    let mut class = match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot create {}: {}.", path.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    };

    writeln!(class, "use nitro.Int32;").unwrap();
    writeln!(class).unwrap();
    writeln!(class, "@pub").unwrap();
    writeln!(class, "class SampleClass;").unwrap();
    writeln!(class).unwrap();
    writeln!(class, "impl SampleClass {{").unwrap();
    writeln!(class, "    @pub").unwrap();
    writeln!(class, "    fn SampleMethod(arg: Int32): Int32 {{").unwrap();
    writeln!(class, "        0").unwrap();
    writeln!(class, "    }}").unwrap();
    writeln!(class, "}}").unwrap();

    Ok(())
}

fn build(args: &ArgMatches, cx: &Context) -> Result<Package, ExitCode> {
    // Initialize LLVM.
    unsafe { llvm_init() };

    // Get path to the project.
    let path = match args.get_one::<PathBuf>("project") {
        Some(v) => Cow::Borrowed(v.as_path()),
        None => Cow::Owned(std::env::current_dir().unwrap()),
    };

    // Get path to stubs.
    let mut stubs = cx.prefix.join("share");

    stubs.push("nitro");
    stubs.push("stub");

    // Open the project.
    let mut project = match Project::open(path.as_ref(), &cx.targets, &stubs, &cx.deps) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot open {}: {}.", path.display(), join_nested(&e));
            return Err(ExitCode::FAILURE);
        }
    };

    // Load the project.
    if let Err(e) = project.load() {
        match e {
            ProjectLoadError::ParseSourceFailed(p, ParseError::ParseFailed(e)) => {
                eprintln!("{}: {}", p.display(), e);
            }
            e => eprintln!(
                "Cannot load {}: {}.",
                project.path().display(),
                join_nested(&e)
            ),
        }

        return Err(ExitCode::FAILURE);
    }

    // Build the project.
    project.build().map_err(|e| {
        match e {
            ProjectBuildError::InvalidSyntax(p, e) => {
                eprintln!("{}: {}", p.display(), e);
            }
            ProjectBuildError::BuildFailed(p, e) => {
                eprintln!("Cannot build {}: {}", p.display(), e);
            }
            e => eprintln!("{}: {}", project.path().display(), join_nested(&e)),
        }

        ExitCode::FAILURE
    })
}

fn pack(args: &ArgMatches, cx: &Context) -> ExitCode {
    // Build.
    let pkg = match build(args, cx) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // Get output path.
    let path = match args.get_one::<PathBuf>("output") {
        Some(v) => Cow::Borrowed(v.as_path()),
        None => {
            let mut p = std::env::current_dir().unwrap();
            p.push(format!("{}.npk", pkg.meta().name()));
            Cow::Owned(p)
        }
    };

    // Pack.
    if let Err(e) = pkg.pack(path.as_ref()) {
        eprintln!("Cannot pack {}: {}.", path.display(), join_nested(&e));
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn export(args: &ArgMatches, cx: &Context) -> ExitCode {
    // Build.
    let pkg = match build(args, cx) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // Export the binaries.
    let tartet = Target::Primitive(PrimitiveTarget::current());
    let path = args.get_one::<PathBuf>("outputs").unwrap();

    if let Err(e) = pkg.export(path, &tartet, &cx.targets, &cx.deps) {
        eprintln!(
            "Cannot export the binaries to {}: {}.",
            path.display(),
            join_nested(&e)
        );
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn join_nested(mut e: &dyn Error) -> String {
    let mut m = e.to_string();

    while let Some(v) = e.source() {
        use std::fmt::Write;
        write!(m, " -> {v}").unwrap();
        e = v;
    }

    m
}

struct Context<'a> {
    prefix: &'a Path,
    targets: TargetResolver,
    deps: DependencyResolver,
}
