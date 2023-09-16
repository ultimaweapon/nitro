use serde::de::{Error, Unexpected, Visitor};
use serde::Deserialize;
use std::fmt::Formatter;

/// Metadata for a Nitro package.
pub struct PackageMeta {
    name: String,
    version: PackageVersion,
}

impl PackageMeta {
    pub fn new(name: String, version: PackageVersion) -> Self {
        Self { name, version }
    }
}

/// A version of a Nitro package.
///
/// This is an implementation of https://semver.org.
#[derive(Clone)]
pub struct PackageVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl PackageVersion {
    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl<'a> Deserialize<'a> for PackageVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(PackageVersionVisitor)
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
