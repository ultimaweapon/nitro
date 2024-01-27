use super::{TypeDeclaration, TypeDeserializeError};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A Nitro library.
///
/// A Nitro library is always a shared library. Nitro can consume a static library but cannot
/// produce it. The reason is because it will cause a surprising behavior to the user in the
/// following scenario:
///
/// - Alice publish a static library named `foo`.
/// - Bob publish a shared library named `bar` that link to `foo`.
/// - Carol publish a shared library named `baz` that also link to `foo`.
/// - Carlos build a binary that link to both `bar` and `baz`.
///
/// There will be two states of `foo` here, which likely to cause a headache to Alice to figure out
/// what wrong with `foo` when Carlos report something is not working.
pub struct Library {
    bin: LibraryBinary,
    types: HashSet<TypeDeclaration>,
}

impl Library {
    const ENTRY_END: u8 = 0;
    const ENTRY_TYPES: u8 = 1;
    const ENTRY_SYSTEM: u8 = 2;

    pub fn new(bin: LibraryBinary, types: HashSet<TypeDeclaration>) -> Self {
        Self { bin, types }
    }

    pub fn open<B, T>(bin: B, types: T) -> Result<Self, LibraryError>
    where
        B: AsRef<Path>,
        T: AsRef<Path>,
    {
        // Read binary magic.
        let bin = bin.as_ref();
        let mut file =
            File::open(bin).map_err(|e| LibraryError::OpenFileFailed(bin.to_owned(), e))?;
        let mut magic = [0u8; 4];

        file.read_exact(&mut magic)
            .map_err(|e| LibraryError::ReadFileFailed(bin.to_owned(), e))?;

        // Check binary type.
        let bin = if &magic == b"\x7FNLS" {
            let mut name = String::new();
            file.read_to_string(&mut name)
                .map_err(|e| LibraryError::ReadFileFailed(bin.to_owned(), e))?;
            LibraryBinary::System(name)
        } else {
            LibraryBinary::Bundle(bin.to_owned())
        };

        // Load types.
        let path = types.as_ref();
        let mut file =
            File::open(path).map_err(|e| LibraryError::OpenFileFailed(path.to_owned(), e))?;
        let mut types = HashSet::new();

        loop {
            let ty = match TypeDeclaration::deserialize(&mut file) {
                Ok(v) => v,
                Err(TypeDeserializeError::EmptyData) => break,
                Err(e) => return Err(LibraryError::ReadTypeFailed(path.to_owned(), e)),
            };

            if !types.insert(ty) {
                return Err(LibraryError::DuplicatedType(path.to_owned()));
            }
        }

        Ok(Self { bin, types })
    }

    pub fn bin(&self) -> &LibraryBinary {
        &self.bin
    }

    pub fn types(&self) -> &HashSet<TypeDeclaration> {
        &self.types
    }

    pub(super) fn serialize<W: Write>(&self, mut w: W) -> Result<(), std::io::Error> {
        // Write magic.
        w.write_all(b"\x7FNLM")?;

        // Write types.
        let types: u32 = self.types.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_TYPES])?;
        w.write_all(&types.to_be_bytes())?;

        for ty in &self.types {
            ty.serialize(&mut w)?;
        }

        // Write binary.
        match &self.bin {
            LibraryBinary::Bundle(path) => {
                let mut file = File::open(&path)?;

                w.write_all(&[Self::ENTRY_END])?;
                std::io::copy(&mut file, &mut w)?;

                Ok(())
            }
            LibraryBinary::System(name) => {
                let len: u16 = name.len().try_into().unwrap();

                w.write_all(&[Self::ENTRY_SYSTEM])?;
                w.write_all(&len.to_be_bytes())?;
                w.write_all(name.as_bytes())?;
                w.write_all(&[Self::ENTRY_END])
            }
        }
    }

    pub(super) fn unpack<R, B, T>(mut data: R, bin: B, types: T) -> Result<(), LibraryUnpackError>
    where
        R: Read,
        B: AsRef<Path>,
        T: AsRef<Path>,
    {
        // Check magic.
        let mut magic = [0u8; 4];

        data.read_exact(&mut magic)?;

        if magic.ne(b"\x7FNLM") {
            return Err(LibraryUnpackError::NotNitroLibrary);
        }

        // Iterate over the entries.
        let mut bin = File::create(bin).map_err(LibraryUnpackError::WriteBinaryFailed)?;
        let mut types = File::create(types).map_err(LibraryUnpackError::WriteTypeFailed)?;
        let mut sys = None;

        loop {
            // Read entry type.
            let mut ty = 0;

            data.read_exact(std::slice::from_mut(&mut ty))?;

            // Process the entry.
            match ty {
                Self::ENTRY_END => break,
                Self::ENTRY_TYPES => {
                    // Read types count.
                    let mut buf = [0u8; 4];
                    data.read_exact(&mut buf)?;
                    let ntype: usize = u32::from_be_bytes(buf).try_into().unwrap();

                    // Read types.
                    for i in 0..ntype {
                        let ty = TypeDeclaration::deserialize(&mut data)
                            .map_err(|e| LibraryUnpackError::ReadTypeFailed(i, e))?;
                        ty.serialize(&mut types)
                            .map_err(LibraryUnpackError::WriteTypeFailed)?;
                    }
                }
                Self::ENTRY_SYSTEM => {
                    // Read name length.
                    let mut buf = [0u8; 2];
                    data.read_exact(&mut buf)?;
                    let len: usize = u16::from_be_bytes(buf).into();

                    // Read name.
                    let mut buf = vec![0u8; len];
                    data.read_exact(&mut buf)?;

                    match String::from_utf8(buf) {
                        Ok(v) => sys = Some(v),
                        Err(_) => return Err(LibraryUnpackError::InvalidSystemName),
                    }
                }
                v => return Err(LibraryUnpackError::UnknownEntry(v)),
            }
        }

        // Write binary.
        match sys {
            Some(name) => {
                bin.write_all(b"\x7FNLS")
                    .map_err(LibraryUnpackError::WriteBinaryFailed)?;
                bin.write_all(name.as_bytes())
                    .map_err(LibraryUnpackError::WriteBinaryFailed)?;
            }
            None => {
                std::io::copy(&mut data, &mut bin)
                    .map_err(LibraryUnpackError::WriteBinaryFailed)?;
            }
        }

        Ok(())
    }
}

/// A library's binary.
pub enum LibraryBinary {
    Bundle(PathBuf),
    System(String),
}

/// Represents an error when [`Library`] is failed to construct.
#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("cannot open {0}")]
    OpenFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot read {0}")]
    ReadFileFailed(PathBuf, #[source] std::io::Error),

    #[error("cannot read type declaration from {0}")]
    ReadTypeFailed(PathBuf, #[source] TypeDeserializeError),

    #[error("duplicated type declaration in {0}")]
    DuplicatedType(PathBuf),
}

/// Represents an error when [`Library`] is failed to unpack from a serialized data.
#[derive(Debug, Error)]
pub enum LibraryUnpackError {
    #[error("cannot read data")]
    ReadDataFailed(#[source] std::io::Error),

    #[error("the data is not a Nitro library")]
    NotNitroLibrary,

    #[error("cannot write binary")]
    WriteBinaryFailed(#[source] std::io::Error),

    #[error("cannot write type")]
    WriteTypeFailed(#[source] std::io::Error),

    #[error("cannot read type #{0}")]
    ReadTypeFailed(usize, #[source] TypeDeserializeError),

    #[error("invalid name for system library")]
    InvalidSystemName,

    #[error("unknown entry {0}")]
    UnknownEntry(u8),
}

impl From<std::io::Error> for LibraryUnpackError {
    fn from(value: std::io::Error) -> Self {
        Self::ReadDataFailed(value)
    }
}
