use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use thiserror::Error;

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

    pub(super) fn deserialize<R>(mut r: R) -> Result<Self, TypeDeserializeError>
    where
        R: Read,
    {
        // Iterate over the entries.
        let mut name = None;
        let mut struc = false;
        let mut class = false;
        let mut funcs = HashSet::new();

        loop {
            // Read entry type.
            let mut entry = 0;

            r.read_exact(std::slice::from_mut(&mut entry))?;

            // Process the entry.
            match entry {
                Self::ENTRY_END => break,
                Self::ENTRY_NAME => {
                    // Read name length.
                    let mut buf = [0u8; 2];
                    r.read_exact(&mut buf)?;
                    let len: usize = u16::from_be_bytes(buf).into();

                    // Read name.
                    let mut buf = vec![0u8; len];
                    r.read_exact(&mut buf)?;

                    match String::from_utf8(buf) {
                        Ok(v) => name = Some(v),
                        Err(_) => return Err(TypeDeserializeError::InvalidTypeName),
                    }
                }
                Self::ENTRY_STRUCT => struc = true,
                Self::ENTRY_CLASS => class = true,
                Self::ENTRY_FUNC => {
                    // Read function count.
                    let mut buf = [0u8; 4];
                    r.read_exact(&mut buf)?;
                    let count: usize = u32::from_be_bytes(buf).try_into().unwrap();

                    // Read functions.
                    for i in 0..count {
                        if let Some(f) = funcs.replace(Function::deserialize(&mut r, i)?) {
                            return Err(TypeDeserializeError::DuplicatedFunction(f));
                        }
                    }
                }
                v => return Err(TypeDeserializeError::UnknownTypeEntry(v)),
            }
        }

        // Construct type.
        let name = name.ok_or(TypeDeserializeError::TypeNameNotFound)?;
        let ty = match (struc, class) {
            (true, true) | (false, false) => return Err(TypeDeserializeError::Ambiguity),
            (true, false) => Self::Basic(BasicType {
                is_class: false,
                attrs: Attributes {
                    public: None,
                    ext: None,
                    repr: None,
                },
                name,
                funcs,
            }),
            (false, true) => Self::Basic(BasicType {
                is_class: true,
                attrs: Attributes {
                    public: None,
                    ext: None,
                    repr: None,
                },
                name,
                funcs,
            }),
        };

        Ok(ty)
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
#[derive(Debug)]
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

    fn deserialize<R: Read>(mut r: R, i: usize) -> Result<Self, TypeDeserializeError> {
        // Iterate over the entries.
        let mut name = None;
        let mut params = Vec::new();
        let mut ret = None;

        loop {
            // Read entry type.
            let mut ty = 0;

            r.read_exact(std::slice::from_mut(&mut ty))?;

            // Process the entry.
            match ty {
                Self::ENTRY_END => break,
                Self::ENTRY_NAME => {
                    // Read name length.
                    let mut buf = [0u8; 2];
                    r.read_exact(&mut buf)?;
                    let len: usize = u16::from_be_bytes(buf).into();

                    // Read name.
                    let mut buf = vec![0u8; len];
                    r.read_exact(&mut buf)?;

                    match String::from_utf8(buf) {
                        Ok(v) => name = Some(v),
                        Err(_) => return Err(TypeDeserializeError::InvalidFunctionName(i)),
                    }
                }
                Self::ENTRY_RET => match Type::deserialize(&mut r) {
                    Some(v) => ret = Some(v),
                    None => return Err(TypeDeserializeError::InvalidFunctionRet(i)),
                },
                Self::ENTRY_PARAMS => {
                    // Read param count.
                    let mut buf = 0u8;
                    r.read_exact(std::slice::from_mut(&mut buf))?;
                    let count: usize = buf.into();

                    // Read params.
                    for p in 0..count {
                        params.push(FunctionParam::deserialize(&mut r, i, p)?);
                    }
                }
                v => return Err(TypeDeserializeError::UnknownFunctionEntry(i, v)),
            }
        }

        // Construct the function.
        let name = name.ok_or(TypeDeserializeError::FunctionNameNotFound(i))?;
        let ret = ret.ok_or(TypeDeserializeError::FunctionNameRetFound(i))?;

        Ok(Self { name, params, ret })
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

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let params: Vec<String> = self.params.iter().map(|p| p.to_string()).collect();

        write!(f, "fn {}({}): {}", self.name, params.join(", "), self.ret)
    }
}

/// A function parameter.
#[derive(Debug)]
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

    fn deserialize<R: Read>(mut r: R, f: usize, i: usize) -> Result<Self, TypeDeserializeError> {
        // Iterate over the entries.
        let mut name = None;
        let mut ty = None;

        loop {
            // Read entry type.
            let mut entry = 0;

            r.read_exact(std::slice::from_mut(&mut entry))?;

            // Process the entry.
            match entry {
                Self::ENTRY_END => break,
                Self::ENTRY_NAME => {
                    // Read name length.
                    let mut buf = 0u8;
                    r.read_exact(std::slice::from_mut(&mut buf))?;
                    let len: usize = buf.into();

                    // Read name.
                    let mut buf = vec![0u8; len];
                    r.read_exact(&mut buf)?;

                    match String::from_utf8(buf) {
                        Ok(v) => name = Some(v),
                        Err(_) => return Err(TypeDeserializeError::InvalidParamName(f, i)),
                    }
                }
                Self::ENTRY_TYPE => match Type::deserialize(&mut r) {
                    Some(v) => ty = Some(v),
                    None => return Err(TypeDeserializeError::InvalidParamType(f, i)),
                },
                v => return Err(TypeDeserializeError::UnknownParamEntry(f, i, v)),
            }
        }

        // Construct param.
        let name = name.ok_or(TypeDeserializeError::ParamNameNotFound(f, i))?;
        let ty = ty.ok_or(TypeDeserializeError::ParamTypeNotFound(f, i))?;

        Ok(Self { name, ty })
    }
}

impl Display for FunctionParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

/// Type of something (e.g. function parameter).
#[derive(Debug)]
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

    fn deserialize<R: Read>(mut r: R) -> Option<Self> {
        // Get category.
        let mut cat = 0;
        r.read_exact(std::slice::from_mut(&mut cat)).ok()?;

        // Check category.
        if cat == 3 {
            return Some(Self::Never);
        }

        // Get prefixes.
        let mut ptr = 0;
        r.read_exact(std::slice::from_mut(&mut ptr)).ok()?;

        if cat == 0 {
            return Some(Self::Unit { ptr: ptr.into() });
        }

        // Get package length.
        let mut len = 0;
        r.read_exact(std::slice::from_mut(&mut len)).ok()?;

        // Get package.
        let pkg = match len {
            0 => None,
            v => {
                // Read name.
                let mut buf = vec![0u8; v.into()];
                r.read_exact(&mut buf).ok()?;
                let name = String::from_utf8(buf).ok()?;

                // Read version.
                let mut buf = [0u8; 2];
                r.read_exact(&mut buf).ok()?;
                let ver = u16::from_be_bytes(buf);

                Some((name, ver))
            }
        };

        // Read name length.
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf).ok()?;
        let len: usize = u16::from_be_bytes(buf).into();

        // Read name.
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf).ok()?;
        let name = String::from_utf8(buf).ok()?;

        // Construct type.
        let ty = match cat {
            1 => Self::Struct {
                ptr: ptr.into(),
                pkg,
                name,
            },
            2 => Self::Class {
                ptr: ptr.into(),
                pkg,
                name,
            },
            _ => return None,
        };

        Some(ty)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit { ptr } => {
                for _ in 0..*ptr {
                    f.write_str("*")?;
                }
                f.write_str("()")
            }
            Self::Never => f.write_str("!"),
            Self::Struct { ptr, pkg, name } | Self::Class { ptr, pkg, name } => {
                for _ in 0..*ptr {
                    f.write_str("*")?;
                }

                match pkg {
                    Some((pkg, ver)) => write!(f, "{pkg}:{ver}.")?,
                    None => f.write_str("self.")?,
                }

                f.write_str(name)
            }
        }
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

/// Represents an error when [`TypeDeclaration`] is failed to deserialize from the data.
#[derive(Debug, Error)]
pub enum TypeDeserializeError {
    #[error("cannot read data")]
    ReadDataFailed(#[source] std::io::Error),

    #[error("invalid type name")]
    InvalidTypeName,

    #[error("invalid name for function #{0}")]
    InvalidFunctionName(usize),

    #[error("invalid name for parameter #{1} on function #{0}")]
    InvalidParamName(usize, usize),

    #[error("invalid type for parameter #{1} on function #{0}")]
    InvalidParamType(usize, usize),

    #[error("unknown entry {2} for parameter #{1} on function #{0}")]
    UnknownParamEntry(usize, usize, u8),

    #[error("name for parameter #{1} on function #{0} is not found")]
    ParamNameNotFound(usize, usize),

    #[error("type for parameter #{1} on function #{0} is not found")]
    ParamTypeNotFound(usize, usize),

    #[error("invalid return type for function #{0}")]
    InvalidFunctionRet(usize),

    #[error("unknown entry {1} for function #{0}")]
    UnknownFunctionEntry(usize, u8),

    #[error("name for function #{0} is not found")]
    FunctionNameNotFound(usize),

    #[error("return type for function #{0} is not found")]
    FunctionNameRetFound(usize),

    #[error("multiple definition of '{0}'")]
    DuplicatedFunction(Function),

    #[error("unknown type entry {0}")]
    UnknownTypeEntry(u8),

    #[error("type name not found")]
    TypeNameNotFound,

    #[error("type is ambiguity")]
    Ambiguity,
}

impl From<std::io::Error> for TypeDeserializeError {
    fn from(value: std::io::Error) -> Self {
        Self::ReadDataFailed(value)
    }
}
