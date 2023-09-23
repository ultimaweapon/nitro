use crate::pkg::PackageVersion;
use serde::Deserialize;

/// Contains information that was loaded from `.nitro` file.
#[derive(Deserialize)]
pub struct ProjectMeta {
    package: ProjectPackage,
}

impl ProjectMeta {
    pub fn package(&self) -> &ProjectPackage {
        &self.package
    }
}

/// A package table of `.nitro` file.
#[derive(Deserialize)]
pub struct ProjectPackage {
    name: String, // TODO: Only allow identifier.
    #[serde(rename = "type")]
    ty: ProjectType,
    version: PackageVersion,
}

impl ProjectPackage {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn ty(&self) -> ProjectType {
        self.ty
    }

    pub fn version(&self) -> &PackageVersion {
        &self.version
    }
}

/// Type of the project.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    #[serde(rename = "exe")]
    Executable,

    #[serde(rename = "lib")]
    Library,
}
