use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::dirs;
use crate::download::LatestReleases;

/// Semver-like three-segment version number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Version(u32, u32, u32);
impl Version {
    pub fn from_str(s: &str) -> Self {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$").unwrap();
        }

        if let Some(cap) = RE.captures(&s) {
            Self(
                cap[1].parse::<u32>().unwrap(),
                cap[2].parse::<u32>().unwrap(),
                cap[3].parse::<u32>().unwrap(),
            )
        } else {
            panic!("Invalid version number {}", s);
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
                version: Version::from_str(s),
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
