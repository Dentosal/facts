use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use strum_macros::EnumString;

use crate::dirs::credentials_file;
use crate::version::VersionReq;

#[derive(Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Password {
    pub plaintext: String,
}
impl FromStr for Password {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            plaintext: s.to_owned(),
        })
    }
}
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(...)")
    }
}
impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(...)")
    }
}

/// Factorio mod portal credentials
#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct LoginCredentials {
    pub username: Option<String>,
    pub password: Option<Password>,
}

/// Factorio API credentials
#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenCredentials {
    pub username: String,
    pub token: Password,
}
impl TokenCredentials {
    pub fn load() -> Option<Self> {
        serde_json::from_slice(&fs::read(credentials_file()).ok()?).unwrap()
    }

    pub fn store(&self) {
        fs::write(credentials_file(), serde_json::to_string(self).unwrap())
            .expect("Could not write file");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, StructOpt, Deserialize, Serialize)]
#[structopt(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum AutoUpdate {
    /// Kicks players out to update whenever possible
    Forced,
    /// Restarts server when no players are connected
    Enabled,
    /// Only auto updates on startup
    Startup,
    /// Never autoupdates
    Disabled,
}
impl AutoUpdate {
    pub fn live(self) -> bool {
        match self {
            Self::Forced => true,
            Self::Enabled => true,
            Self::Startup => false,
            Self::Disabled => false,
        }
    }
}

/// Configuration that is persisted per-server by facts
#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct MetaConfig {
    /// Version of Factorio to use
    #[structopt(long, default_value = "stable")]
    pub factorio: VersionReq,

    /// Automatically apply patches
    #[structopt(long, default_value = "enabled")]
    pub autoupdate: AutoUpdate,

    /// Automatically apply patches
    #[structopt(long, default_value = "60")]
    pub autoupdate_interval_minutes: u64,
}
impl MetaConfig {
    pub fn apply_update(&mut self, update: MetaConfigUpdate) {
        if let Some(v) = update.factorio {
            self.factorio = v;
        }
        if let Some(v) = update.autoupdate {
            self.autoupdate = v;
        }
        if let Some(v) = update.autoupdate_interval_minutes {
            self.autoupdate_interval_minutes = v;
        }
    }
}

/// Configuration that is persisted per-server by facts
#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct MetaConfigUpdate {
    #[structopt(long)]
    pub factorio: Option<VersionReq>,
    #[structopt(long)]
    pub autoupdate: Option<AutoUpdate>,
    #[structopt(long)]
    pub autoupdate_interval_minutes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct ImportConfig {
    /// Path to server-settings.json
    #[structopt(long)]
    pub server_settings: Option<PathBuf>,

    /// Path to server-adminlist.json
    #[structopt(long)]
    pub server_adminlist: Option<PathBuf>,

    /// Add server admins
    #[structopt(long)]
    pub add_admin: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, StructOpt, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct CreateConfig {
    /// Path to map-gen-settings.json
    #[structopt(long)]
    pub map_gen_settings: Option<PathBuf>,

    /// Path to map-settings.json
    #[structopt(long)]
    pub map_settings: Option<PathBuf>,

    #[structopt(flatten)]
    pub import: ImportConfig,

    #[structopt(flatten)]
    pub meta: MetaConfig,
}

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
#[structopt(rename_all = "kebab-case")]
pub enum Args {
    /// Creates a new server, downloading necessary binaries and data files
    Create {
        /// Name for the server
        name: String,

        #[structopt(flatten)]
        config: CreateConfig,
    },
    /// Import existing world to facts
    Import {
        /// Name for the server
        name: String,

        /// World file
        path: PathBuf,

        #[structopt(flatten)]
        config: ImportConfig,

        #[structopt(flatten)]
        meta: MetaConfig,
    },
    /// Export world to a zip file
    Export {
        /// Name for the server
        name: String,

        /// World file
        path: PathBuf,

        /// Allow overwriting target file
        #[structopt(long)]
        force: bool,
    },
    /// Edits server config
    Edit {
        /// Name of the server
        name: String,

        #[structopt(flatten)]
        config: ImportConfig,

        #[structopt(flatten)]
        meta: MetaConfigUpdate,
    },
    /// Log in to the mod portal, optionally save credentials
    Login {
        #[structopt(flatten)]
        credentials: LoginCredentials,
    },
    /// List server mods
    ListMods {
        /// Name of the server
        name: String,
    },
    /// Adds server mods
    AddMod {
        /// Name of the server
        name: String,

        mods: Vec<String>,
    },
    /// Adds server mods
    RemoveMod {
        /// Name of the server
        name: String,

        mods: Vec<String>,
    },
    /// Update server mods
    UpdateMods {
        /// Name of the server
        name: String,
    },
    /// Display server config
    Show {
        /// Name of the server
        name: String,
    },
    /// Update server (and mods) to latest available one
    Update {
        /// Name of the server
        name: String,
    },
    /// Delete server
    Delete {
        /// Name of the server
        name: String,
        /// Skip confirmation prompt
        #[structopt(long)]
        force: bool,
    },
    /// List all servers
    List {
        #[structopt(short, long)]
        extended: bool,
    },
    /// Remove all unused files
    Prune,
    /// Starts a server
    Start {
        /// Name of the server
        name: String,
    },
}
