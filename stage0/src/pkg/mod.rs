pub use self::lib::*;
pub use self::meta::*;
pub use self::target::*;
pub use self::ty::*;

use crate::dep::DepResolver;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

mod lib;
mod meta;
mod target;
mod ty;

/// A unpacked Nitro package.
///
/// One package can contains only a single executable and a single library, per architecture.
pub struct Package {
    meta: PackageMeta,
    exes: HashMap<Target, PathBuf>,
    libs: HashMap<Target, (PathBuf, Library)>,
}

impl Package {
    const META_END: u8 = 0;
    const META_NAME: u8 = 1;
    const META_VERSION: u8 = 2;
    const META_DATE: u8 = 3;

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PackageOpenError> {
        todo!()
    }

    pub fn new(
        meta: PackageMeta,
        exes: HashMap<Target, PathBuf>,
        libs: HashMap<Target, (PathBuf, Library)>,
    ) -> Self {
        assert!(!exes.is_empty() || !libs.is_empty());

        Self { meta, exes, libs }
    }

    pub fn pack<F: AsRef<Path>>(&self, file: F) -> Result<(), PackagePackError> {
        // Create a package file.
        let path = file.as_ref();
        let mut file = match File::create(path) {
            Ok(v) => v,
            Err(e) => return Err(PackagePackError::CreateFileFailed(e)),
        };

        // Write file magic.
        file.write_all(b"\x7FNPK")
            .map_err(|e| PackagePackError::WriteFailed(e))?;

        // Write package name.
        let meta = &self.meta;

        file.write_all(&[Self::META_NAME]).unwrap();
        file.write_all(&meta.name().to_bin()).unwrap();

        // Write package version.
        file.write_all(&[Self::META_VERSION]).unwrap();
        file.write_all(&meta.version().to_bin().to_be_bytes())
            .unwrap();

        // Write created date.
        let date = SystemTime::now();

        file.write_all(&[Self::META_DATE]).unwrap();
        file.write_all(
            &date
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_be_bytes(),
        )
        .unwrap();

        // End of meta data.
        file.write_all(&[Self::META_END]).unwrap();

        Ok(())
    }

    pub fn export<T>(&self, to: T, resolver: &mut DepResolver) -> Result<(), PackageExportError>
    where
        T: AsRef<Path>,
    {
        todo!()
    }

    pub fn unpack<F, T>(file: F, to: T) -> Result<(), PackageUnpackError>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
    {
        todo!()
    }
}

/// Represents an error when a package is failed to open.
#[derive(Debug, Error)]
pub enum PackageOpenError {}

/// Represents an error when a package is failed to pack.
#[derive(Debug, Error)]
pub enum PackagePackError {
    #[error("cannot create the specified file")]
    CreateFileFailed(#[source] std::io::Error),

    #[error("cannot write the specified file")]
    WriteFailed(#[source] std::io::Error),
}

/// Represents an error when a package is failed to export.
#[derive(Debug, Error)]
pub enum PackageExportError {}

/// Represents an error when a package is failed to unpack.
#[derive(Debug, Error)]
pub enum PackageUnpackError {}
