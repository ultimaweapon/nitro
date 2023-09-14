pub use self::lib::*;
pub use self::meta::*;

use std::path::{Path, PathBuf};
use thiserror::Error;

mod lib;
mod meta;

/// A unpacked Nitro package.
///
/// One package can contains only a single executable and a single library, per architecture.
pub struct Package {
    meta: PackageMeta,
    exe: Arch<PathBuf>,
    lib: Arch<(PathBuf, Library)>,
}

impl Package {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PackageOpenError> {
        todo!()
    }

    pub fn new(meta: PackageMeta, exe: Arch<PathBuf>, lib: Arch<(PathBuf, Library)>) -> Self {
        assert!(exe.is_some() || lib.is_some());

        Self { meta, exe, lib }
    }

    pub fn pack<F: AsRef<Path>>(&self, file: F) -> Result<(), PackagePackError> {
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

/// Encapsuate an object per architecture.
pub struct Arch<T> {
    aarch64_apple_darwin: Option<T>,
    x86_64_apple_darwin: Option<T>,
    x86_64_pc_win32_msvc: Option<T>,
    x86_64_unknown_linux_gnu: Option<T>,
}

impl<T> Arch<T> {
    pub fn new() -> Self {
        Self {
            aarch64_apple_darwin: None,
            x86_64_apple_darwin: None,
            x86_64_pc_win32_msvc: None,
            x86_64_unknown_linux_gnu: None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.aarch64_apple_darwin.is_some()
            || self.x86_64_apple_darwin.is_some()
            || self.x86_64_pc_win32_msvc.is_some()
            || self.x86_64_unknown_linux_gnu.is_some()
    }
}

/// Represents an error when a package is failed to open.
#[derive(Debug, Error)]
pub enum PackageOpenError {}

/// Represents an error when a package is failed to pack.
#[derive(Debug, Error)]
pub enum PackagePackError {}

/// Represents an error when a package is failed to unpack.
#[derive(Debug, Error)]
pub enum PackageUnpackError {}
