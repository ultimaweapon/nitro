use super::{
    Package, PackageName, PackageNameError, PackageOpenError, PackageUnpackError, PackageVersion,
    TargetResolver,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use thiserror::Error;

/// An object for resolving package dependencies.
pub struct DependencyResolver {
    cache: PathBuf,
    loaded: RefCell<BTreeMap<Dependency, Rc<Package>>>,
    std: PathBuf,
}

impl DependencyResolver {
    pub fn new<C, S>(cache: C, std: S) -> Self
    where
        C: Into<PathBuf>,
        S: Into<PathBuf>,
    {
        Self {
            cache: cache.into(),
            loaded: RefCell::default(),
            std: std.into(),
        }
    }

    pub fn resolve(
        &self,
        id: &Dependency,
        targets: &TargetResolver,
    ) -> Result<Rc<Package>, DependencyResolveError> {
        // Check if already loaded.
        let mut loaded = self.loaded.borrow_mut();

        if let Some((_, loaded)) = loaded.range(id..).next() {
            if loaded.meta.version().major() == id.version.major() {
                return Ok(loaded.clone());
            }
        }

        // Check for cache.
        let cache = self.cache.join(format!("{}-{}", id.name, id.version));

        match cache.symlink_metadata() {
            Ok(_) => match Package::open(&cache, targets) {
                Ok(v) => {
                    let pkg = Rc::new(v);
                    assert!(loaded.insert(id.clone(), pkg.clone()).is_none());
                    return Ok(pkg);
                }
                Err(e) => return Err(DependencyResolveError::OpenPackageFailed(cache, e)),
            },
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(DependencyResolveError::CheckCacheFailed(cache, e));
                }
            }
        }

        // Get package file.
        let pkg: Box<dyn Read> = if id.name.eq("nitro") {
            match File::open(&self.std) {
                Ok(v) => Box::new(v),
                Err(e) => return Err(DependencyResolveError::OpenStdFailed(self.std.clone(), e)),
            }
        } else {
            todo!()
        };

        // Unpack the package.
        Package::unpack(pkg, &cache).map_err(|e| DependencyResolveError::UnpackPackageFailed(e))?;

        // Open the package.
        match Package::open(&cache, targets) {
            Ok(v) => {
                let pkg = Rc::new(v);
                assert!(loaded.insert(id.clone(), pkg.clone()).is_none());
                return Ok(pkg);
            }
            Err(e) => return Err(DependencyResolveError::OpenPackageFailed(cache, e)),
        }
    }
}

/// A package dependency.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Dependency {
    name: PackageName,
    version: PackageVersion,
}

impl Dependency {
    pub fn new(name: PackageName, version: PackageVersion) -> Self {
        Self { name, version }
    }

    pub fn deserialize<R: Read>(mut r: R) -> Result<Self, DependencyError> {
        // Read name.
        let mut data = [0; 32];
        r.read_exact(&mut data)?;
        let name = PackageName::from_bin(&data).map_err(|e| DependencyError::InvalidName(e))?;

        // Read version.
        let mut data = [0; 8];
        r.read_exact(&mut data)?;
        let version = PackageVersion::from_bin(u64::from_be_bytes(data));

        Ok(Self { name, version })
    }

    pub fn serialize<W: Write>(&self, mut w: W) -> Result<(), std::io::Error> {
        w.write_all(&self.name.to_bin())?;
        w.write_all(&self.version.to_bin().to_be_bytes())
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}

/// Represents an error when [`DependencyResolver::resolve()`] is failed.
#[derive(Debug, Error)]
pub enum DependencyResolveError {
    #[error("cannot open a package from {0}")]
    OpenPackageFailed(PathBuf, #[source] PackageOpenError),

    #[error("cannot check {0}")]
    CheckCacheFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot open {0}")]
    OpenStdFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot unpack the package")]
    UnpackPackageFailed(#[source] PackageUnpackError),
}

/// Represents an error when [`Dependency`] is failed to construct.
#[derive(Debug, Error)]
pub enum DependencyError {
    #[error("cannot read data")]
    ReadDataFailed(#[source] std::io::Error),

    #[error("invalid package name")]
    InvalidName(#[source] PackageNameError),
}

impl From<std::io::Error> for DependencyError {
    fn from(value: std::io::Error) -> Self {
        Self::ReadDataFailed(value)
    }
}
