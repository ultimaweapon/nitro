use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
}

impl PartialEq<str> for PackageName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
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

impl FromStr for PackageName {
    type Err = PackageNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check length.
        if s.is_empty() {
            return Err(PackageNameError::EmptyName);
        } else if s.len() > 32 {
            return Err(PackageNameError::NameTooLong);
        }

        // Check first char.
        let mut i = s.as_bytes().into_iter();
        let b = i.next().unwrap();

        if !b.is_ascii_lowercase() {
            return Err(PackageNameError::NotStartWithLowerCase);
        }

        // Check remaining chars.
        for b in i {
            if b.is_ascii_digit() || b.is_ascii_lowercase() {
                continue;
            }

            return Err(PackageNameError::NotDigitOrLowerCase);
        }

        Ok(Self(s.to_owned()))
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl PackageVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

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

impl FromStr for PackageVersion {
    type Err = PackageVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse major.
        let mut parts = s.splitn(3, '.');
        let major = parts
            .next()
            .unwrap()
            .parse()
            .map_err(|e| PackageVersionError::InvalidMajor(e))?;

        // Parse minor.
        let minor = match parts.next() {
            Some(v) => v
                .parse()
                .map_err(|e| PackageVersionError::InvalidMinor(e))?,
            None => return Err(PackageVersionError::NoMinor),
        };

        // Parse patch.
        let patch = match parts.next() {
            Some(v) => v
                .parse()
                .map_err(|e| PackageVersionError::InvalidPatch(e))?,
            None => return Err(PackageVersionError::NoPatch),
        };

        assert_eq!(parts.next(), None);

        Ok(PackageVersion {
            major,
            minor,
            patch,
        })
    }
}

impl Display for PackageVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
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
        value
            .parse()
            .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))
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
        value
            .parse()
            .map_err(|_| Error::invalid_value(Unexpected::Str(value), &self))
    }
}

/// Represents an error when [`PackageName`] is failed to construct.
#[derive(Debug, Error)]
pub enum PackageNameError {
    #[error("name cannot be empty")]
    EmptyName,

    #[error("name cannot exceed 32 bytes")]
    NameTooLong,

    #[error("name must start with a lower-case ASCII")]
    NotStartWithLowerCase,

    #[error("name cannot contains other alphabet except digits or lowe-case ASCIIs")]
    NotDigitOrLowerCase,
}

/// Represents an error when [`PackageVersion`] is failed to construct.
#[derive(Debug, Error)]
pub enum PackageVersionError {
    #[error("invalid major version")]
    InvalidMajor(#[source] ParseIntError),

    #[error("no minor version")]
    NoMinor,

    #[error("invalid minor version")]
    InvalidMinor(#[source] ParseIntError),

    #[error("no patch number")]
    NoPatch,

    #[error("invalid patch number")]
    InvalidPatch(#[source] ParseIntError),
}
