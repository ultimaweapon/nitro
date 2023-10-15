use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::Write;

/// A type that was exported from a package.
pub enum TypeDeclaration {
    Basic(BasicType),
}

impl TypeDeclaration {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_STRUCT: u8 = 2;
    const ENTRY_CLASS: u8 = 3;
    const ENTRY_FUNC: u8 = 4;

    /// Returns a fully qualified type name (no package name is prefixed).
    pub fn name(&self) -> &str {
        match self {
            Self::Basic(v) => v.name(),
        }
    }

    pub(super) fn serialize<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Name.
        let name = self.name();
        let len: u16 = name.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_NAME])?;
        w.write_all(&len.to_be_bytes())?;
        w.write_all(name.as_bytes())?;

        // Type.
        match self {
            Self::Basic(ty) => {
                w.write_all(&[if ty.is_class {
                    Self::ENTRY_CLASS
                } else {
                    Self::ENTRY_STRUCT
                }])?;

                // Functions.
                let len: u32 = ty.funcs.len().try_into().unwrap();

                w.write_all(&[Self::ENTRY_FUNC])?;
                w.write_all(&len.to_be_bytes())?;

                for f in &ty.funcs {
                    f.serialize(w)?;
                }
            }
        }

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

impl PartialEq for TypeDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for TypeDeclaration {}

impl Hash for TypeDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

/// A struct or class.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must be a struct and it will always public. Struct type cannot be a generic type and
/// does not supports inheritance.
///
/// Class in Nitro is a reference type, which mean any variable of a class type will be a pointer to
/// the heap allocated. All fields in the class will always private.
pub struct BasicType {
    is_class: bool,
    attrs: Attributes,
    name: String,
    funcs: HashSet<Function>,
}

impl BasicType {
    pub fn new(is_class: bool, attrs: Attributes, name: String, funcs: HashSet<Function>) -> Self {
        Self {
            is_class,
            attrs,
            name,
            funcs,
        }
    }

    pub fn is_class(&self) -> bool {
        self.is_class
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn funcs(&self) -> impl Iterator<Item = &Function> {
        self.funcs.iter()
    }
}

/// A function.
pub struct Function {
    name: String,
    params: Vec<FunctionParam>,
    ret: Type,
}

impl Function {
    const ENTRY_END: u8 = 0;
    const ENTRY_NAME: u8 = 1;
    const ENTRY_RET: u8 = 2;
    const ENTRY_PARAMS: u8 = 3;

    pub fn new(name: String, params: Vec<FunctionParam>, ret: Type) -> Self {
        Self { name, params, ret }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[FunctionParam] {
        &self.params
    }

    pub fn ret(&self) -> &Type {
        &self.ret
    }

    pub fn mangle(&self, lib: Option<(&str, u16)>, ty: &str) -> String {
        use std::fmt::Write;

        // Check if executable.
        let mut buf = String::new();

        match lib {
            Some((pkg, ver)) => {
                write!(buf, "_NEF{}{}", pkg.len(), pkg).unwrap();

                if ver != 0 {
                    write!(buf, "V{ver}").unwrap();
                }

                write!(buf, "T").unwrap();
            }
            None => write!(buf, "_NIF").unwrap(),
        }

        // Type name.
        for p in ty.split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
        }

        // Function name.
        write!(buf, "F{}{}", self.name.len(), self.name).unwrap();
        write!(buf, "0").unwrap(); // C calling convention.

        // Return type.
        self.ret.mangle(&mut buf);

        // Parameters.
        for p in &self.params {
            p.ty.mangle(&mut buf);
        }

        buf
    }

    fn serialize<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
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

        w.write_all(&[Self::ENTRY_PARAMS, len.try_into().unwrap()])?;

        for p in &self.params {
            p.serialize(w)?;
        }

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// A function parameter.
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }

    fn serialize<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Name.
        let len: u8 = self.name.len().try_into().unwrap();

        w.write_all(&[Self::ENTRY_NAME, len])?;
        w.write_all(self.name.as_bytes())?;

        // Type.
        w.write_all(&[Self::ENTRY_TYPE])?;
        self.ty.serialize(w)?;

        // End.
        w.write_all(&[Self::ENTRY_END])
    }
}

/// Type of something (e.g. function parameter).
pub enum Type {
    Unit {
        ptr: usize,
    },
    Never,
    Struct {
        ptr: usize,
        pkg: Option<(String, u16)>,
        name: String,
    },
    Class {
        ptr: usize,
        pkg: Option<(String, u16)>,
        name: String,
    },
}

impl Type {
    fn mangle(&self, buf: &mut String) {
        match self {
            Self::Unit { ptr } => {
                for _ in 0..*ptr {
                    buf.push('P');
                }
                buf.push('U');
            }
            Self::Never => buf.push('N'),
            Self::Struct { ptr, pkg, name } => {
                Self::mangle_basic(buf, false, *ptr, pkg.as_ref(), name)
            }
            Self::Class { ptr, pkg, name } => {
                Self::mangle_basic(buf, true, *ptr, pkg.as_ref(), name)
            }
        }
    }

    fn mangle_basic(
        buf: &mut String,
        class: bool,
        ptr: usize,
        pkg: Option<&(String, u16)>,
        name: &str,
    ) {
        use std::fmt::Write;

        for _ in 0..ptr {
            buf.push('P');
        }

        buf.push(if class { 'C' } else { 'S' });

        match pkg {
            Some((pkg, ver)) => {
                write!(buf, "E{}{}", pkg.len(), pkg).unwrap();

                if *ver != 0 {
                    write!(buf, "V{ver}T").unwrap();
                } else {
                    buf.push('T');
                }
            }
            None => buf.push('S'),
        }

        for p in name.split('.') {
            write!(buf, "{}{}", p.len(), p).unwrap();
        }
    }

    fn serialize<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // Check if struct or class.
        let (ptr, pkg, name) = match self {
            Self::Unit { ptr } => return w.write_all(&[0, (*ptr).try_into().unwrap()]),
            Self::Never => return w.write_all(&[3]),
            Self::Struct { ptr, pkg, name } => {
                w.write_all(&[1])?;
                (*ptr, pkg, name)
            }
            Self::Class { ptr, pkg, name } => {
                w.write_all(&[2])?;
                (*ptr, pkg, name)
            }
        };

        // Write prefixes.
        w.write_all(&[ptr.try_into().unwrap()])?;

        // Write package.
        match pkg {
            Some((pkg, ver)) => {
                w.write_all(&[pkg.len().try_into().unwrap()])?;
                w.write_all(pkg.as_bytes())?;
                w.write_all(&ver.to_be_bytes())?;
            }
            None => w.write_all(&[0])?,
        }

        // Write name.
        let len: u16 = name.len().try_into().unwrap();

        w.write_all(&len.to_be_bytes())?;
        w.write_all(name.as_bytes())
    }
}

/// A collection of attributes.
pub struct Attributes {
    public: Option<Public>,
    ext: Option<Extern>,
    repr: Option<Representation>,
}

impl Attributes {
    pub fn new(public: Option<Public>, ext: Option<Extern>, repr: Option<Representation>) -> Self {
        Self { public, ext, repr }
    }

    pub fn public(&self) -> Option<Public> {
        self.public
    }

    pub fn repr(&self) -> Option<Representation> {
        self.repr
    }
}

/// Argument of `@pub`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Public {
    External,
}

/// Argument of `@ext`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Extern {
    C,
}

/// Argument of `@repr`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    I32,
    U8,
    Un,
}
