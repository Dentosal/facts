use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::dirs;
use crate::download::LatestReleases;
use crate::error::InvalidVersionNumber;

/// Semver-like three-segment version number,
/// representing an exact released version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Version(u32, u32, u32);
impl Version {
    pub fn try_from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$").unwrap();
        }

        if let Some(cap) = RE.captures(&s) {
            Ok(Self(
                cap[1].parse::<u32>().unwrap(),
                cap[2].parse::<u32>().unwrap(),
                cap[3].parse::<u32>().unwrap(),
            ))
        } else {
            Err(Box::new(InvalidVersionNumber::Version(s.to_owned())))
        }
    }

    pub fn location(self) -> Result<PathBuf, PathBuf> {
        dirs::version_data(self)
    }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// Semver-like two-segment version number,
/// used to represent a group of compatible versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Version2(u32, u32);
impl Version2 {
    pub fn try_from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$").unwrap();
        }

        if let Some(cap) = RE.captures(&s) {
            Ok(Self(
                cap[1].parse::<u32>().unwrap(),
                cap[2].parse::<u32>().unwrap(),
            ))
        } else {
            Err(Box::new(InvalidVersionNumber::Version2(s.to_owned())))
        }
    }

    pub fn includes(self, version: Version) -> bool {
        self.extend_start() <= version && version < self.extend_after()
    }

    /// Extend with a zero in patch number, i.e. first version under this
    fn extend_start(self) -> Version {
        Version(self.0, self.1, 0)
    }

    /// Extend to next version after this, e.g. 0.1.2 -> 0.2.0
    fn extend_after(self) -> Version {
        Version(self.0, self.1 + 1, 0)
    }
}
impl fmt::Display for Version2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct ResolvedVersionReq {
    pub version: Version,
    /// Stability, if known
    pub stability_hint: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum VersionReq {
    // Latest stable
    Stable,
    // Latest experimental
    Experimental,
    // Specific version, possibly excluding minor and/or patch segments
    Specific(String),
}
impl FromStr for VersionReq {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, String> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*))?)?$").unwrap();
        }

        if input == "s" || input == "stable" {
            Ok(Self::Stable)
        } else if input == "e" || input == "experimental" {
            Ok(Self::Experimental)
        } else if RE.is_match(input) {
            Ok(Self::Specific(input.to_owned()))
        } else {
            Err(format!("Invalid version {:?}", input))
        }
    }
}
impl VersionReq {
    pub fn resolve(&self) -> Result<ResolvedVersionReq, Box<dyn std::error::Error>> {
        Ok(match self {
            Self::Specific(s) => ResolvedVersionReq {
                version: Version::try_from_str(s)?,
                stability_hint: None,
            },
            Self::Stable => ResolvedVersionReq {
                version: LatestReleases::get()?.stable,
                stability_hint: Some(true),
            },
            Self::Experimental => ResolvedVersionReq {
                version: LatestReleases::get()?.experimental,
                stability_hint: Some(false),
            },
        })
    }
}
impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Specific(s) => &s,
            Self::Stable => "latest stable",
            Self::Experimental => "latest experimental",
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version2range_includes() {
        let v01 = Version2::from_str("0.1");
        let v02 = Version2::from_str("0.2");
        let v03 = Version2::from_str("0.3");
        let v10 = Version2::from_str("1.0");
        let v20 = Version2::from_str("2.0");

        let vr0 = v01.to(v02);
        assert!(!vr0.includes(Version::from_str("0.0.9")));
        assert!(vr0.includes(Version::from_str("0.1.0")));
        assert!(vr0.includes(Version::from_str("0.1.3")));
        assert!(vr0.includes(Version::from_str("0.2.0")));
        assert!(vr0.includes(Version::from_str("0.2.3")));
        assert!(!vr0.includes(Version::from_str("0.3.0")));

        let vr1 = v10.to(v20);
        assert!(!vr1.includes(Version::from_str("0.0.9")));
        assert!(!vr1.includes(Version::from_str("0.1.0")));
        assert!(!vr1.includes(Version::from_str("0.1.3")));
        assert!(!vr1.includes(Version::from_str("0.2.0")));
        assert!(!vr1.includes(Version::from_str("0.2.3")));
        assert!(!vr1.includes(Version::from_str("0.3.0")));
        assert!(!vr1.includes(Version::from_str("0.9.0")));
        assert!(vr1.includes(Version::from_str("1.0.2")));
        assert!(vr1.includes(Version::from_str("1.3.4")));
        assert!(vr1.includes(Version::from_str("2.0.6")));
        assert!(!vr1.includes(Version::from_str("2.3.8")));
        assert!(!vr1.includes(Version::from_str("3.0.0")));

        let vr2 = v10.to(v10);
        assert!(!vr2.includes(Version::from_str("0.9.0")));
        assert!(vr2.includes(Version::from_str("1.0.2")));
        assert!(!vr2.includes(Version::from_str("1.3.4")));
        assert!(!vr2.includes(Version::from_str("2.0.6")));
        assert!(!vr2.includes(Version::from_str("2.3.8")));
        assert!(!vr2.includes(Version::from_str("3.0.0")));

        let vr3 = v02.to(v03);
        assert!(!vr3.includes(Version::from_str("0.1.0")));
        assert!(vr3.includes(Version::from_str("0.2.0")));
        assert!(vr3.includes(Version::from_str("0.2.5")));
        assert!(vr3.includes(Version::from_str("0.3.0")));
        assert!(vr3.includes(Version::from_str("0.3.5")));
        assert!(!vr3.includes(Version::from_str("1.0.2")));
        assert!(!vr3.includes(Version::from_str("1.3.4")));
        assert!(!vr3.includes(Version::from_str("2.0.6")));
        assert!(!vr3.includes(Version::from_str("2.3.8")));
        assert!(!vr3.includes(Version::from_str("3.0.0")));
    }
}
