use crate::ast::ParseError;
use crate::ffi::llvm_init;
use crate::pkg::{DependencyResolver, TargetResolver};
use crate::project::{Project, ProjectBuildError, ProjectLoadError};
use clap::{command, value_parser, Arg, ArgMatches, Command};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Write;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod codegen;
mod ffi;
mod lexer;
mod pkg;
mod project;
mod ty;
mod zstd;

fn main() -> ExitCode {
    // Parse arguments.
    let args = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("build")
                .about("Build a Nitro project")
                .arg(
                    Arg::new("export")
                        .help("Export all binaries to the specified directory")
                        .short('o')
                        .long("outputs")
                        .value_name("DIRECTORY")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("package")
                        .help("Export a package to the specified file")
                        .long("pkg")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("project")
                        .help("Path to the project (default to current directory)")
                        .value_name("PATH")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .get_matches();

    // Execute the command.
    match args.subcommand().unwrap() {
        ("build", args) => build(args),
        _ => todo!(),
    }
}

fn build(args: &ArgMatches) -> ExitCode {
    // Initialize LLVM.
    unsafe { llvm_init() };

    // Get path to the project.
    let path = match args.get_one::<PathBuf>("project") {
        Some(v) => Cow::Borrowed(v),
        None => Cow::Owned(std::env::current_dir().unwrap()),
    };

    // Setup target resolver.
    let targets = TargetResolver::new();

    // Setup dependency resolver.
    let deps = DependencyResolver::new();

    // Open the project.
    let mut project = match Project::open(path.as_ref(), &targets, &deps) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot open {}: {}.", path.display(), join_nested(&e));
            return ExitCode::FAILURE;
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

        return ExitCode::FAILURE;
    }

    // Build the project.
    let pkg = match project.build() {
        Ok(v) => v,
        Err(ProjectBuildError::InvalidSyntax(p, e)) => {
            eprintln!("{}: {}", p.display(), e);
            return ExitCode::FAILURE;
        }
        Err(ProjectBuildError::BuildFailed(p, e)) => {
            eprintln!("Cannot build {}: {}", p.display(), e);
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("{}: {}", project.path().display(), join_nested(&e));
            return ExitCode::FAILURE;
        }
    };

    // Export the binaries.
    if let Some(path) = args.get_one::<PathBuf>("export") {
        if let Err(e) = pkg.export(&path, &targets, &deps) {
            eprintln!(
                "Cannot export the binaries to {}: {}.",
                path.display(),
                join_nested(&e)
            );
            return ExitCode::FAILURE;
        }
    }

    // Export the package.
    if let Some(path) = args.get_one::<PathBuf>("package") {
        if let Err(e) = pkg.pack(&path) {
            eprintln!("Cannot pack {}: {}.", path.display(), join_nested(&e));
            return ExitCode::FAILURE;
        }
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
