use crate::ast::{ParseError, SourceFile};
use std::path::{Path, PathBuf};

/// A Pluto project.
pub struct Project {
    path: PathBuf,
    sources: Vec<SourceFile>,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            sources: Vec::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn parse_source<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ParseError> {
        self.sources.push(SourceFile::parse(path.as_ref())?);
        Ok(())
    }
}
