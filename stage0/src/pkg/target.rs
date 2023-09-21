/// Output target of the code.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Target {
    arch: Architecture,
    vendor: Vendor,
    os: OperatingSystem,
    env: Option<Environment>,
}

impl Target {
    pub fn parse(triple: &str) -> Self {
        let mut triple = triple.split('-').fuse();
        let target = Self {
            arch: match triple.next().unwrap() {
                "aarch64" => Architecture::Aarch64,
                "x86_64" => Architecture::X86_64,
                _ => todo!(),
            },
            vendor: match triple.next().unwrap() {
                "apple" => Vendor::Apple,
                "pc" => Vendor::Pc,
                "unknown" => Vendor::Unknown,
                _ => todo!(),
            },
            os: match triple.next().unwrap() {
                "darwin" => OperatingSystem::Darwin,
                "linux" => OperatingSystem::Linux,
                "win32" => OperatingSystem::Win32,
                _ => todo!(),
            },
            env: triple.next().map(|v| match v {
                "gnu" => Environment::Gnu,
                "msvc" => Environment::Msvc,
                _ => todo!(),
            }),
        };

        assert!(triple.next().is_none());

        target
    }

    pub fn os(&self) -> OperatingSystem {
        self.os
    }

    pub fn to_llvm(&self) -> String {
        let mut buf = String::with_capacity(64);

        match self.arch {
            Architecture::Aarch64 => buf.push_str("aarch64"),
            Architecture::X86_64 => buf.push_str("x86_64"),
        }

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
