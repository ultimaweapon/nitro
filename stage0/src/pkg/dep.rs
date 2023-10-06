use super::{PackageName, PackageVersion};
use std::hash::{Hash, Hasher};
use std::io::Write;

/// An object for resolving package dependencies.
pub struct DependencyResolver {}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {}
    }
}

/// A package dependency.
pub struct Dependency {
    name: PackageName,
    version: PackageVersion,
}

impl Dependency {
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
