use crate::ast::ParseError;
use crate::dep::DepResolver;
use crate::ffi::llvm_init;
use crate::project::{Project, ProjectBuildError, ProjectLoadError, ProjectType};
use clap::{command, value_parser, Arg, ArgMatches, Command};
use std::error::Error;
use std::fmt::Write;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod codegen;
mod dep;
mod ffi;
mod lexer;
mod pkg;
mod project;

fn main() -> ExitCode {
    // Parse arguments.
    let args = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("build")
                .about("Build a Nitro project")
                .arg(
                    Arg::new("export")
                        .help("Export executables or a package to the specified directory")
                        .long("export")
                        .value_name("PATH")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("project")
                        .help("Path to the project")
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

    // Setup dependency resolver.
    let mut resolver = DepResolver::new();

    // Open the project.
    let path = args.get_one::<PathBuf>("project").unwrap();
    let mut project = match Project::open(&path, &mut resolver) {
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

    // Export the project.
    if let Some(path) = args.get_one::<PathBuf>("export") {
        match project.meta().package().ty() {
            ProjectType::Executable => {
                // Export the project.
                if let Err(e) = pkg.export(&path, &mut resolver) {
                    eprintln!(
                        "Cannot export the project to {}: {}.",
                        path.display(),
                        join_nested(&e)
                    );
                    return ExitCode::FAILURE;
                }
            }
            ProjectType::Library => {
                // Create a directory to pack the package.
                if let Err(e) = create_dir_all(&path) {
                    eprintln!("Cannot create {}: {}.", path.display(), e);
                    return ExitCode::FAILURE;
                }

                // Pack the package.
                let path = path.join(format!("{}.npk", project.meta().package().name()));

                if let Err(e) = pkg.pack(&path) {
                    eprintln!("Cannot pack {}: {}.", path.display(), join_nested(&e));
                    return ExitCode::FAILURE;
                }
            }
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
