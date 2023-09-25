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
    pub fn new(name: String, ty: Type) -> Self {
        Self { name, ty }
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
}
