use crate::pkg::{PackageName, PackageVersion};
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
    name: PackageName,
    #[serde(rename = "type")]
    ty: ProjectType,
    version: PackageVersion,
}

impl ProjectPackage {
    pub fn name(&self) -> &PackageName {
        &self.name
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
