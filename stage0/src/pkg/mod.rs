pub use self::dep::*;
pub use self::lib::*;
pub use self::meta::*;
pub use self::target::*;
pub use self::ty::*;

use crate::zstd::ZstdWriter;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

mod dep;
mod lib;
mod meta;
mod target;
mod ty;

/// An unpacked Nitro package.
///
/// One package can contains only a single executable and a single library per target.
pub struct Package {
    meta: PackageMeta,
    exes: HashMap<Target, Binary<PathBuf>>,
    libs: HashMap<Target, Binary<Library>>,
}

impl Package {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_VERSION: u8 = 2;
    const ENTRY_DATE: u8 = 3;
    const ENTRY_EXE: u8 = 4;
    const ENTRY_LIB: u8 = 5;

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PackageOpenError> {
        todo!()
    }

    pub fn new(
        meta: PackageMeta,
        exes: HashMap<Target, Binary<PathBuf>>,
        libs: HashMap<Target, Binary<Library>>,
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
        file.write_all(b"\x7FNPK")?;

        // Write package name.
        let meta = &self.meta;

        file.write_all(&[Self::ENTRY_NAME])?;
        file.write_all(&meta.name().to_bin())?;

        // Write package version.
        file.write_all(&[Self::ENTRY_VERSION])?;
        file.write_all(&meta.version().to_bin().to_be_bytes())?;

        // Write created date.
        let date = SystemTime::now();

        file.write_all(&[Self::ENTRY_DATE])?;
        file.write_all(
            &date
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_be_bytes(),
        )?;

        // Write libraries
        for (target, lib) in &self.libs {
            // Write the target.
            file.write_all(&[Self::ENTRY_LIB])?;
            file.write_all(target.id().as_bytes())?;

            // Write dependencies.
            let count = TryInto::<u16>::try_into(lib.deps.len())
                .unwrap()
                .to_be_bytes();

            file.write_all(&count)?;

            for dep in &lib.deps {
                dep.serialize(&mut file)?;
            }

            // Create a placeholder for binary length.
            let lenoff = file.stream_position().unwrap();

            file.write_all(&[0; 4])?;

            // Write the library.
            let mut writer = ZstdWriter::new(&mut file);

            lib.bin.serialize(&mut writer)?;
            writer.flush()?;

            drop(writer);

            // Write library length.
            let cur = file.stream_position().unwrap();
            let len: u32 = (cur - lenoff - 4).try_into().unwrap();

            file.seek(SeekFrom::Start(lenoff)).unwrap();
            file.write_all(&len.to_be_bytes())?;
            file.seek(SeekFrom::Start(cur)).unwrap();
        }

        // End of entries.
        file.write_all(&[Self::ENTRY_END])?;

        Ok(())
    }

    pub fn export<T>(
        &self,
        to: T,
        targets: &TargetResolver,
        deps: &DependencyResolver,
    ) -> Result<(), PackageExportError>
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

/// A compiled binary file.
pub struct Binary<T> {
    bin: T,
    deps: HashSet<Dependency>,
}

impl<T> Binary<T> {
    pub fn new(bin: T, deps: HashSet<Dependency>) -> Self {
        Self { bin, deps }
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

impl From<std::io::Error> for PackagePackError {
    fn from(value: std::io::Error) -> Self {
        Self::WriteFailed(value)
    }
}

/// Represents an error when a package is failed to export.
#[derive(Debug, Error)]
pub enum PackageExportError {}

/// Represents an error when a package is failed to unpack.
#[derive(Debug, Error)]
pub enum PackageUnpackError {}
