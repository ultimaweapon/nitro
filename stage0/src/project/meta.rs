use crate::pkg::PackageMeta;
use serde::Deserialize;
use std::path::PathBuf;

/// Contains information that was loaded from `Nitro.yml` file.
#[derive(Deserialize)]
pub struct ProjectMeta {
    package: PackageMeta,
    executable: Option<ProjectBinary>,
    library: Option<ProjectBinary>,
}

impl ProjectMeta {
    pub fn package(&self) -> &PackageMeta {
        &self.package
    }

    pub fn executable(&self) -> Option<&ProjectBinary> {
        self.executable.as_ref()
    }

    pub fn library(&self) -> Option<&ProjectBinary> {
        self.library.as_ref()
    }
}

/// Contains information of the binary that the project will output.
#[derive(Deserialize)]
pub struct ProjectBinary {
    sources: PathBuf,
}

impl ProjectBinary {
    pub fn sources(&self) -> &PathBuf {
        &self.sources
    }
}
