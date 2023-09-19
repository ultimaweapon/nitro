use crate::ast::ParseError;
use crate::project::{Project, ProjectBuildError, ProjectLoadError};
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
    // Initialize LLVM.
    unsafe {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmPrinters();
    }

    // Get binary name.
    let mut args = std::env::args_os();
    let app = match args.next() {
        Some(v) => match v.into_string() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Binary name in the command line is not UTF-8.");
                return ExitCode::FAILURE;
            }
        },
        None => {
            eprintln!("No binary name in the command line.");
            return ExitCode::FAILURE;
        }
    };

    // Get path to the target project.
    let project = match args.next() {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("Usage: {app} PATH");
            return ExitCode::FAILURE;
        }
    };

    // Open the project.
    let mut project = match Project::open(&project) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot open {}: {}.", project.display(), join_nested(&e));
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
