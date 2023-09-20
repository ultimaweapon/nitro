use crate::ast::ParseError;
use crate::project::{Project, ProjectBuildError, ProjectLoadError};
use clap::{command, value_parser, Arg, ArgMatches, Command};
use llvm_sys::target::{
    LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs,
    LLVM_InitializeAllTargets,
};
use std::error::Error;
use std::fmt::Write;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod codegen;
mod lexer;
mod pkg;
mod project;

fn main() -> ExitCode {
    // Parse arguments.
    let args = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("build").about("Build a Nitro project").arg(
                Arg::new("path")
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
    unsafe {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmPrinters();
    }

    // Open the project.
    let path = args.get_one::<PathBuf>("path").unwrap();
    let mut project = match Project::open(&path) {
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
        Err(ProjectBuildError::CreateDirectoryFailed(p, e)) => {
            eprintln!("Cannot create {}: {}.", p.display(), e);
            return ExitCode::FAILURE;
        }
        Err(ProjectBuildError::BuildFailed(p, e)) => {
            eprintln!("Cannot build {}: {}", p.display(), e);
            return ExitCode::FAILURE;
        }
        Err(ProjectBuildError::LinkFailed(p, e)) => {
            eprintln!("Cannot link {}: {}.", p.display(), e);
            return ExitCode::FAILURE;
        }
    };

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
