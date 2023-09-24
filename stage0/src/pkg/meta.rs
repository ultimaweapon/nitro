use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};

/// Metadata for a Nitro package.
pub struct PackageMeta {
    name: PackageName,
    version: PackageVersion,
}

impl PackageMeta {
    pub fn new(name: PackageName, version: PackageVersion) -> Self {
        Self { name, version }
    }

    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn version(&self) -> &PackageVersion {
        &self.version
    }
}

/// Name of a Nitro package.
///
/// A package name must start with a lower case ASCII and followed by zero of more 0-9 and a-z (only
/// lower case). The maximum length is 32 characters.
#[derive(Clone)]
pub struct PackageName(String);

impl PackageName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_bin(&self) -> [u8; 32] {
        let mut bin = [0; 32];
        let src = self.0.as_bytes();

        bin[..src.len()].copy_from_slice(src);
        bin
    }

    fn is_valid(v: &str) -> bool {
        // Check length.
        if v.is_empty() || v.len() > 32 {
            return false;
        }

        // Check first char.
        let mut i = v.as_bytes().into_iter();
        let b = i.next().unwrap();

        if !b.is_ascii_lowercase() {
            return false;
        }

        // Check remaining chars.
        for b in i {
            if b.is_ascii_digit() || b.is_ascii_lowercase() {
                continue;
            }

            return false;
        }

        true
    }
}

impl<'a> Deserialize<'a> for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(PackageNameVisitor)
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A version of a Nitro package.
///
/// This is an implementation of https://semver.org.
#[derive(Clone)]
pub struct PackageVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl PackageVersion {
    pub fn major(&self) -> u16 {
        self.major
    }

    pub fn to_bin(&self) -> u64 {
        let major: u64 = self.major.into();
        let minor: u64 = self.minor.into();
        let patch: u64 = self.patch.into();

        (major << 32) | (minor << 16) | patch
    }
}

impl<'a> Deserialize<'a> for PackageVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(PackageVersionVisitor)
    }
}

/// An implementation of [`Visitor`] for [`PackageName`].
struct PackageNameVisitor;

impl<'a> Visitor<'a> for PackageNameVisitor {
    type Value = PackageName;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string begin with a-z and followed by zero or more 0-9 and a-z")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if PackageName::is_valid(value) {
            Ok(PackageName(value.to_owned()))
        } else {
            Err(Error::invalid_value(Unexpected::Str(value), &self))
        }
    }
}

/// An implementation of [`Visitor`] for [`PackageVersion`].
struct PackageVersionVisitor;

impl<'a> Visitor<'a> for PackageVersionVisitor {
    type Value = PackageVersion;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string with 'major.minor.patch' format")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut parts = value.splitn(3, '.');
        let major = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::invalid_value(Unexpected::Str(value), &self))?;
        let minor = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::invalid_value(Unexpected::Str(value), &self))?;
        let patch = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| Error::invalid_value(Unexpected::Str(value), &self))?;

        assert_eq!(parts.next(), None);

        Ok(PackageVersion {
            major,
            minor,
            patch,
        })
    }
}
