/// Output target of the code.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Target {
    arch: Architecture,
    vendor: Vendor,
    os: OperatingSystem,
    env: Option<Environment>,
}

impl Target {
    pub const SUPPORTED_TARGETS: [Target; 4] = [
        Target {
            arch: Architecture::Aarch64,
            vendor: Vendor::Apple,
            os: OperatingSystem::Darwin,
            env: None,
        },
        Target {
            arch: Architecture::X86_64,
            vendor: Vendor::Apple,
            os: OperatingSystem::Darwin,
            env: None,
        },
        Target {
            arch: Architecture::X86_64,
            vendor: Vendor::Pc,
            os: OperatingSystem::Win32,
            env: Some(Environment::Msvc),
        },
        Target {
            arch: Architecture::X86_64,
            vendor: Vendor::Unknown,
            os: OperatingSystem::Linux,
            env: Some(Environment::Gnu),
        },
    ];

    pub fn arch(&self) -> Architecture {
        self.arch
    }

    pub fn os(&self) -> OperatingSystem {
        self.os
    }

    pub fn to_llvm(&self) -> String {
        let mut buf = String::with_capacity(64);

        buf.push_str(self.arch.name());

        match self.vendor {
            Vendor::Apple => buf.push_str("-apple"),
            Vendor::Pc => buf.push_str("-pc"),
            Vendor::Unknown => buf.push_str("-unknown"),
        }

        match self.os {
            OperatingSystem::Darwin => buf.push_str("-darwin"),
            OperatingSystem::Linux => buf.push_str("-linux"),
            OperatingSystem::Win32 => buf.push_str("-win32"),
        }

        if let Some(env) = self.env {
            match env {
                Environment::Gnu => buf.push_str("-gnu"),
                Environment::Msvc => buf.push_str("-msvc"),
            }
        }

        buf
    }
}

/// Architecture CPU of the target.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    Aarch64,
    X86_64,
}

impl Architecture {
    pub fn name(self) -> &'static str {
        match self {
            Architecture::Aarch64 => "aarch64",
            Architecture::X86_64 => "x86_64",
        }
    }
}

/// Vendor of the target.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vendor {
    Apple,
    Pc,
    Unknown,
}

/// OS of the target.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatingSystem {
    Darwin,
    Linux,
    Win32,
}

/// Environment of the target.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    Gnu,
    Msvc,
}
