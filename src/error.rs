use std::error::Error;
use std::fmt;

use crate::version::{Version, VersionReq};

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
