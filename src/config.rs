use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use strum_macros::EnumString;

use crate::version::VersionReq;

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
    /// Edits server config
    Edit {
        /// Name of the server
        name: String,

        #[structopt(flatten)]
        config: ImportConfig,

        #[structopt(flatten)]
        meta: MetaConfigUpdate,
    },
    /// Display server config
    Show {
        /// Name of the server
        name: String,
    },
    /// Update server to latest available one
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
