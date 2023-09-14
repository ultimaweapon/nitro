use crate::ast::ParseError;
use crate::project::Project;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::ExitCode;

mod ast;
mod lexer;
mod project;

fn main() -> ExitCode {
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

    // Get path to the project to compile.
    let project = match args.next() {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("Usage: {app} PATH");
            return ExitCode::FAILURE;
        }
    };

    // Enumerate all project files.
    let mut project = Project::new(project);
    let mut jobs: VecDeque<PathBuf> = VecDeque::from([project.path().to_owned()]);

    while let Some(path) = jobs.pop_front() {
        // Enumerate files.
        let items = match std::fs::read_dir(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Cannot enumerate files in {}: {}", path.display(), e);
                return ExitCode::FAILURE;
            }
        };

        for item in items {
            let item = match item {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Cannot read the file in {}: {}", path.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            // Get metadata.
            let path = item.path();
            let meta = match std::fs::metadata(&path) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Cannot get metadata of {}: {}", path.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            // Check if directory.
            if meta.is_dir() {
                jobs.push_back(path);
                continue;
            }

            // Skip if not a Nitro source.
            match path.extension() {
                Some(v) => {
                    if v != "nt" {
                        continue;
                    }
                }
                None => continue,
            }

            // Parse the source.
            if let Err(e) = project.parse_source(&path) {
                match e {
                    ParseError::ReadFailed(e) => eprintln!("{}: {}", path.display(), e),
                    ParseError::ParseFailed(e) => eprintln!("{}: {}", path.display(), e),
                }

                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
