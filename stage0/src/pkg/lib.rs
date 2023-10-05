use super::ExportedType;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

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
    types: HashSet<ExportedType>,
}

impl Library {
    const ENTRY_END: u8 = 0;
    const ENTRY_TYPES: u8 = 1;

    pub fn new(bin: LibraryBinary, types: HashSet<ExportedType>) -> Self {
        Self { bin, types }
    }

    pub fn serialize<W: Write>(&self, mut w: W) -> Result<(), std::io::Error> {
        // Write magic.
        w.write_all(b"\x7FNLM")?;

        // Write types.
        let types: u32 = self.types.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_TYPES])?;
        w.write_all(&types.to_be_bytes())?;

        for ty in &self.types {
            ty.serialize(&mut w)?;
        }

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

/// A library's binary.
pub enum LibraryBinary {
    Bundle(PathBuf),
    System(String),
}
