pub use self::dep::*;
pub use self::lib::*;
pub use self::meta::*;
pub use self::target::*;
pub use self::ty::*;

use crate::zstd::{ZstdReader, ZstdWriter};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

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

    pub fn new(
        meta: PackageMeta,
        exes: HashMap<Target, Binary<PathBuf>>,
        libs: HashMap<Target, Binary<Library>>,
    ) -> Self {
        assert!(!exes.is_empty() || !libs.is_empty());

        Self { meta, exes, libs }
    }

    pub fn meta(&self) -> &PackageMeta {
        &self.meta
    }

    pub fn libs(&self) -> &HashMap<Target, Binary<Library>> {
        &self.libs
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
        target: &Target,
        targets: &TargetResolver,
        deps: &DependencyResolver,
    ) -> Result<(), PackageExportError>
    where
        T: AsRef<Path>,
    {
        // Create destination directory.
        let to = to.as_ref();

        if let Err(e) = std::fs::create_dir_all(to) {
            return Err(PackageExportError::CreateDirectoryFailed(to.to_owned(), e));
        }

        // Resolve primitive target.
        let pt = match targets.primitive(target) {
            Ok(v) => v,
            Err(e) => return Err(PackageExportError::ResolvePrimitiveTargetFailed(e)),
        };

        // If there is an executable, export it otherwise export a library instead.
        let base = self.meta.name();
        let (from, to) = if self.exes.is_empty() {
            let lib = self
                .libs
                .get(target)
                .ok_or(PackageExportError::TargetNotFound)?;

            // Get binary path.
            let from = match lib.bin.bin() {
                LibraryBinary::Bundle(v) => v,
                LibraryBinary::System(_) => return Err(PackageExportError::SystemLibrary),
            };

            // Get destination path.
            let ver = self.meta.version().major();
            let to = to.join(match pt.os() {
                TargetOs::Darwin => {
                    if ver == 0 {
                        format!("lib{base}.dylib")
                    } else {
                        format!("lib{base}-v{ver}.dylib")
                    }
                }
                TargetOs::Linux => {
                    if ver == 0 {
                        format!("lib{base}.so")
                    } else {
                        format!("lib{base}-v{ver}.so")
                    }
                }
                TargetOs::Win32 => {
                    if ver == 0 {
                        format!("{base}.dll")
                    } else {
                        format!("{base}-v{ver}.dll")
                    }
                }
            });

            (from, to)
        } else {
            let exe = self
                .exes
                .get(target)
                .ok_or(PackageExportError::TargetNotFound)?;

            // Get destination path.
            let to = to.join(match pt.os() {
                TargetOs::Darwin | TargetOs::Linux => base.as_str().to_owned(),
                TargetOs::Win32 => format!("{base}.exe"),
            });

            (&exe.bin, to)
        };

        // Export.
        if let Err(e) = std::fs::copy(from, &to) {
            return Err(PackageExportError::CopyFailed(from.clone(), to, e));
        }

        Ok(())
    }

    pub fn unpack<P, T>(mut pkg: P, to: T) -> Result<(), PackageUnpackError>
    where
        P: Read,
        T: AsRef<Path>,
    {
        // Check magic.
        let mut magic = [0u8; 4];

        pkg.read_exact(&mut magic)?;

        if magic.ne(b"\x7FNPK") {
            return Err(PackageUnpackError::NotNitroPackage);
        }

        // Create destination directory.
        let to = to.as_ref();

        if let Err(e) = std::fs::create_dir_all(to) {
            return Err(PackageUnpackError::CreateDirectoryFailed(to.to_owned(), e));
        }

        // Create a directory for libraries.
        let libs = to.join("libs");

        if let Err(e) = std::fs::create_dir(&libs) {
            return Err(PackageUnpackError::CreateDirectoryFailed(libs, e));
        }

        // Iterate over the entries.
        let mut name = None;
        let mut version = None;
        let mut nlib = 0;

        loop {
            // Read entry type.
            let mut ty = 0;

            pkg.read_exact(std::slice::from_mut(&mut ty))?;

            // Process the entry.
            match ty {
                Self::ENTRY_END => break,
                Self::ENTRY_NAME => {
                    let mut data = [0u8; 32];
                    pkg.read_exact(&mut data)?;
                    name = Some(
                        PackageName::from_bin(&data)
                            .map_err(|e| PackageUnpackError::InvalidNameEntry(e))?,
                    );
                }
                Self::ENTRY_VERSION => {
                    let mut data = [0u8; 8];
                    pkg.read_exact(&mut data)?;
                    version = Some(PackageVersion::from_bin(u64::from_be_bytes(data)));
                }
                Self::ENTRY_DATE => {
                    let mut data = [0u8; 8];
                    pkg.read_exact(&mut data)?;
                }
                Self::ENTRY_LIB => {
                    // Read target.
                    let mut data = [0u8; 16];
                    pkg.read_exact(&mut data)?;

                    // Create a directory to unpack the library.
                    let target = Uuid::from_bytes(data);
                    let dir = libs.join(target.to_string());

                    if let Err(e) = std::fs::create_dir(&dir) {
                        return Err(PackageUnpackError::CreateDirectoryFailed(dir, e));
                    }

                    // Read dependency count.
                    let mut data = [0u8; 2];
                    pkg.read_exact(&mut data)?;
                    let ndep: usize = u16::from_be_bytes(data).into();

                    // Read dependencies.
                    let mut deps = Vec::with_capacity(ndep);

                    for i in 0..ndep {
                        match Dependency::deserialize(&mut pkg) {
                            Ok(v) => deps.push(v),
                            Err(e) => {
                                return Err(PackageUnpackError::InvalidLibraryDependency(
                                    nlib, i, e,
                                ));
                            }
                        };
                    }

                    // Read binary length.
                    let mut data = [0; 4];
                    pkg.read_exact(&mut data)?;
                    let len: u64 = u32::from_be_bytes(data).into();

                    // Read the binary.
                    let reader = ZstdReader::new(pkg.by_ref().take(len));

                    if let Err(e) = Library::unpack(reader, dir.join("bin"), dir.join("types")) {
                        return Err(PackageUnpackError::UnpackLibraryFailed(dir, e));
                    }

                    // Write dependencies.
                    let path = dir.join("deps.yml");
                    let file = match File::create(&path) {
                        Ok(v) => v,
                        Err(e) => return Err(PackageUnpackError::WriteFileFailed(path, e)),
                    };

                    serde_yaml::to_writer(file, &deps).unwrap();

                    nlib += 1;
                }
                v => return Err(PackageUnpackError::UnknownEntry(v)),
            }
        }

        // Write metadata.
        let name = name.ok_or(PackageUnpackError::NoNameEntry)?;
        let version = version.ok_or(PackageUnpackError::NoVersionEntry)?;
        let meta = PackageMeta::new(name, version);
        let path = to.join("meta.yml");
        let file = match File::create(&path) {
            Ok(v) => v,
            Err(e) => return Err(PackageUnpackError::WriteFileFailed(path, e)),
        };

        serde_yaml::to_writer(file, &meta).unwrap();

        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PackageOpenError> {
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

    pub fn bin(&self) -> &T {
        &self.bin
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
pub enum PackageExportError {
    #[error("cannot create {0}")]
    CreateDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot resolve primitive target")]
    ResolvePrimitiveTargetFailed(#[source] TargetResolveError),

    #[error("no binary for the specified target")]
    TargetNotFound,

    #[error("a system library cannot be exported")]
    SystemLibrary,

    #[error("cannot copy {0} to {1}")]
    CopyFailed(PathBuf, PathBuf, #[source] std::io::Error),
}

/// Represents an error when a package is failed to unpack.
#[derive(Debug, Error)]
pub enum PackageUnpackError {
    #[error("cannot read the package")]
    ReadPackageFailed(#[source] std::io::Error),

    #[error("the specified file is not a Nitro package")]
    NotNitroPackage,

    #[error("cannot create {0}")]
    CreateDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("name entry in the package is not valid")]
    InvalidNameEntry(#[source] PackageNameError),

    #[error("unknown entry {0} in the package")]
    UnknownEntry(u8),

    #[error("no name entry in the package")]
    NoNameEntry,

    #[error("no version entry in the package")]
    NoVersionEntry,

    #[error("dependency #{1} for library entry #{0} is not valid")]
    InvalidLibraryDependency(usize, usize, #[source] DependencyError),

    #[error("cannot unpack the library to {0}")]
    UnpackLibraryFailed(PathBuf, #[source] LibraryUnpackError),

    #[error("cannot write {0}")]
    WriteFileFailed(PathBuf, #[source] std::io::Error),
}

impl From<std::io::Error> for PackageUnpackError {
    fn from(value: std::io::Error) -> Self {
        Self::ReadPackageFailed(value)
    }
}
