//! SIP version
//!
//! This module contains a definition of the SIP `Version` type. You should
//! not directly use the type from this module but rather the
//! `imersio_sip::Version` type.
//!
//! The `Version` type contains constants that represent the various versions
//! of the SIP protocol (only one version `SIP/2.0` for now).
//!
//! # Example
//!
//! ```
//! use imersio_sip::Version;
//! println!("{}", Version::SIP_2);
//! ```

use crate::Error;

/// Represents a version of the SIP specification.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Version(Sip);

#[derive(Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
enum Sip {
    Sip2,
}

impl Version {
    /// `SIP/2.0`
    pub const SIP_2: Version = Version(Sip::Sip2);

    /// Return a &str representation of the SIP method.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self.0 {
            Sip::Sip2 => "SIP/2.0",
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse(value)
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for Version {
    #[inline]
    fn default() -> Version {
        Version::SIP_2
    }
}

impl AsRef<str> for Version {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a Version> for Version {
    #[inline]
    fn eq(&self, other: &&'a Version) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Version> for &'a Version {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Version {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Version> for str {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Version {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Version> for &'a str {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        *self == other.as_ref()
    }
}

fn parse(input: &str) -> Result<Version, Error> {
    match parser::sip_version(input) {
        Ok((rest, version)) => {
            if !rest.is_empty() {
                Err(Error::RemainingUnparsedData)
            } else {
                Ok(version)
            }
        }
        Err(e) => Err(Error::InvalidVersion(e.to_string())),
    }
}

pub(crate) mod parser {
    use crate::{parser::ParserResult, Version};
    use nom::{bytes::complete::tag, combinator::value, error::context};

    pub(crate) fn sip_version(input: &str) -> ParserResult<&str, Version> {
        context("sip_version", value(Version::SIP_2, tag("SIP/2.0")))(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::assert_err;

    #[test]
    fn test_version_eq() {
        assert_eq!(Version::SIP_2.to_string(), "SIP/2.0");

        assert_eq!(Version::SIP_2, "SIP/2.0");
        assert_eq!(&Version::SIP_2, "SIP/2.0");

        assert_eq!("SIP/2.0", Version::SIP_2);
        assert_eq!("SIP/2.0", &Version::SIP_2);

        assert_eq!(&Version::SIP_2, Version::SIP_2);
        assert_eq!(Version::SIP_2, &Version::SIP_2);
    }

    #[test]
    fn test_invalid_version() {
        assert_err!(Version::try_from(""));
        assert_err!(Version::try_from("SIP/1.0"));
        assert_err!(Version::try_from("crappy-version"));
    }

    #[test]
    fn test_valid_version() {
        assert_eq!(Version::try_from("SIP/2.0").unwrap(), "SIP/2.0");
    }
}