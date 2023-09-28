use super::{ExportedType, PackageName, PackageVersion};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Contains information about a Nitro library.
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
    types: HashSet<ExportedType>,
}

impl Library {
    const ENTRY_END: u8 = 0;
    const ENTRY_TYPES: u8 = 1;

    pub fn new() -> Self {
        Self {
            types: HashSet::new(),
        }
    }

    pub fn add_type(&mut self, ty: ExportedType) {
        assert!(self.types.insert(ty));
    }

    pub fn write_module_definition<F>(
        &self,
        pkg: &PackageName,
        ver: &PackageVersion,
        file: F,
    ) -> Result<(), std::io::Error>
    where
        F: AsRef<Path>,
    {
        // Create the file.
        let mut file = File::create(file)?;

        file.write_all(b"EXPORTS\n")?;

        // Dumpt public types.
        for ty in &self.types {
            for func in ty.funcs() {
                file.write_all(b"    ")?;
                file.write_all(func.mangle(pkg, ver, ty).as_bytes())?;
                file.write_all(b"\n")?;
            }
        }

        Ok(())
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
