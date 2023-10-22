use super::{Package, PackageName, PackageVersion};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use thiserror::Error;

/// An object for resolving package dependencies.
pub struct DependencyResolver {}

impl DependencyResolver {
    pub fn new<S: Into<PathBuf>>(std: S) -> Self {
        Self {}
    }

    pub fn resolve(&self, id: &Dependency) -> Result<Rc<Package>, DependencyResolveError> {
        todo!()
    }
}

/// A package dependency.
#[derive(Debug)]
pub struct Dependency {
    name: PackageName,
    version: PackageVersion,
}

impl Dependency {
    pub fn new(name: PackageName, version: PackageVersion) -> Self {
        Self { name, version }
    }

    pub fn serialize<W: Write>(&self, mut w: W) -> Result<(), std::io::Error> {
        w.write_all(&self.name.to_bin())?;
        w.write_all(&self.version.to_bin().to_be_bytes())
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version.major() == other.version.major()
    }
}

impl Eq for Dependency {}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.major().hash(state);
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}

/// Represents an error when [`DependencyResolver::resolve()`] is failed.
#[derive(Debug, Error)]
pub enum DependencyResolveError {}
