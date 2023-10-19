use crate::ast::ParseError;
use crate::ffi::llvm_init;
use crate::pkg::{DependencyResolver, Package, PrimitiveTarget, Target, TargetResolver};
use crate::project::{Project, ProjectBuildError, ProjectLoadError};
use clap::{command, value_parser, Arg, ArgMatches, Command};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Write;
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
        ("build", args) => match build(args, &cx) {
            Ok(_) => ExitCode::SUCCESS,
            Err(v) => v,
        },
        ("pack", args) => pack(args, &cx),
        ("export", args) => export(args, &cx),
        _ => todo!(),
    }
}

fn build(args: &ArgMatches, cx: &Context) -> Result<Package, ExitCode> {
    // Initialize LLVM.
    unsafe { llvm_init() };

    // Get path to the project.
    let path = match args.get_one::<PathBuf>("project") {
        Some(v) => Cow::Borrowed(v),
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
