use crate::pkg::{PackageName, PackageVersion};
use serde::Deserialize;
use std::path::PathBuf;

/// Contains information that was loaded from `Nitro.yml` file.
#[derive(Deserialize)]
pub struct ProjectMeta {
    package: ProjectPackage,
    executable: Option<ProjectBinary>,
    library: Option<ProjectBinary>,
}

impl ProjectMeta {
    pub fn package(&self) -> &ProjectPackage {
        &self.package
    }

    pub fn executable(&self) -> Option<&ProjectBinary> {
        self.executable.as_ref()
    }

    pub fn library(&self) -> Option<&ProjectBinary> {
        self.library.as_ref()
    }
}

/// Contains information of a package that the project will output.
#[derive(Deserialize)]
pub struct ProjectPackage {
    name: PackageName,
    version: PackageVersion,
}

impl ProjectPackage {
    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn version(&self) -> &PackageVersion {
        &self.version
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
