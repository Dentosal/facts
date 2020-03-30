use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::version::{Version, VersionReq};

#[derive(Debug)]
#[must_use]
pub struct OutputFileAlreadyExists(pub PathBuf);
impl fmt::Display for OutputFileAlreadyExists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "File {:?} already exists, use --force to overwrite",
            self.0
        )
    }
}
impl Error for OutputFileAlreadyExists {}

#[derive(Debug)]
#[must_use]
pub struct WorldAlreadyExists(pub String);
impl fmt::Display for WorldAlreadyExists {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "World {} already exists", self.0)
    }
}
impl Error for WorldAlreadyExists {}

#[derive(Debug)]
#[must_use]
pub struct NoSuchWorld(pub String);
impl fmt::Display for NoSuchWorld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "World {} does not exist", self.0)
    }
}
impl Error for NoSuchWorld {}

#[derive(Debug)]
#[must_use]
pub struct NoDownloadAvailable(pub Version);
impl fmt::Display for NoDownloadAvailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No download link available for version {:?}", self.0)
    }
}
impl Error for NoDownloadAvailable {}

#[derive(Debug)]
#[must_use]
pub struct NoSuchMod(pub String);
impl fmt::Display for NoSuchMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No mod named {:?}", self.0)
    }
}
impl Error for NoSuchMod {}

#[derive(Debug)]
#[must_use]
pub struct NoMatchingModVersions(pub String, pub Version);
impl fmt::Display for NoMatchingModVersions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "No supported versions of mod {:?} for Factorio version {:?}",
            self.0, self.1
        )
    }
}
impl Error for NoMatchingModVersions {}

#[derive(Debug)]
#[must_use]
pub struct DowngradingNotAllowed {
    pub current: Version,
    pub requested: VersionReq,
}
impl fmt::Display for DowngradingNotAllowed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Current server version ({}) is newer than the requested one ({})",
            self.current, self.requested
        )
    }
}
impl Error for DowngradingNotAllowed {}

#[derive(Debug, Clone)]
pub enum ServerError {
    PortUnavailable,
}
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            ServerError::PortUnavailable =>
                "UDP port already in use. Is there another server running?",
        })
    }
}
impl Error for ServerError {}

#[derive(Debug, Clone)]
pub enum InvalidVersionNumber {
    Version(String),
    Version2(String),
}
impl fmt::Display for InvalidVersionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidVersionNumber::Version(s) => write!(
                f,
                "Invalid version number: {:?} (required major.minor.patch)",
                s
            ),
            InvalidVersionNumber::Version2(s) => {
                write!(f, "Invalid version number: {:?} (required major.minor)", s)
            },
        }
    }
}
impl Error for InvalidVersionNumber {}

#[derive(Debug, Clone)]
pub struct InternalDataModified(pub String);
impl fmt::Display for InternalDataModified {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Internal data was modified incorrectly: {}", self.0)
    }
}
impl Error for InternalDataModified {}

#[derive(Debug, Clone)]
pub struct NotLoggedIn;
impl fmt::Display for NotLoggedIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Log in first with `facts login USERNAME PASSWORD`")
    }
}
impl Error for NotLoggedIn {}

#[derive(Debug, Clone)]
pub struct LoginFailed(pub String);
impl fmt::Display for LoginFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Login failed: {}", self.0)
    }
}
impl Error for LoginFailed {}
