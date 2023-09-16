use crate::pkg::PackageVersion;
use serde::Deserialize;

/// Contains information that was loaded from `.nitro` file.
#[derive(Deserialize)]
pub struct ProjectMeta {
    pub package: ProjectPackage,
}

/// A package table of `.nitro` file.
#[derive(Deserialize)]
pub struct ProjectPackage {
    pub name: String, // TODO: Only allow a-z, 0-9 and . alphabets.
    #[serde(rename = "type")]
    pub ty: ProjectType,
    pub version: PackageVersion,
}

/// Type of the project.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    #[serde(rename = "exe")]
    Executable,

    #[serde(rename = "lib")]
    Library,
}
