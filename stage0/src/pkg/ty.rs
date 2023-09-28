use super::{PackageName, PackageVersion};
use std::collections::HashSet;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

/// A type that was exported from a package.
pub struct ExportedType {
    name: String,
    funcs: HashSet<ExportedFunc>,
}

impl ExportedType {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_FUNC: u8 = 2;

    pub fn new(name: String) -> Self {
        Self {
            name,
            funcs: HashSet::new(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn funcs(&self) -> impl Iterator<Item = &ExportedFunc> {
        self.funcs.iter()
    }

    pub fn add_func(&mut self, f: ExportedFunc) {
        assert!(self.funcs.insert(f));
    }

    pub(super) fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Type name.
        let len: u16 = self.name.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_NAME])?;
        w.write_all(&len.to_be_bytes())?;
        w.write_all(self.name.as_bytes())?;

        // Functions.
        let len: u32 = self.funcs.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_FUNC])?;
        w.write_all(&len.to_be_bytes())?;

        for f in &self.funcs {
            f.serialize(w)?;
        }

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

impl PartialEq for ExportedType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ExportedType {}

impl Hash for ExportedType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// A function that was exported from a package.
pub struct ExportedFunc {
    name: String,
    params: Vec<FunctionParam>,
    ret: Type,
}

impl ExportedFunc {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_RET: u8 = 2;
    const ENTRY_PARAMS: u8 = 3;

    pub fn new(name: String, params: Vec<FunctionParam>, ret: Type) -> Self {
        Self { name, params, ret }
    }

    pub fn mangle(&self, pkg: &PackageName, ver: &PackageVersion, ty: &ExportedType) -> String {
        // Package name.
        let mut buf = String::new();
        let pkg = pkg.as_str();

        write!(buf, "_N{}{}", pkg.len(), pkg).unwrap();

        // Package version.
        if ver.major() != 0 {
            write!(buf, "V{}", ver.major()).unwrap();
        }

        // Type name.
        write!(buf, "T").unwrap();

        for p in ty.name.split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
        }

        // Function name.
        write!(buf, "F{}{}", self.name.len(), self.name).unwrap();
        write!(buf, "0").unwrap(); // C calling convention.

        // Return type.
        self.ret.mangle(&mut buf);

        // Parameters.
        for p in self.params.iter().map(|p| &p.ty) {
            p.mangle(&mut buf);
        }

        buf
    }

    fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Name.
        let len: u16 = self.name.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_NAME])?;
        w.write_all(&len.to_be_bytes())?;
        w.write_all(self.name.as_bytes())?;

        // Return.
        w.write_all(&[Self::ENTRY_RET])?;
        self.ret.serialize(w)?;

        // Params.
        let len: u8 = self.params.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_PARAMS])?;
        w.write_all(&[len])?;

        for p in &self.params {
            p.serialize(w)?;
        }

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

impl PartialEq for ExportedFunc {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ExportedFunc {}

impl Hash for ExportedFunc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Parameter of a function.
pub struct FunctionParam {
    name: String,
    ty: Type,
}

impl FunctionParam {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_TYPE: u8 = 2;

    pub fn new(name: String, ty: Type) -> Self {
        Self { name, ty }
    }

    fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Name.
        let len: u8 = self.name.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_NAME])?;
        w.write_all(&[len])?;
        w.write_all(self.name.as_bytes())?;

        // Type.
        w.write_all(&[Self::ENTRY_TYPE])?;
        self.ty.serialize(w)?;

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

/// Type of something.
pub enum Type {
    Unit(usize),
    Never,
    Local(usize, String),
    External(usize, String, u16, String),
}

impl Type {
    fn mangle(&self, buf: &mut String) {
        match self {
            Self::Unit(p) => {
                for _ in 0..*p {
                    buf.push('P');
                }

                buf.push('U');
            }
            Self::Never => buf.push('N'),
            Self::Local(p, n) => {
                for _ in 0..*p {
                    buf.push('P');
                }

                buf.push('S');

                for p in n.split('.') {
                    write!(buf, "{}{}", p.len(), p).unwrap();
                }
            }
            Self::External(ptr, pkg, ver, path) => {
                for _ in 0..*ptr {
                    buf.push('P');
                }

                write!(buf, "E{}{}", pkg.len(), pkg).unwrap();

                if *ver != 0 {
                    write!(buf, "V{ver}T").unwrap();
                } else {
                    buf.push('T');
                }

                for p in path.split('.') {
                    write!(buf, "{}{}", p.len(), p).unwrap();
                }
            }
        }
    }

    fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        match self {
            Self::Unit(p) => w.write_all(&[0, (*p).try_into().unwrap()])?,
            Self::Never => w.write_all(&[3])?,
            Self::Local(p, n) => {
                let l: u16 = n.len().try_into().unwrap();

                w.write_all(&[1, (*p).try_into().unwrap()])?;
                w.write_all(&l.to_be_bytes())?;
                w.write_all(n.as_bytes())?;
            }
            Self::External(ptr, pkg, ver, path) => {
                let l: u16 = path.len().try_into().unwrap();

                w.write_all(&[2, (*ptr).try_into().unwrap()])?;
                w.write_all(&[pkg.len().try_into().unwrap()])?;
                w.write_all(pkg.as_bytes())?;
                w.write_all(&ver.to_be_bytes())?;
                w.write_all(&l.to_be_bytes())?;
                w.write_all(path.as_bytes())?;
            }
        }

        Ok(())
    }
}
