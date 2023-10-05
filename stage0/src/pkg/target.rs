use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;
use uuid::{uuid, Uuid};

/// A struct to resolve primitive target.
pub struct TargetResolver {}

impl TargetResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve(
        &mut self,
        target: &Target,
    ) -> Result<&'static PrimitiveTarget, TargetResolveError> {
        match target {
            Target::Primitive(v) => Ok(v),
            Target::Custom(_) => todo!(),
        }
    }
}

/// Output target of the code.
#[derive(Debug, Clone)]
pub enum Target {
    Primitive(&'static PrimitiveTarget),
    Custom(Rc<CustomTarget>),
}

impl Target {
    pub fn id(&self) -> &Uuid {
        match self {
            Self::Primitive(v) => &v.id,
            Self::Custom(v) => &v.id,
        }
    }
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Target {}

impl Hash for Target {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(v) => v.fmt(f),
            Self::Custom(v) => v.fmt(f),
        }
    }
}

/// Contains data for a primitive target.
#[derive(Debug)]
pub struct PrimitiveTarget {
    id: Uuid,
    arch: TargetArch,
    vendor: TargetVendor,
    os: TargetOs,
    env: Option<TargetEnv>,
}

impl PrimitiveTarget {
    pub const ALL: [Self; 4] = [
        Self {
            id: uuid!("df56f1f4-8bee-4814-b6a7-e8b21ff72669"),
            arch: TargetArch::X86_64,
            vendor: TargetVendor::Unknown,
            os: TargetOs::Linux,
            env: Some(TargetEnv::Gnu),
        },
        Self {
            id: uuid!("27155b2c-a146-4c8a-b591-73aad7efb336"),
            arch: TargetArch::AArch64,
            vendor: TargetVendor::Apple,
            os: TargetOs::Darwin,
            env: None,
        },
        Self {
            id: uuid!("99e919be-e464-4e6a-a604-0242e8b751b9"),
            arch: TargetArch::X86_64,
            vendor: TargetVendor::Apple,
            os: TargetOs::Darwin,
            env: None,
        },
        Self {
            id: uuid!("69d6f6e5-dc4c-408d-acb8-b2a64db28b8b"),
            arch: TargetArch::X86_64,
            vendor: TargetVendor::Pc,
            os: TargetOs::Win32,
            env: Some(TargetEnv::Msvc),
        },
    ];

    pub fn arch(&self) -> TargetArch {
        self.arch
    }

    pub fn os(&self) -> TargetOs {
        self.os
    }
}

impl Display for PrimitiveTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.arch.name())?;
        f.write_str("-")?;
        f.write_str(self.vendor.name())?;
        f.write_str("-")?;
        f.write_str(self.os.name())?;

        if let Some(env) = &self.env {
            f.write_str("-")?;
            f.write_str(env.name())?;
        }

        Ok(())
    }
}

/// Contains data for a custom target.
#[derive(Debug)]
pub struct CustomTarget {
    id: Uuid,
    parent: Uuid,
}

impl Display for CustomTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

/// Architecture CPU of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetArch {
    AArch64,
    X86_64,
}

impl TargetArch {
    pub fn name(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64",
            Self::X86_64 => "x86_64",
        }
    }
}

/// Vendor of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetVendor {
    Apple,
    Pc,
    Unknown,
}

impl TargetVendor {
    pub fn name(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Pc => "pc",
            Self::Unknown => "unknown",
        }
    }
}

/// OS of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetOs {
    Darwin,
    Linux,
    Win32,
}

impl TargetOs {
    pub fn name(self) -> &'static str {
        match self {
            Self::Darwin => "darwin",
            Self::Linux => "linux",
            Self::Win32 => "win32",
        }
    }

    pub fn is_unix(self) -> bool {
        match self {
            Self::Darwin | Self::Linux => true,
            Self::Win32 => false,
        }
    }
}

/// Environment of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetEnv {
    Gnu,
    Msvc,
}

impl TargetEnv {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gnu => "gnu",
            Self::Msvc => "msvc",
        }
    }
}

/// Represents an error when [`TargetResolver`] is failed.
#[derive(Debug, Error)]
pub enum TargetResolveError {}
